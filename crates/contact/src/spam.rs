//! Spam protection: a honeypot field, a minimum time between serving the form
//! and submitting it, and a per-IP rate limit. All in memory: fine for one
//! server process; a restart forgets everything, which only ever errs on the
//! side of letting a real person through.

use std::collections::hash_map::RandomState;
use std::collections::{HashMap, VecDeque};
use std::hash::{BuildHasher, Hasher};
use std::net::IpAddr;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// Cookie carrying the form token.
pub const TOKEN_COOKIE: &str = "arthub_ct";

/// Sliding-window limiter: at most `max` accepted submissions per `window`.
pub struct RateLimiter {
    max: usize,
    window: Duration,
    hits: Mutex<HashMap<IpAddr, VecDeque<Instant>>>,
}

impl RateLimiter {
    pub fn new(max: usize, window: Duration) -> Self {
        Self {
            max,
            window,
            hits: Mutex::new(HashMap::new()),
        }
    }

    fn prune(q: &mut VecDeque<Instant>, now: Instant, window: Duration) {
        while q.front().is_some_and(|t| now.duration_since(*t) >= window) {
            q.pop_front();
        }
    }

    /// Would one more submission from `ip` be allowed right now?
    pub fn allows(&self, ip: IpAddr, now: Instant) -> bool {
        let mut hits = self.hits.lock().unwrap();
        let q = hits.entry(ip).or_default();
        Self::prune(q, now, self.window);
        q.len() < self.max
    }

    /// Count an accepted submission from `ip`.
    pub fn record(&self, ip: IpAddr, now: Instant) {
        let mut hits = self.hits.lock().unwrap();
        // Keep memory bounded: drop idle addresses now and then.
        if hits.len() > 10_000 {
            hits.retain(|_, q| {
                q.back()
                    .is_some_and(|t| now.duration_since(*t) < self.window)
            });
        }
        let q = hits.entry(ip).or_default();
        Self::prune(q, now, self.window);
        q.push_back(now);
    }
}

/// One-time tokens recording when the form was served.
pub struct FormTokens {
    ttl: Duration,
    issued: Mutex<HashMap<String, Instant>>,
    seq: AtomicU64,
    seed: RandomState,
}

impl FormTokens {
    pub fn new(ttl: Duration) -> Self {
        Self {
            ttl,
            issued: Mutex::new(HashMap::new()),
            seq: AtomicU64::new(0),
            seed: RandomState::new(),
        }
    }

    /// A fresh unguessable token, valid for `ttl`.
    pub fn issue(&self, now: Instant) -> String {
        let n = self.seq.fetch_add(1, Ordering::Relaxed);
        let mut parts = [0u64; 2];
        for (i, p) in parts.iter_mut().enumerate() {
            let mut h = self.seed.build_hasher();
            h.write_u64(n);
            h.write_usize(i);
            h.write_u128(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos(),
            );
            *p = h.finish();
        }
        let token = format!("{:016x}{:016x}", parts[0], parts[1]);
        let mut issued = self.issued.lock().unwrap();
        if issued.len() > 50_000 {
            issued.retain(|_, t| now.duration_since(*t) < self.ttl);
        }
        issued.insert(token.clone(), now);
        token
    }

    /// How long ago `token` was issued (it stays valid for retries until it
    /// expires). `None` if unknown or expired.
    pub fn age(&self, token: &str, now: Instant) -> Option<Duration> {
        let issued = self.issued.lock().unwrap();
        let t = issued.get(token)?;
        let age = now.duration_since(*t);
        (age < self.ttl).then_some(age)
    }
}

/// What to do with a submission before it's even validated.
#[derive(Debug, PartialEq, Eq)]
pub enum Gate {
    Open,
    /// The honeypot was filled: pretend it worked, send nothing.
    Honeypot,
    /// Submitted faster than a person could have typed it.
    TooFast,
    RateLimited,
}

pub struct Guard {
    pub limiter: RateLimiter,
    pub tokens: FormTokens,
    pub min_fill_time: Duration,
}

impl Default for Guard {
    /// 5 submissions per IP per hour; at least 3 s on the form.
    fn default() -> Self {
        Self {
            limiter: RateLimiter::new(5, Duration::from_secs(3600)),
            tokens: FormTokens::new(Duration::from_secs(6 * 3600)),
            min_fill_time: Duration::from_secs(3),
        }
    }
}

impl Guard {
    /// `token` is the form token cookie, if the browser sent one. A missing
    /// or expired token isn't held against anyone (cookies may be off); the
    /// honeypot and rate limit still apply.
    pub fn check(&self, honeypot: &str, token: Option<&str>, ip: IpAddr, now: Instant) -> Gate {
        if !honeypot.trim().is_empty() {
            return Gate::Honeypot;
        }
        if token
            .and_then(|t| self.tokens.age(t, now))
            .is_some_and(|age| age < self.min_fill_time)
        {
            return Gate::TooFast;
        }
        if !self.limiter.allows(ip, now) {
            return Gate::RateLimited;
        }
        Gate::Open
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ip(n: u8) -> IpAddr {
        IpAddr::from([10, 0, 0, n])
    }

    #[test]
    fn honeypot_is_rejected_first() {
        let g = Guard::default();
        let t0 = Instant::now();
        assert_eq!(
            g.check("http://spam.example", None, ip(1), t0),
            Gate::Honeypot
        );
        assert_eq!(g.check("  ", None, ip(1), t0), Gate::Open);
    }

    #[test]
    fn too_fast_then_fine() {
        let g = Guard::default();
        let t0 = Instant::now();
        let token = g.tokens.issue(t0);
        assert_eq!(
            g.check("", Some(&token), ip(1), t0 + Duration::from_millis(800)),
            Gate::TooFast
        );
        assert_eq!(
            g.check("", Some(&token), ip(1), t0 + Duration::from_secs(4)),
            Gate::Open
        );
    }

    #[test]
    fn unknown_or_missing_token_is_not_held_against_anyone() {
        let g = Guard::default();
        let t0 = Instant::now();
        assert_eq!(g.check("", None, ip(1), t0), Gate::Open);
        assert_eq!(g.check("", Some("made-up"), ip(1), t0), Gate::Open);
    }

    #[test]
    fn tokens_expire_and_are_unique() {
        let tokens = FormTokens::new(Duration::from_secs(60));
        let t0 = Instant::now();
        let (a, b) = (tokens.issue(t0), tokens.issue(t0));
        assert_ne!(a, b);
        assert_eq!(a.len(), 32);
        assert!(tokens.age(&a, t0 + Duration::from_secs(59)).is_some());
        assert!(tokens.age(&a, t0 + Duration::from_secs(61)).is_none());
    }

    #[test]
    fn rate_limit_five_per_hour_per_ip() {
        let g = Guard::default();
        let t0 = Instant::now();
        for i in 0..5 {
            let at = t0 + Duration::from_secs(i * 60);
            assert_eq!(g.check("", None, ip(1), at), Gate::Open, "submission {i}");
            g.limiter.record(ip(1), at);
        }
        let sixth = t0 + Duration::from_secs(300);
        assert_eq!(g.check("", None, ip(1), sixth), Gate::RateLimited);
        // Other addresses are unaffected.
        assert_eq!(g.check("", None, ip(2), sixth), Gate::Open);
        // An hour after the first, a slot frees up.
        assert_eq!(
            g.check("", None, ip(1), t0 + Duration::from_secs(3601)),
            Gate::Open
        );
    }

    #[test]
    fn checking_alone_does_not_use_up_the_limit() {
        let g = Guard::default();
        let t0 = Instant::now();
        for _ in 0..20 {
            assert_eq!(g.check("", None, ip(3), t0), Gate::Open);
        }
    }
}
