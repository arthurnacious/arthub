//! Portfolio content.
//!
//! The site never parses anything at runtime. The site's build script calls
//! [`load::load`] (feature `parse`) to read and validate `content/`, then
//! [`codegen::render`] writes it out as the `'static` data below, which gets
//! compiled straight into the server and the browser bundle.

#[cfg(feature = "parse")]
pub mod codegen;
#[cfg(feature = "parse")]
pub mod load;
pub mod owned;

/// An image with its pixel size, so pages can reserve space before it loads.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Image {
    /// Site URL, e.g. `/media/work/neon-city/cover.jpg`.
    pub src: &'static str,
    pub width: u32,
    pub height: u32,
    pub alt: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MediaKind {
    Image,
    Video,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Media {
    pub kind: MediaKind,
    pub src: &'static str,
    /// Pixel size (0 for videos; they're laid out by aspect ratio instead).
    pub width: u32,
    pub height: u32,
    pub alt: &'static str,
    pub caption: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Link {
    pub label: &'static str,
    pub url: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Credit {
    pub role: &'static str,
    pub name: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Project {
    pub slug: &'static str,
    pub title: &'static str,
    pub year: u16,
    pub client: &'static str,
    pub role: &'static str,
    pub categories: &'static [&'static str],
    pub tools: &'static [&'static str],
    /// Featured projects appear on the home page and as arcade cabinets.
    pub featured: bool,
    /// sRGB accent used for the cabinet's glow.
    pub accent: [u8; 3],
    pub cover: Image,
    /// Asset path of the cover for the game room (`media/work/…`).
    pub cover_asset: &'static str,
    pub preview_video: Option<&'static str>,
    pub summary: &'static str,
    /// The Markdown body, rendered to HTML at build time.
    pub body_html: &'static str,
    pub gallery: &'static [Media],
    pub links: &'static [Link],
    pub credits: &'static [Credit],
    /// Shown with an "Example" badge until replaced with real work.
    pub example: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoleKind {
    Job,
    Freelance,
    Education,
    Milestone,
}

impl RoleKind {
    pub fn label(self) -> &'static str {
        match self {
            RoleKind::Job => "Role",
            RoleKind::Freelance => "Freelance",
            RoleKind::Education => "Education",
            RoleKind::Milestone => "Milestone",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Role {
    pub title: &'static str,
    pub org: &'static str,
    pub kind: RoleKind,
    /// Free-form, e.g. `2023` or `Mar 2023`.
    pub start: &'static str,
    /// Empty means ongoing ("Now").
    pub end: &'static str,
    pub location: &'static str,
    pub summary: &'static str,
    pub placeholder: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Experience {
    pub roles: &'static [Role],
    /// URL of a downloadable CV, if one is provided.
    pub cv: Option<&'static str>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Level {
    /// Use it every day; would teach it.
    Daily,
    /// Ship real work with it.
    Confident,
    /// Learning or occasional.
    Learning,
}

impl Level {
    pub fn label(self) -> &'static str {
        match self {
            Level::Daily => "Daily driver",
            Level::Confident => "Confident",
            Level::Learning => "Learning",
        }
    }

    /// Chalk tallies on the skills board.
    pub fn tallies(self) -> u8 {
        match self {
            Level::Daily => 5,
            Level::Confident => 3,
            Level::Learning => 1,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Skill {
    pub name: &'static str,
    pub level: Level,
    pub note: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SkillGroup {
    pub name: &'static str,
    pub blurb: &'static str,
    pub skills: &'static [Skill],
    pub placeholder: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stat {
    pub value: &'static str,
    pub label: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Currently {
    pub label: &'static str,
    pub text: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct About {
    pub name: &'static str,
    pub role: &'static str,
    pub location: &'static str,
    pub intro: &'static str,
    pub portrait: Option<Image>,
    pub body_html: &'static str,
    pub stats: &'static [Stat],
    pub currently: &'static [Currently],
    pub placeholder: bool,
    /// Direct email, shown on the contact page and as the fallback if the form fails.
    pub email: &'static str,
    /// How soon you reply, e.g. "two working days".
    pub reply_time: &'static str,
}

/// Everything in `content/`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Site {
    /// Newest first.
    pub projects: &'static [Project],
    pub experience: Experience,
    pub skills: &'static [SkillGroup],
    pub about: About,
}

impl Site {
    pub fn project(&self, slug: &str) -> Option<&'static Project> {
        self.projects.iter().find(|p| p.slug == slug)
    }

    pub fn featured(&self) -> impl Iterator<Item = &'static Project> {
        self.projects.iter().filter(|p| p.featured)
    }

    /// Every category used, in first-seen order.
    pub fn categories(&self) -> Vec<&'static str> {
        let mut out: Vec<&'static str> = Vec::new();
        for p in self.projects {
            for c in p.categories {
                if !out.contains(c) {
                    out.push(c);
                }
            }
        }
        out
    }

    /// The project after `slug`, wrapping round (for "Next project").
    pub fn next_after(&self, slug: &str) -> Option<&'static Project> {
        let i = self.projects.iter().position(|p| p.slug == slug)?;
        self.projects.get((i + 1) % self.projects.len())
    }
}
