//! Owned, serializable mirrors of the content types, for the parts that live
//! in the database (projects, experience, skills) and travel as JSON.
//!
//! The pages read `&'static` [`Site`] data. A [`Catalog`] loaded from the
//! database becomes one with [`Catalog::leak`]: a deliberate, bounded leak
//! (one snapshot per content edit on the server, one per page load in the
//! browser) that keeps every page reading plain static data.

use serde::{Deserialize, Serialize};

use crate as c;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Image {
    pub src: String,
    pub width: u32,
    pub height: u32,
    pub alt: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Media {
    pub video: bool,
    pub src: String,
    pub width: u32,
    pub height: u32,
    pub alt: String,
    pub caption: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Link {
    pub label: String,
    pub url: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Credit {
    pub role: String,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Project {
    pub slug: String,
    pub title: String,
    pub year: u16,
    pub client: String,
    pub role: String,
    pub categories: Vec<String>,
    pub tools: Vec<String>,
    pub featured: bool,
    pub accent: [u8; 3],
    pub cover: Image,
    pub cover_asset: String,
    pub preview_video: Option<String>,
    pub summary: String,
    /// The Markdown source (what the dashboard edits).
    pub body_md: String,
    /// Rendered from `body_md` when saved.
    pub body_html: String,
    pub gallery: Vec<Media>,
    pub links: Vec<Link>,
    pub credits: Vec<Credit>,
    pub example: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Role {
    pub title: String,
    pub org: String,
    /// `job`, `freelance`, `education` or `milestone`.
    pub kind: String,
    pub start: String,
    pub end: String,
    pub location: String,
    pub summary: String,
    pub placeholder: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    /// `daily`, `confident` or `learning`.
    pub level: String,
    pub note: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SkillGroup {
    pub name: String,
    pub blurb: String,
    pub skills: Vec<Skill>,
    pub placeholder: bool,
}

/// Everything the dashboard edits.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Catalog {
    /// In display order (newest first).
    pub projects: Vec<Project>,
    pub roles: Vec<Role>,
    pub cv: Option<String>,
    pub skills: Vec<SkillGroup>,
}

pub fn role_kind(s: &str) -> c::RoleKind {
    match s {
        "job" => c::RoleKind::Job,
        "freelance" => c::RoleKind::Freelance,
        "education" => c::RoleKind::Education,
        _ => c::RoleKind::Milestone,
    }
}

pub fn role_kind_id(k: c::RoleKind) -> &'static str {
    match k {
        c::RoleKind::Job => "job",
        c::RoleKind::Freelance => "freelance",
        c::RoleKind::Education => "education",
        c::RoleKind::Milestone => "milestone",
    }
}

pub fn level(s: &str) -> c::Level {
    match s {
        "daily" => c::Level::Daily,
        "confident" => c::Level::Confident,
        _ => c::Level::Learning,
    }
}

pub fn level_id(l: c::Level) -> &'static str {
    match l {
        c::Level::Daily => "daily",
        c::Level::Confident => "confident",
        c::Level::Learning => "learning",
    }
}

fn leak_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

fn leak_strs(v: Vec<String>) -> &'static [&'static str] {
    Box::leak(
        v.into_iter()
            .map(leak_str)
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    )
}

fn leak_slice<T>(v: Vec<T>) -> &'static [T] {
    Box::leak(v.into_boxed_slice())
}

impl Image {
    fn leak(self) -> c::Image {
        c::Image {
            src: leak_str(self.src),
            width: self.width,
            height: self.height,
            alt: leak_str(self.alt),
        }
    }

    fn from_static(i: &c::Image) -> Self {
        Self {
            src: i.src.into(),
            width: i.width,
            height: i.height,
            alt: i.alt.into(),
        }
    }
}

impl Project {
    fn leak(self) -> c::Project {
        c::Project {
            slug: leak_str(self.slug),
            title: leak_str(self.title),
            year: self.year,
            client: leak_str(self.client),
            role: leak_str(self.role),
            categories: leak_strs(self.categories),
            tools: leak_strs(self.tools),
            featured: self.featured,
            accent: self.accent,
            cover: self.cover.leak(),
            cover_asset: leak_str(self.cover_asset),
            preview_video: self.preview_video.map(leak_str),
            summary: leak_str(self.summary),
            body_html: leak_str(self.body_html),
            gallery: leak_slice(
                self.gallery
                    .into_iter()
                    .map(|m| c::Media {
                        kind: if m.video {
                            c::MediaKind::Video
                        } else {
                            c::MediaKind::Image
                        },
                        src: leak_str(m.src),
                        width: m.width,
                        height: m.height,
                        alt: leak_str(m.alt),
                        caption: leak_str(m.caption),
                    })
                    .collect(),
            ),
            links: leak_slice(
                self.links
                    .into_iter()
                    .map(|l| c::Link {
                        label: leak_str(l.label),
                        url: leak_str(l.url),
                    })
                    .collect(),
            ),
            credits: leak_slice(
                self.credits
                    .into_iter()
                    .map(|x| c::Credit {
                        role: leak_str(x.role),
                        name: leak_str(x.name),
                    })
                    .collect(),
            ),
            example: self.example,
        }
    }

    /// From compiled content. The Markdown source isn't compiled in, so the
    /// body arrives as HTML only (the dashboard edits it as HTML-in-Markdown).
    pub fn from_static(p: &c::Project) -> Self {
        Self {
            slug: p.slug.into(),
            title: p.title.into(),
            year: p.year,
            client: p.client.into(),
            role: p.role.into(),
            categories: p.categories.iter().map(|s| s.to_string()).collect(),
            tools: p.tools.iter().map(|s| s.to_string()).collect(),
            featured: p.featured,
            accent: p.accent,
            cover: Image::from_static(&p.cover),
            cover_asset: p.cover_asset.into(),
            preview_video: p.preview_video.map(Into::into),
            summary: p.summary.into(),
            body_md: p.body_html.into(),
            body_html: p.body_html.into(),
            gallery: p
                .gallery
                .iter()
                .map(|m| Media {
                    video: m.kind == c::MediaKind::Video,
                    src: m.src.into(),
                    width: m.width,
                    height: m.height,
                    alt: m.alt.into(),
                    caption: m.caption.into(),
                })
                .collect(),
            links: p
                .links
                .iter()
                .map(|l| Link {
                    label: l.label.into(),
                    url: l.url.into(),
                })
                .collect(),
            credits: p
                .credits
                .iter()
                .map(|x| Credit {
                    role: x.role.into(),
                    name: x.name.into(),
                })
                .collect(),
            example: p.example,
        }
    }
}

impl Catalog {
    /// What's compiled in from `content/` (seeds an empty database).
    pub fn from_static(s: &c::Site) -> Self {
        Self {
            projects: s.projects.iter().map(Project::from_static).collect(),
            roles: s
                .experience
                .roles
                .iter()
                .map(|r| Role {
                    title: r.title.into(),
                    org: r.org.into(),
                    kind: role_kind_id(r.kind).into(),
                    start: r.start.into(),
                    end: r.end.into(),
                    location: r.location.into(),
                    summary: r.summary.into(),
                    placeholder: r.placeholder,
                })
                .collect(),
            cv: s.experience.cv.map(Into::into),
            skills: s
                .skills
                .iter()
                .map(|g| SkillGroup {
                    name: g.name.into(),
                    blurb: g.blurb.into(),
                    skills: g
                        .skills
                        .iter()
                        .map(|k| Skill {
                            name: k.name.into(),
                            level: level_id(k.level).into(),
                            note: k.note.into(),
                        })
                        .collect(),
                    placeholder: g.placeholder,
                })
                .collect(),
        }
    }

    /// A `'static` site from this catalog plus the compiled About page.
    pub fn leak(self, about: c::About) -> &'static c::Site {
        let site = c::Site {
            projects: leak_slice(self.projects.into_iter().map(Project::leak).collect()),
            experience: c::Experience {
                roles: leak_slice(
                    self.roles
                        .into_iter()
                        .map(|r| c::Role {
                            kind: role_kind(&r.kind),
                            title: leak_str(r.title),
                            org: leak_str(r.org),
                            start: leak_str(r.start),
                            end: leak_str(r.end),
                            location: leak_str(r.location),
                            summary: leak_str(r.summary),
                            placeholder: r.placeholder,
                        })
                        .collect(),
                ),
                cv: self.cv.map(leak_str),
            },
            skills: leak_slice(
                self.skills
                    .into_iter()
                    .map(|g| c::SkillGroup {
                        name: leak_str(g.name),
                        blurb: leak_str(g.blurb),
                        skills: leak_slice(
                            g.skills
                                .into_iter()
                                .map(|k| c::Skill {
                                    level: level(&k.level),
                                    name: leak_str(k.name),
                                    note: leak_str(k.note),
                                })
                                .collect(),
                        ),
                        placeholder: g.placeholder,
                    })
                    .collect(),
            ),
            about,
        };
        Box::leak(Box::new(site))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_through_static() {
        let mut cat = Catalog::default();
        cat.roles.push(Role {
            title: "Engineer".into(),
            org: "Acme".into(),
            kind: "freelance".into(),
            start: "2024".into(),
            end: String::new(),
            location: "Bloemfontein".into(),
            summary: "Things.".into(),
            placeholder: false,
        });
        cat.skills.push(SkillGroup {
            name: "Rust".into(),
            blurb: String::new(),
            skills: vec![Skill {
                name: "Leptos".into(),
                level: "daily".into(),
                note: String::new(),
            }],
            placeholder: false,
        });
        let about = c::About {
            name: "",
            role: "",
            location: "",
            intro: "",
            portrait: None,
            body_html: "",
            stats: &[],
            currently: &[],
            placeholder: true,
            email: "",
            reply_time: "",
        };
        let site = cat.clone().leak(about);
        assert_eq!(site.experience.roles[0].kind, c::RoleKind::Freelance);
        assert_eq!(site.skills[0].skills[0].level, c::Level::Daily);
        assert_eq!(Catalog::from_static(site), cat);
    }
}
