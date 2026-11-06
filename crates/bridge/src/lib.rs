//! The contract between the Leptos site and the Bevy game room.
//!
//! The two are separate WASM modules that share nothing but the page, so they
//! talk through `CustomEvent`s on `window`:
//!
//! * room → site: [`ROOM_EVENT`], detail `{ type, ...fields }` ([`RoomEvent`])
//! * site → room: [`COMMAND`], detail `{ type, value }` ([`Command`])
//!
//! Every payload is a flat object of strings, so neither side needs serde and
//! the wire format is readable in devtools. This crate owns the encoding both
//! ways; the `web` module (wasm only) does the actual dispatching.

pub const ROOM_EVENT: &str = "arthub:room";
pub const COMMAND: &str = "arthub:command";
/// DOM id of the canvas Bevy renders into.
pub const CANVAS_ID: &str = "game-room-canvas";

#[cfg(target_arch = "wasm32")]
pub mod web;

// ---------------------------------------------------------------------------
// Hotspots
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum Hotspot {
    #[default]
    Overview,
    PoolTable,
    ArcadeRow,
    TrophyShelf,
    Chalkboard,
    Jukebox,
    Tv,
    Bar,
    Exit,
}

impl Hotspot {
    pub const ALL: [Hotspot; 9] = [
        Hotspot::Overview,
        Hotspot::PoolTable,
        Hotspot::ArcadeRow,
        Hotspot::TrophyShelf,
        Hotspot::Chalkboard,
        Hotspot::Jukebox,
        Hotspot::Tv,
        Hotspot::Bar,
        Hotspot::Exit,
    ];

    pub fn id(self) -> &'static str {
        match self {
            Hotspot::Overview => "overview",
            Hotspot::PoolTable => "pool",
            Hotspot::ArcadeRow => "arcade",
            Hotspot::TrophyShelf => "trophies",
            Hotspot::Chalkboard => "chalkboard",
            Hotspot::Jukebox => "jukebox",
            Hotspot::Tv => "tv",
            Hotspot::Bar => "bar",
            Hotspot::Exit => "exit",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Hotspot::Overview => "Overview",
            Hotspot::PoolTable => "Pool Table",
            Hotspot::ArcadeRow => "Arcade Row",
            Hotspot::TrophyShelf => "Trophy Shelf",
            Hotspot::Chalkboard => "Chalkboard",
            Hotspot::Jukebox => "Jukebox",
            Hotspot::Tv => "TV",
            Hotspot::Bar => "Bar & Phone",
            Hotspot::Exit => "Exit",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|h| h.id() == id)
    }

    /// Next / previous hotspot in tour order (swipe and arrow keys).
    pub fn cycle(self, forward: bool) -> Self {
        let i = Self::ALL.iter().position(|h| *h == self).unwrap_or(0) as isize;
        let n = Self::ALL.len() as isize;
        Self::ALL[(i + if forward { 1 } else { -1 }).rem_euclid(n) as usize]
    }
}

// ---------------------------------------------------------------------------
// Settings shared by both sides
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Quality {
    Low,
    #[default]
    Medium,
    High,
}

impl Quality {
    pub const ALL: [Quality; 3] = [Quality::Low, Quality::Medium, Quality::High];

    pub fn id(self) -> &'static str {
        match self {
            Quality::Low => "low",
            Quality::Medium => "medium",
            Quality::High => "high",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|q| q.id() == id)
    }

    pub fn lower(self) -> Option<Self> {
        match self {
            Quality::High => Some(Quality::Medium),
            Quality::Medium => Some(Quality::Low),
            Quality::Low => None,
        }
    }
}

/// `Preview` is the hero-section teaser: drift only, no input, no pool.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Mode {
    Preview,
    #[default]
    Full,
}

impl Mode {
    pub fn id(self) -> &'static str {
        match self {
            Mode::Preview => "preview",
            Mode::Full => "full",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "preview" => Some(Mode::Preview),
            "full" => Some(Mode::Full),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Site → room
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    FocusHotspot(Hotspot),
    Pause,
    Resume,
    SetQuality(Quality),
    /// Master mute for music and sound effects.
    Mute(bool),
    SetMode(Mode),
    PoolReset,
    PoolExit,
    /// Return the camera to the current hotspot's view.
    ResetView,
}

impl Command {
    pub fn encode(&self) -> (&'static str, String) {
        match self {
            Command::FocusHotspot(h) => ("focus_hotspot", h.id().into()),
            Command::Pause => ("pause", String::new()),
            Command::Resume => ("resume", String::new()),
            Command::SetQuality(q) => ("set_quality", q.id().into()),
            Command::Mute(m) => ("mute", m.to_string()),
            Command::SetMode(m) => ("set_mode", m.id().into()),
            Command::PoolReset => ("pool_reset", String::new()),
            Command::PoolExit => ("pool_exit", String::new()),
            Command::ResetView => ("reset_view", String::new()),
        }
    }

    pub fn decode(kind: &str, value: &str) -> Option<Self> {
        Some(match kind {
            "focus_hotspot" => Command::FocusHotspot(Hotspot::from_id(value)?),
            "pause" => Command::Pause,
            "resume" => Command::Resume,
            "set_quality" => Command::SetQuality(Quality::from_id(value)?),
            "mute" => Command::Mute(value == "true"),
            "set_mode" => Command::SetMode(Mode::from_id(value)?),
            "pool_reset" => Command::PoolReset,
            "pool_exit" => Command::PoolExit,
            "reset_view" => Command::ResetView,
            _ => return None,
        })
    }
}

// ---------------------------------------------------------------------------
// Room → site
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub enum RoomEvent {
    /// The room finished building its scene and is rendering.
    Ready,
    /// Open a site route, e.g. `/work/neon-city`.
    Navigate {
        path: String,
    },
    /// The camera settled on a hotspot.
    Hotspot(Hotspot),
    /// Pointer is over an interactive object (`None` = left it).
    Hover {
        label: Option<String>,
    },
    ShowreelOpen,
    Music {
        playing: bool,
    },
    /// The user walked out the door.
    Exit,
    PoolActive(bool),
    PoolScore {
        potted: u32,
        shots: u32,
    },
    /// Aim power while dragging a shot, 0..=1.
    PoolPower(f32),
    /// The room changed tier on its own (frame-time auto-drop).
    Quality(Quality),
    /// First time the visitor moved the camera (retire the controls hint).
    Interacted,
}

impl RoomEvent {
    pub fn encode(&self) -> (&'static str, Vec<(&'static str, String)>) {
        match self {
            RoomEvent::Ready => ("ready", vec![]),
            RoomEvent::Navigate { path } => ("navigate", vec![("path", path.clone())]),
            RoomEvent::Hotspot(h) => ("hotspot", vec![("id", h.id().into())]),
            RoomEvent::Hover { label } => (
                "hover",
                label.iter().map(|l| ("label", l.clone())).collect(),
            ),
            RoomEvent::ShowreelOpen => ("showreel", vec![]),
            RoomEvent::Music { playing } => ("music", vec![("playing", playing.to_string())]),
            RoomEvent::Exit => ("exit", vec![]),
            RoomEvent::PoolActive(a) => ("pool_active", vec![("active", a.to_string())]),
            RoomEvent::PoolScore { potted, shots } => (
                "pool_score",
                vec![("potted", potted.to_string()), ("shots", shots.to_string())],
            ),
            RoomEvent::PoolPower(p) => ("pool_power", vec![("power", format!("{p:.3}"))]),
            RoomEvent::Quality(q) => ("quality", vec![("tier", q.id().into())]),
            RoomEvent::Interacted => ("interacted", vec![]),
        }
    }

    /// `get` looks up a field of the event detail by name.
    pub fn decode(kind: &str, get: impl Fn(&str) -> Option<String>) -> Option<Self> {
        Some(match kind {
            "ready" => RoomEvent::Ready,
            "navigate" => RoomEvent::Navigate { path: get("path")? },
            "hotspot" => RoomEvent::Hotspot(Hotspot::from_id(&get("id")?)?),
            "hover" => RoomEvent::Hover {
                label: get("label"),
            },
            "showreel" => RoomEvent::ShowreelOpen,
            "music" => RoomEvent::Music {
                playing: get("playing")? == "true",
            },
            "exit" => RoomEvent::Exit,
            "pool_active" => RoomEvent::PoolActive(get("active")? == "true"),
            "pool_score" => RoomEvent::PoolScore {
                potted: get("potted")?.parse().ok()?,
                shots: get("shots")?.parse().ok()?,
            },
            "pool_power" => RoomEvent::PoolPower(get("power")?.parse().ok()?),
            "quality" => RoomEvent::Quality(Quality::from_id(&get("tier")?)?),
            "interacted" => RoomEvent::Interacted,
            _ => return None,
        })
    }
}

// ---------------------------------------------------------------------------
// Featured projects (one arcade cabinet each)
// ---------------------------------------------------------------------------

/// A featured project as the room needs it. The site builds this list from
/// `content/` and hands it to the room when starting it, so new featured
/// projects appear as cabinets without rebuilding the room.
#[derive(Clone, Debug, PartialEq)]
pub struct FeaturedProject {
    pub slug: String,
    pub title: String,
    /// Cabinet glow colour (sRGB).
    pub color: [u8; 3],
    /// Screen image, relative to the asset root (e.g. `media/work/x/cover.jpg`).
    pub cover: Option<String>,
}

/// One project per line, tab-separated: `slug, title, rrggbb, cover`.
pub fn encode_featured(list: &[FeaturedProject]) -> String {
    let clean = |s: &str| s.replace(['\t', '\n', '\r'], " ");
    list.iter()
        .map(|p| {
            let [r, g, b] = p.color;
            format!(
                "{}\t{}\t{r:02x}{g:02x}{b:02x}\t{}",
                clean(&p.slug),
                clean(&p.title),
                clean(p.cover.as_deref().unwrap_or(""))
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn decode_featured(s: &str) -> Vec<FeaturedProject> {
    s.lines()
        .filter_map(|line| {
            let mut f = line.split('\t');
            let (slug, title, hex, cover) =
                (f.next()?, f.next()?, f.next()?, f.next().unwrap_or(""));
            let v = u32::from_str_radix(hex, 16).ok()?;
            Some(FeaturedProject {
                slug: slug.into(),
                title: title.into(),
                color: [(v >> 16) as u8, (v >> 8) as u8, v as u8],
                cover: (!cover.is_empty()).then(|| cover.into()),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commands_round_trip() {
        let all = [
            Command::FocusHotspot(Hotspot::Jukebox),
            Command::Pause,
            Command::Resume,
            Command::SetQuality(Quality::Low),
            Command::Mute(true),
            Command::Mute(false),
            Command::SetMode(Mode::Preview),
            Command::PoolReset,
            Command::PoolExit,
            Command::ResetView,
        ];
        for c in all {
            let (k, v) = c.encode();
            assert_eq!(Command::decode(k, &v), Some(c));
        }
    }

    #[test]
    fn events_round_trip() {
        let all = [
            RoomEvent::Ready,
            RoomEvent::Navigate {
                path: "/work/neon-city".into(),
            },
            RoomEvent::Hotspot(Hotspot::Bar),
            RoomEvent::Hover {
                label: Some("Jukebox".into()),
            },
            RoomEvent::Hover { label: None },
            RoomEvent::ShowreelOpen,
            RoomEvent::Music { playing: true },
            RoomEvent::Exit,
            RoomEvent::PoolActive(true),
            RoomEvent::PoolScore {
                potted: 3,
                shots: 5,
            },
            RoomEvent::PoolPower(0.5),
            RoomEvent::Quality(Quality::High),
            RoomEvent::Interacted,
        ];
        for e in all {
            let (k, fields) = e.encode();
            let get = |n: &str| fields.iter().find(|(f, _)| *f == n).map(|(_, v)| v.clone());
            assert_eq!(RoomEvent::decode(k, get), Some(e));
        }
    }

    #[test]
    fn featured_round_trip() {
        let list = vec![
            FeaturedProject {
                slug: "a".into(),
                title: "Tabs\tand\nlines".into(),
                color: [1, 2, 255],
                cover: Some("media/a.jpg".into()),
            },
            FeaturedProject {
                slug: "b".into(),
                title: "B".into(),
                color: [0, 0, 0],
                cover: None,
            },
        ];
        let back = decode_featured(&encode_featured(&list));
        assert_eq!(back[0].title, "Tabs and lines");
        assert_eq!(back[0].color, [1, 2, 255]);
        assert_eq!(back[0].cover.as_deref(), Some("media/a.jpg"));
        assert_eq!(back[1].cover, None);
    }

    #[test]
    fn hotspot_cycle_wraps() {
        assert_eq!(Hotspot::Overview.cycle(false), Hotspot::Exit);
        assert_eq!(Hotspot::Exit.cycle(true), Hotspot::Overview);
        for h in Hotspot::ALL {
            assert_eq!(Hotspot::from_id(h.id()), Some(h));
        }
    }
}
