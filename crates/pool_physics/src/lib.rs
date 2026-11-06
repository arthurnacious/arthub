//! Pure-Rust 8-ball pool simulation.
//!
//! 2D, table centred at the origin, `x` along the length, `y` along the
//! width. Units are metres and seconds. The simulation advances in fixed
//! substeps of 1/480 s, so it is deterministic for a given sequence of
//! shots regardless of the frame rate used to drive [`PoolGame::step`].

pub use glam::Vec2;

/// Radius of every ball (2 1/4" diameter).
pub const BALL_RADIUS: f32 = 0.028575;

/// Fixed simulation substep.
const SUBSTEP: f32 = 1.0 / 480.0;
/// Maximum time consumed by one call to [`PoolGame::step`].
const MAX_FRAME_DT: f32 = 0.1;
/// Cue speed at `power = 1.0`.
const MAX_CUE_SPEED: f32 = 5.5;
/// Constant rolling-friction deceleration (m/s^2).
const ROLLING_DECEL: f32 = 0.18;
/// Speeds below this are clamped to zero (ball sleeps).
const SLEEP_SPEED: f32 = 0.01;
/// Ball-ball restitution.
const BALL_RESTITUTION: f32 = 0.95;
/// Cushion restitution (normal component).
const CUSHION_RESTITUTION: f32 = 0.75;
/// Tangential speed retained after a cushion hit.
const CUSHION_TANGENT_KEEP: f32 = 0.97;
/// Gap between racked balls.
const RACK_GAP: f32 = 1e-4;
/// Minimum impact speed that produces a hit/cushion event.
const EVENT_MIN_SPEED: f32 = 0.02;
/// Fraction of the pocket radius (measured along the rail from the pocket
/// centre) in which the cushion is absent, letting balls drop in.
const MOUTH_FACTOR: f32 = 0.9;

/// Playing surface geometry.
#[derive(Clone, Debug, PartialEq)]
pub struct Table {
    pub half_length: f32,
    pub half_width: f32,
    pub pocket_radius: f32,
    pub pockets: [Vec2; 6],
}

impl Table {
    /// A 9ft-ish table: 2.54 x 1.27 m playing surface, six pockets.
    pub fn standard() -> Self {
        let hl = 1.27;
        let hw = 0.635;
        Self {
            half_length: hl,
            half_width: hw,
            pocket_radius: 0.06,
            pockets: [
                Vec2::new(-hl, -hw),
                Vec2::new(0.0, -hw),
                Vec2::new(hl, -hw),
                Vec2::new(-hl, hw),
                Vec2::new(0.0, hw),
                Vec2::new(hl, hw),
            ],
        }
    }

    /// Head spot, where the cue ball is placed.
    fn head_spot(&self) -> Vec2 {
        Vec2::new(-self.half_length * 0.5, 0.0)
    }

    /// Foot spot, where the apex ball of the rack sits.
    fn foot_spot(&self) -> Vec2 {
        Vec2::new(self.half_length * 0.5, 0.0)
    }

    fn mouth(&self) -> f32 {
        self.pocket_radius * MOUTH_FACTOR
    }

    /// Is the short (end) cushion at `x = ±half_length` present at this `y`?
    fn end_rail_present(&self, y: f32) -> bool {
        y.abs() < self.half_width - self.mouth()
    }

    /// Is the long (side) cushion at `y = ±half_width` present at this `x`?
    fn side_rail_present(&self, x: f32) -> bool {
        let ax = x.abs();
        ax < self.half_length - self.mouth() && ax > self.mouth()
    }
}

impl Default for Table {
    fn default() -> Self {
        Self::standard()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BallKind {
    Cue,
    Solid,
    Eight,
    Stripe,
}

#[derive(Clone, Debug)]
pub struct Ball {
    /// 0 = cue ball, 1..=15 object balls.
    pub id: u8,
    pub pos: Vec2,
    pub vel: Vec2,
    pub pocketed: bool,
}

impl Ball {
    pub fn kind(&self) -> BallKind {
        match self.id {
            0 => BallKind::Cue,
            8 => BallKind::Eight,
            1..=7 => BallKind::Solid,
            _ => BallKind::Stripe,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum GameEvent {
    /// Two balls collided; `speed` is the closing speed along the normal.
    BallHit { a: u8, b: u8, speed: f32 },
    /// A ball bounced off a cushion; `speed` is the normal impact speed.
    Cushion { ball: u8, speed: f32 },
    /// The cue struck the cue ball (reported on the next `step`).
    Cue { speed: f32 },
    /// A ball dropped into a pocket (also emitted for the cue ball).
    Pocketed { ball: u8 },
    /// The cue ball was pocketed.
    Scratch,
    /// Everything stopped after a shot.
    Settled,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AimPrediction {
    /// Cue-ball centre at first contact, or where it reaches a cushion.
    pub ghost: Vec2,
    /// Id of the first object ball struck, if any.
    pub target: Option<u8>,
    /// Unit direction the object ball will travel.
    pub target_dir: Option<Vec2>,
}

#[derive(Clone, Debug)]
pub struct PoolGame {
    pub table: Table,
    /// Always 16 balls, `balls[i].id == i`.
    pub balls: Vec<Ball>,
    /// Object balls pocketed since the last rack.
    pub potted: u32,
    /// Shots taken since the last rack.
    pub shots: u32,
    accumulator: f32,
    shot_in_progress: bool,
    pending: Vec<GameEvent>,
}

impl Default for PoolGame {
    fn default() -> Self {
        Self::new()
    }
}

impl PoolGame {
    /// A new game, racked and ready to break.
    pub fn new() -> Self {
        let mut game = Self {
            table: Table::standard(),
            balls: (0..16u8)
                .map(|id| Ball {
                    id,
                    pos: Vec2::ZERO,
                    vel: Vec2::ZERO,
                    pocketed: false,
                })
                .collect(),
            potted: 0,
            shots: 0,
            accumulator: 0.0,
            shot_in_progress: false,
            pending: Vec::new(),
        };
        game.rack();
        game
    }

    /// Standard 8-ball rack: apex on the foot spot pointing at the head,
    /// 8 in the centre, one solid and one stripe on the back corners.
    pub fn rack(&mut self) {
        const ORDER: [[u8; 5]; 5] = [
            [1, 0, 0, 0, 0],
            [9, 2, 0, 0, 0],
            [10, 8, 3, 0, 0],
            [11, 4, 12, 5, 0],
            [6, 13, 15, 7, 14],
        ];
        let d = 2.0 * BALL_RADIUS + RACK_GAP;
        let dx = d * 3f32.sqrt() * 0.5;
        let foot = self.table.foot_spot();

        self.balls = (0..16u8)
            .map(|id| Ball {
                id,
                pos: Vec2::ZERO,
                vel: Vec2::ZERO,
                pocketed: false,
            })
            .collect();
        self.balls[0].pos = self.table.head_spot();
        for (row, ids) in ORDER.iter().enumerate() {
            for (j, &id) in ids.iter().take(row + 1).enumerate() {
                let x = foot.x + row as f32 * dx;
                let y = (j as f32 - row as f32 * 0.5) * d;
                self.balls[id as usize].pos = Vec2::new(x, y);
            }
        }
        self.potted = 0;
        self.shots = 0;
        self.accumulator = 0.0;
        self.shot_in_progress = false;
        self.pending.clear();
    }

    pub fn is_moving(&self) -> bool {
        self.balls
            .iter()
            .any(|b| !b.pocketed && b.vel != Vec2::ZERO)
    }

    pub fn can_shoot(&self) -> bool {
        !self.is_moving() && !self.balls[0].pocketed
    }

    /// Strike the cue ball. `power` in 0..=1 maps to 0..5.5 m/s.
    /// Returns false (and does nothing) if a shot is not currently allowed.
    pub fn shoot(&mut self, dir: Vec2, power: f32) -> bool {
        if !self.can_shoot() {
            return false;
        }
        let dir = dir.normalize_or_zero();
        if dir == Vec2::ZERO || !power.is_finite() {
            return false;
        }
        let speed = power.clamp(0.0, 1.0) * MAX_CUE_SPEED;
        self.balls[0].vel = dir * speed;
        self.shots += 1;
        self.shot_in_progress = true;
        self.accumulator = 0.0;
        self.pending.push(GameEvent::Cue { speed });
        true
    }

    /// Advance the simulation by `dt` seconds (capped at 0.1 s per call)
    /// in fixed 1/480 s substeps.
    pub fn step(&mut self, dt: f32) -> Vec<GameEvent> {
        let mut events = std::mem::take(&mut self.pending);
        if dt.is_finite() && dt > 0.0 {
            self.accumulator += dt.min(MAX_FRAME_DT);
        }
        if !self.is_moving() {
            self.accumulator = 0.0;
        }
        while self.accumulator >= SUBSTEP {
            self.accumulator -= SUBSTEP;
            self.substep(SUBSTEP, &mut events);
            if !self.is_moving() {
                self.accumulator = 0.0;
                break;
            }
        }
        if self.shot_in_progress && !self.is_moving() {
            self.shot_in_progress = false;
            if self.balls[0].pocketed {
                self.respot_cue();
            }
            events.push(GameEvent::Settled);
        }
        events
    }

    /// Put the cue ball back on the head spot, nudged along `x` until it
    /// does not overlap any ball.
    pub fn respot_cue(&mut self) {
        let spot = self.table.head_spot();
        let limit = self.table.half_length - BALL_RADIUS;
        let step = 2.0 * BALL_RADIUS + 1e-3;
        let mut chosen = spot;
        'search: for k in 0..200 {
            // 0, -1, +1, -2, +2, ... multiples of `step`.
            let n = (k + 1) / 2;
            let sign = if k % 2 == 1 { -1.0 } else { 1.0 };
            let x = spot.x + sign * n as f32 * step * 0.25;
            if x.abs() > limit {
                continue;
            }
            let candidate = Vec2::new(x, spot.y);
            let clear = self
                .balls
                .iter()
                .skip(1)
                .all(|b| b.pocketed || b.pos.distance(candidate) >= 2.0 * BALL_RADIUS + 1e-4);
            if clear {
                chosen = candidate;
                break 'search;
            }
        }
        let cue = &mut self.balls[0];
        cue.pos = chosen;
        cue.vel = Vec2::ZERO;
        cue.pocketed = false;
    }

    /// Cast the cue ball along `dir` and report what it hits first.
    pub fn predict(&self, dir: Vec2) -> AimPrediction {
        let cue = &self.balls[0];
        let origin = cue.pos;
        let dir = dir.normalize_or_zero();
        if dir == Vec2::ZERO || cue.pocketed {
            return AimPrediction {
                ghost: origin,
                target: None,
                target_dir: None,
            };
        }

        // Distance to the cushion box the cue-ball centre lives in.
        let bx = self.table.half_length - BALL_RADIUS;
        let by = self.table.half_width - BALL_RADIUS;
        let mut t_wall = f32::INFINITY;
        if dir.x > 0.0 {
            t_wall = t_wall.min((bx - origin.x) / dir.x);
        } else if dir.x < 0.0 {
            t_wall = t_wall.min((-bx - origin.x) / dir.x);
        }
        if dir.y > 0.0 {
            t_wall = t_wall.min((by - origin.y) / dir.y);
        } else if dir.y < 0.0 {
            t_wall = t_wall.min((-by - origin.y) / dir.y);
        }
        let t_wall = t_wall.max(0.0);

        let mut best: Option<(f32, u8)> = None;
        let r2 = 2.0 * BALL_RADIUS;
        for b in self.balls.iter().skip(1).filter(|b| !b.pocketed) {
            if let Some(t) = ray_circle(origin, dir, b.pos, r2)
                && t <= t_wall
                && best.is_none_or(|(bt, _)| t < bt)
            {
                best = Some((t, b.id));
            }
        }

        match best {
            Some((t, id)) => {
                let ghost = origin + dir * t;
                let to = (self.balls[id as usize].pos - ghost).normalize_or_zero();
                AimPrediction {
                    ghost,
                    target: Some(id),
                    target_dir: Some(if to == Vec2::ZERO { dir } else { to }),
                }
            }
            None => AimPrediction {
                ghost: origin + dir * t_wall,
                target: None,
                target_dir: None,
            },
        }
    }

    fn substep(&mut self, dt: f32, events: &mut Vec<GameEvent>) {
        // Integrate with constant rolling deceleration.
        for b in self.balls.iter_mut().filter(|b| !b.pocketed) {
            let speed = b.vel.length();
            if speed == 0.0 {
                continue;
            }
            b.pos += b.vel * dt;
            let new_speed = speed - ROLLING_DECEL * dt;
            if new_speed <= SLEEP_SPEED {
                b.vel = Vec2::ZERO;
            } else {
                b.vel *= new_speed / speed;
            }
        }

        self.resolve_ball_collisions(dt, events);
        self.resolve_cushions(events);
        self.check_pockets(events);
    }

    fn resolve_ball_collisions(&mut self, dt: f32, events: &mut Vec<GameEvent>) {
        let r2 = 2.0 * BALL_RADIUS;
        let n = self.balls.len();
        for _iter in 0..6 {
            let mut any = false;
            for i in 0..n {
                if self.balls[i].pocketed {
                    continue;
                }
                for j in (i + 1)..n {
                    if self.balls[j].pocketed {
                        continue;
                    }
                    let (pi, pj) = (self.balls[i].pos, self.balls[j].pos);
                    let delta = pj - pi;
                    let dist2 = delta.length_squared();
                    if dist2 >= r2 * r2 {
                        continue;
                    }
                    any = true;
                    let (vi, vj) = (self.balls[i].vel, self.balls[j].vel);
                    let rel = vi - vj;

                    // Rewind to the moment of contact (swept test) so the
                    // impulse uses the true contact normal.
                    let back = rewind_time(delta, rel, r2).min(dt);
                    let ci = pi - vi * back;
                    let cj = pj - vj * back;
                    let mut normal = (cj - ci).normalize_or_zero();
                    if normal == Vec2::ZERO {
                        normal = (pj - pi).normalize_or_zero();
                    }
                    if normal == Vec2::ZERO {
                        normal = Vec2::X;
                    }
                    let closing = rel.dot(normal);
                    if closing > 0.0 {
                        let imp = 0.5 * (1.0 + BALL_RESTITUTION) * closing;
                        let nvi = vi - normal * imp;
                        let nvj = vj + normal * imp;
                        self.balls[i].vel = nvi;
                        self.balls[j].vel = nvj;
                        self.balls[i].pos = ci + nvi * back;
                        self.balls[j].pos = cj + nvj * back;
                        if closing >= EVENT_MIN_SPEED {
                            events.push(GameEvent::BallHit {
                                a: self.balls[i].id,
                                b: self.balls[j].id,
                                speed: closing,
                            });
                        }
                    }

                    // Positional de-penetration.
                    let delta = self.balls[j].pos - self.balls[i].pos;
                    let dist = delta.length();
                    if dist < r2 {
                        let n = if dist > 1e-9 { delta / dist } else { normal };
                        let push = (r2 - dist) * 0.5 + 1e-7;
                        self.balls[i].pos -= n * push;
                        self.balls[j].pos += n * push;
                    }
                }
            }
            if !any {
                break;
            }
        }
    }

    fn resolve_cushions(&mut self, events: &mut Vec<GameEvent>) {
        let t = &self.table;
        let bx = t.half_length - BALL_RADIUS;
        let by = t.half_width - BALL_RADIUS;
        for b in self.balls.iter_mut().filter(|b| !b.pocketed) {
            if b.pos.x.abs() > bx && t.end_rail_present(b.pos.y) {
                let s = b.pos.x.signum();
                b.pos.x = s * bx;
                let vn = b.vel.x * s;
                if vn > 0.0 {
                    b.vel.x = -b.vel.x * CUSHION_RESTITUTION;
                    b.vel.y *= CUSHION_TANGENT_KEEP;
                    if vn >= EVENT_MIN_SPEED {
                        events.push(GameEvent::Cushion {
                            ball: b.id,
                            speed: vn,
                        });
                    }
                }
            }
            if b.pos.y.abs() > by && t.side_rail_present(b.pos.x) {
                let s = b.pos.y.signum();
                b.pos.y = s * by;
                let vn = b.vel.y * s;
                if vn > 0.0 {
                    b.vel.y = -b.vel.y * CUSHION_RESTITUTION;
                    b.vel.x *= CUSHION_TANGENT_KEEP;
                    if vn >= EVENT_MIN_SPEED {
                        events.push(GameEvent::Cushion {
                            ball: b.id,
                            speed: vn,
                        });
                    }
                }
            }
        }
    }

    fn check_pockets(&mut self, events: &mut Vec<GameEvent>) {
        let t = &self.table;
        let pr2 = t.pocket_radius * t.pocket_radius;
        let out_x = t.half_length + BALL_RADIUS;
        let out_y = t.half_width + BALL_RADIUS;
        for b in self.balls.iter_mut().filter(|b| !b.pocketed) {
            let in_pocket = t.pockets.iter().any(|p| p.distance_squared(b.pos) <= pr2);
            // Safety net: anything that got past the rail line is in a pocket.
            let escaped = b.pos.x.abs() > out_x || b.pos.y.abs() > out_y;
            if in_pocket || escaped {
                b.pocketed = true;
                b.vel = Vec2::ZERO;
                events.push(GameEvent::Pocketed { ball: b.id });
                if b.id == 0 {
                    events.push(GameEvent::Scratch);
                } else {
                    self.potted += 1;
                }
            }
        }
    }
}

/// Time to rewind two overlapping balls (relative position `delta = pj - pi`,
/// relative velocity `rel = vi - vj`) until their centres are `r` apart.
fn rewind_time(delta: Vec2, rel: Vec2, r: f32) -> f32 {
    // |delta + rel * s| = r, smallest s >= 0 (moving backwards in time,
    // pj - pi grows by (vi - vj) * s).
    let a = rel.length_squared();
    if a < 1e-12 {
        return 0.0;
    }
    let b = 2.0 * delta.dot(rel);
    let c = delta.length_squared() - r * r;
    let disc = b * b - 4.0 * a * c;
    if disc < 0.0 {
        return 0.0;
    }
    let s = (-b + disc.sqrt()) / (2.0 * a);
    s.max(0.0)
}

/// Smallest `t >= 0` with `|origin + dir * t - centre| = r`, for unit `dir`.
fn ray_circle(origin: Vec2, dir: Vec2, centre: Vec2, r: f32) -> Option<f32> {
    let m = origin - centre;
    let b = m.dot(dir);
    let c = m.length_squared() - r * r;
    if c <= 0.0 {
        // Already touching: only counts if moving towards it.
        return (b < 0.0).then_some(0.0);
    }
    if b > 0.0 {
        return None;
    }
    let disc = b * b - c;
    if disc < 0.0 {
        return None;
    }
    Some(-b - disc.sqrt())
}

#[cfg(test)]
mod tests;
