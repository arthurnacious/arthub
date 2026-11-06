//! Read, validate and render `content/`. Every problem is collected, so a
//! broken build lists all of them at once, each with its file.
//!
//! Layout:
//!
//! ```text
//! content/projects/<slug>.md     one project: TOML front matter (+++) + Markdown
//! content/about.md               about page: TOML front matter + Markdown bio
//! content/experience.toml        timeline + optional CV
//! content/skills.toml            skill groups
//! assets/media/work/<slug>/      that project's images and videos
//! assets/media/about/            portrait
//! assets/media/                  CV (and anything else shared)
//! ```

use std::collections::HashSet;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use pulldown_cmark::{Options, Parser, html};
use serde::Deserialize;

/// Where content and the served assets live.
#[derive(Clone, Debug)]
pub struct Paths {
    /// Workspace root; error messages show paths relative to it.
    pub root: PathBuf,
    pub content: PathBuf,
    pub assets: PathBuf,
}

impl Paths {
    /// From a crate inside `crates/`, the workspace's `content/` and `assets/`.
    pub fn from_crate_dir(manifest_dir: &str) -> Self {
        let root = Path::new(manifest_dir).join("../..");
        let root = root.canonicalize().unwrap_or(root);
        Self {
            content: root.join("content"),
            assets: root.join("assets"),
            root,
        }
    }
}

// ---------------------------------------------------------------------------
// Owned model (what the loader produces)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct ImageData {
    pub src: String,
    pub width: u32,
    pub height: u32,
    pub alt: String,
}

#[derive(Clone, Debug)]
pub struct MediaData {
    pub video: bool,
    pub src: String,
    pub width: u32,
    pub height: u32,
    pub alt: String,
    pub caption: String,
}

#[derive(Clone, Debug)]
pub struct ProjectData {
    pub slug: String,
    pub title: String,
    pub year: u16,
    pub client: String,
    pub role: String,
    pub categories: Vec<String>,
    pub tools: Vec<String>,
    pub featured: bool,
    pub accent: [u8; 3],
    pub cover: ImageData,
    pub cover_asset: String,
    pub preview_video: Option<String>,
    pub summary: String,
    pub body_html: String,
    pub gallery: Vec<MediaData>,
    pub links: Vec<(String, String)>,
    pub credits: Vec<(String, String)>,
    pub example: bool,
    order: i32,
}

#[derive(Clone, Debug)]
pub struct RoleData {
    pub title: String,
    pub org: String,
    pub kind: String,
    pub start: String,
    pub end: String,
    pub location: String,
    pub summary: String,
    pub placeholder: bool,
}

#[derive(Clone, Debug)]
pub struct SkillGroupData {
    pub name: String,
    pub blurb: String,
    pub skills: Vec<(String, String, String)>,
    pub placeholder: bool,
}

#[derive(Clone, Debug)]
pub struct AboutData {
    pub name: String,
    pub role: String,
    pub location: String,
    pub intro: String,
    pub portrait: Option<ImageData>,
    pub body_html: String,
    pub stats: Vec<(String, String)>,
    pub currently: Vec<(String, String)>,
    pub placeholder: bool,
    pub email: String,
    pub reply_time: String,
}

#[derive(Clone, Debug)]
pub struct SiteData {
    pub projects: Vec<ProjectData>,
    pub roles: Vec<RoleData>,
    pub cv: Option<String>,
    pub skills: Vec<SkillGroupData>,
    pub about: AboutData,
}

// ---------------------------------------------------------------------------
// File formats
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProjectFront {
    title: String,
    slug: String,
    year: u16,
    client: String,
    role: String,
    categories: Vec<String>,
    #[serde(default)]
    tools: Vec<String>,
    #[serde(default)]
    featured: bool,
    accent: Option<String>,
    cover_image: String,
    cover_alt: Option<String>,
    preview_video: Option<String>,
    summary: String,
    #[serde(default)]
    gallery: Vec<GalleryFront>,
    #[serde(default)]
    links: Vec<LinkFront>,
    #[serde(default)]
    credits: Vec<CreditFront>,
    /// Lower sorts first within a year; defaults to 0.
    #[serde(default)]
    order: i32,
    #[serde(default)]
    example: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GalleryFront {
    src: String,
    alt: String,
    #[serde(default)]
    caption: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LinkFront {
    label: String,
    url: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CreditFront {
    role: String,
    name: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ExperienceFile {
    cv: Option<String>,
    #[serde(default)]
    role: Vec<RoleFront>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RoleFront {
    title: String,
    org: String,
    kind: String,
    start: String,
    #[serde(default)]
    end: String,
    #[serde(default)]
    location: String,
    summary: String,
    #[serde(default)]
    placeholder: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SkillsFile {
    #[serde(default)]
    group: Vec<GroupFront>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GroupFront {
    name: String,
    #[serde(default)]
    blurb: String,
    #[serde(default)]
    placeholder: bool,
    skills: Vec<SkillFront>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SkillFront {
    name: String,
    level: String,
    #[serde(default)]
    note: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AboutFront {
    name: String,
    role: String,
    #[serde(default)]
    location: String,
    intro: String,
    portrait: Option<String>,
    portrait_alt: Option<String>,
    #[serde(default)]
    stats: Vec<StatFront>,
    #[serde(default)]
    currently: Vec<CurrentlyFront>,
    #[serde(default)]
    placeholder: bool,
    email: String,
    #[serde(default = "default_reply_time")]
    reply_time: String,
}

fn default_reply_time() -> String {
    "a couple of days".into()
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StatFront {
    value: String,
    label: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CurrentlyFront {
    label: String,
    text: String,
}

// ---------------------------------------------------------------------------
// Loading
// ---------------------------------------------------------------------------

const IMAGE_EXT: &[&str] = &["jpg", "jpeg", "png", "webp", "avif"];
/// Covers are also textures in the game room, which can't decode AVIF.
const COVER_EXT: &[&str] = &["jpg", "jpeg", "png", "webp"];
const VIDEO_EXT: &[&str] = &["mp4", "webm"];
const ROLE_KINDS: &[&str] = &["job", "freelance", "education", "milestone"];
const LEVELS: &[&str] = &["daily", "confident", "learning"];
/// Cabinet colours handed out in order when a project sets no `accent`.
const PALETTE: [[u8; 3]; 5] = [
    [255, 46, 136],
    [34, 230, 255],
    [255, 176, 32],
    [140, 92, 255],
    [228, 223, 209],
];

/// Collects errors with the file they came from.
struct Errors {
    root: PathBuf,
    list: Vec<String>,
}

impl Errors {
    fn push(&mut self, file: &Path, msg: impl AsRef<str>) {
        let shown = file.strip_prefix(&self.root).unwrap_or(file);
        self.list
            .push(format!("{}: {}", shown.display(), msg.as_ref()));
    }
}

/// Load and validate everything. On failure, every problem found.
pub fn load(paths: &Paths) -> Result<SiteData, Vec<String>> {
    let mut errs = Errors {
        root: paths.root.clone(),
        list: Vec::new(),
    };

    let projects = load_projects(paths, &mut errs);
    let (roles, cv) = load_experience(paths, &mut errs);
    let skills = load_skills(paths, &mut errs);
    let about = load_about(paths, &mut errs);

    if errs.list.is_empty() {
        Ok(SiteData {
            projects,
            roles,
            cv,
            skills,
            about: about.expect("no errors"),
        })
    } else {
        Err(errs.list)
    }
}

/// Files and folders a build script should watch.
pub fn watched(paths: &Paths) -> Vec<PathBuf> {
    vec![paths.content.clone(), paths.assets.join("media")]
}

fn read(path: &Path, errs: &mut Errors) -> Option<String> {
    match fs::read_to_string(path) {
        Ok(s) => Some(s),
        Err(e) => {
            errs.push(path, format!("can't read file ({e})"));
            None
        }
    }
}

/// Split `+++\n<toml>\n+++\n<markdown>`.
fn front_matter<'a>(path: &Path, src: &'a str, errs: &mut Errors) -> Option<(&'a str, &'a str)> {
    let src = src.strip_prefix('\u{feff}').unwrap_or(src);
    let Some(rest) = src
        .strip_prefix("+++\n")
        .or_else(|| src.strip_prefix("+++\r\n"))
    else {
        errs.push(
            path,
            "must start with a `+++` line opening the TOML front matter",
        );
        return None;
    };
    let Some(end) = rest.find("\n+++") else {
        errs.push(
            path,
            "front matter is never closed: add a `+++` line after it",
        );
        return None;
    };
    let body = rest[end + 4..].trim_start_matches(['\r', '\n']);
    Some((&rest[..end + 1], body))
}

fn parse_toml<T: for<'de> Deserialize<'de>>(
    path: &Path,
    src: &str,
    line_offset: usize,
    errs: &mut Errors,
) -> Option<T> {
    match toml::from_str::<T>(src) {
        Ok(v) => Some(v),
        Err(e) => {
            let at = e
                .span()
                .map(|s| src[..s.start].lines().count().max(1) + line_offset);
            let msg = e.message().replace('\n', " ");
            match at {
                Some(line) => errs.push(path, format!("line {line}: {msg}")),
                None => errs.push(path, msg),
            }
            None
        }
    }
}

pub fn markdown(src: &str) -> String {
    let opts =
        Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_SMART_PUNCTUATION;
    let mut out = String::new();
    html::push_html(&mut out, Parser::new_ext(src, opts));
    out
}

fn ext_of(name: &str) -> String {
    Path::new(name)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
}

/// Check a media file exists in `dir` and has an allowed extension; return
/// its site URL, asset-relative path and pixel size (images only).
fn media(
    file: &Path,
    field: &str,
    name: &str,
    dir_rel: &str,
    paths: &Paths,
    allowed: &[&str],
    errs: &mut Errors,
) -> Option<(String, String, u32, u32)> {
    if name.contains("..") || name.starts_with('/') {
        errs.push(
            file,
            format!("{field} \"{name}\" must be a file name inside {dir_rel}/, not a path"),
        );
        return None;
    }
    let ext = ext_of(name);
    if !allowed.contains(&ext.as_str()) {
        errs.push(
            file,
            format!(
                "{field} \"{name}\": .{ext} isn't supported here (use {})",
                allowed.join(", ")
            ),
        );
        return None;
    }
    let rel = format!("{dir_rel}/{name}");
    let full = paths.assets.join(&rel);
    if !full.is_file() {
        errs.push(
            file,
            format!("{field} \"{name}\" not found (expected assets/{rel})"),
        );
        return None;
    }
    let (w, h) = if VIDEO_EXT.contains(&ext.as_str()) {
        (0, 0)
    } else {
        match imagesize::size(&full) {
            Ok(s) => (s.width as u32, s.height as u32),
            Err(e) => {
                errs.push(
                    file,
                    format!("{field} \"{name}\" isn't a readable image ({e})"),
                );
                return None;
            }
        }
    };
    Some((format!("/{rel}"), rel, w, h))
}

fn hex(s: &str) -> Option<[u8; 3]> {
    let s = s.strip_prefix('#')?;
    if s.len() != 6 {
        return None;
    }
    let v = u32::from_str_radix(s, 16).ok()?;
    Some([(v >> 16) as u8, (v >> 8) as u8, v as u8])
}

fn load_projects(paths: &Paths, errs: &mut Errors) -> Vec<ProjectData> {
    let dir = paths.content.join("projects");
    let mut files: Vec<PathBuf> = match fs::read_dir(&dir) {
        Ok(rd) => rd
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| ext_of(&p.to_string_lossy()) == "md")
            .collect(),
        Err(e) => {
            errs.push(&dir, format!("can't read the projects folder ({e})"));
            return Vec::new();
        }
    };
    files.sort();

    let mut out = Vec::new();
    let mut slugs = HashSet::new();
    for (i, file) in files.iter().enumerate() {
        let Some(src) = read(file, errs) else {
            continue;
        };
        let Some((front, body)) = front_matter(file, &src, errs) else {
            continue;
        };
        let Some(f) = parse_toml::<ProjectFront>(file, front, 1, errs) else {
            continue;
        };
        let before = errs.list.len();

        let stem = file.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        if f.slug != stem {
            errs.push(
                file,
                format!(
                    "slug \"{}\" must match the file name (\"{stem}.md\")",
                    f.slug
                ),
            );
        }
        if !f
            .slug
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
            || f.slug.is_empty()
        {
            errs.push(
                file,
                format!("slug \"{}\" may only use a-z, 0-9 and -", f.slug),
            );
        }
        if !slugs.insert(f.slug.clone()) {
            errs.push(
                file,
                format!("slug \"{}\" is used by another project", f.slug),
            );
        }
        if !(1990..=2100).contains(&f.year) {
            errs.push(file, format!("year {} looks wrong", f.year));
        }
        if f.categories.is_empty() {
            errs.push(file, "needs at least one category");
        }
        for (field, v) in [
            ("title", &f.title),
            ("summary", &f.summary),
            ("role", &f.role),
        ] {
            if v.trim().is_empty() {
                errs.push(file, format!("{field} is empty"));
            }
        }
        let accent = match &f.accent {
            Some(a) => hex(a).unwrap_or_else(|| {
                errs.push(
                    file,
                    format!("accent \"{a}\" must be a hex colour like \"#ff2e88\""),
                );
                [0; 3]
            }),
            None => PALETTE[i % PALETTE.len()],
        };

        let dir_rel = format!("media/work/{}", f.slug);
        let cover = media(
            file,
            "cover_image",
            &f.cover_image,
            &dir_rel,
            paths,
            COVER_EXT,
            errs,
        );
        let video = f
            .preview_video
            .as_ref()
            .and_then(|v| media(file, "preview_video", v, &dir_rel, paths, VIDEO_EXT, errs));
        let mut gallery = Vec::new();
        for g in &f.gallery {
            let all: Vec<&str> = IMAGE_EXT.iter().chain(VIDEO_EXT).copied().collect();
            if let Some((src, _, w, h)) =
                media(file, "gallery src", &g.src, &dir_rel, paths, &all, errs)
            {
                gallery.push(MediaData {
                    video: VIDEO_EXT.contains(&ext_of(&g.src).as_str()),
                    src,
                    width: w,
                    height: h,
                    alt: g.alt.clone(),
                    caption: g.caption.clone(),
                });
            }
        }
        for l in &f.links {
            if !(l.url.starts_with("https://")
                || l.url.starts_with("http://")
                || l.url.starts_with("mailto:")
                || l.url.starts_with('/'))
            {
                errs.push(
                    file,
                    format!(
                        "link \"{}\" url must start with https://, http://, mailto: or /",
                        l.label
                    ),
                );
            }
        }

        if errs.list.len() > before {
            continue;
        }
        let (cover_src, cover_asset, w, h) = cover.expect("validated");
        out.push(ProjectData {
            cover: ImageData {
                src: cover_src,
                width: w,
                height: h,
                alt: f.cover_alt.clone().unwrap_or_else(|| f.title.clone()),
            },
            cover_asset,
            preview_video: video.map(|v| v.0),
            body_html: markdown(body),
            gallery,
            links: f.links.into_iter().map(|l| (l.label, l.url)).collect(),
            credits: f.credits.into_iter().map(|c| (c.role, c.name)).collect(),
            accent,
            slug: f.slug,
            title: f.title,
            year: f.year,
            client: f.client,
            role: f.role,
            categories: f.categories,
            tools: f.tools,
            featured: f.featured,
            summary: f.summary,
            example: f.example,
            order: f.order,
        });
    }
    // Newest first; `order` breaks ties within a year.
    out.sort_by(|a, b| {
        b.year
            .cmp(&a.year)
            .then(a.order.cmp(&b.order))
            .then(a.title.cmp(&b.title))
    });
    if out.is_empty() && errs.list.is_empty() {
        errs.push(&dir, "no projects yet: add at least one <slug>.md");
    }
    out
}

fn load_experience(paths: &Paths, errs: &mut Errors) -> (Vec<RoleData>, Option<String>) {
    let file = paths.content.join("experience.toml");
    let Some(src) = read(&file, errs) else {
        return (Vec::new(), None);
    };
    let Some(f) = parse_toml::<ExperienceFile>(&file, &src, 0, errs) else {
        return (Vec::new(), None);
    };
    let cv =
        f.cv.as_ref()
            .and_then(|cv| media(&file, "cv", cv, "media", paths, &["pdf"], errs).map(|m| m.0));
    let roles = f
        .role
        .into_iter()
        .filter_map(|r| {
            if !ROLE_KINDS.contains(&r.kind.as_str()) {
                errs.push(
                    &file,
                    format!(
                        "role \"{}\": kind \"{}\" must be one of {}",
                        r.title,
                        r.kind,
                        ROLE_KINDS.join(", ")
                    ),
                );
                return None;
            }
            Some(RoleData {
                title: r.title,
                org: r.org,
                kind: r.kind,
                start: r.start,
                end: r.end,
                location: r.location,
                summary: r.summary,
                placeholder: r.placeholder,
            })
        })
        .collect();
    (roles, cv)
}

fn load_skills(paths: &Paths, errs: &mut Errors) -> Vec<SkillGroupData> {
    let file = paths.content.join("skills.toml");
    let Some(src) = read(&file, errs) else {
        return Vec::new();
    };
    let Some(f) = parse_toml::<SkillsFile>(&file, &src, 0, errs) else {
        return Vec::new();
    };
    f.group
        .into_iter()
        .map(|g| SkillGroupData {
            skills: g
                .skills
                .into_iter()
                .filter_map(|s| {
                    if !LEVELS.contains(&s.level.as_str()) {
                        errs.push(
                            &file,
                            format!(
                                "skill \"{}\": level \"{}\" must be one of {}",
                                s.name,
                                s.level,
                                LEVELS.join(", ")
                            ),
                        );
                        return None;
                    }
                    Some((s.name, s.level, s.note))
                })
                .collect(),
            name: g.name,
            blurb: g.blurb,
            placeholder: g.placeholder,
        })
        .collect()
}

fn load_about(paths: &Paths, errs: &mut Errors) -> Option<AboutData> {
    let file = paths.content.join("about.md");
    let src = read(&file, errs)?;
    let (front, body) = front_matter(&file, &src, errs)?;
    let f = parse_toml::<AboutFront>(&file, front, 1, errs)?;
    if !f.email.contains('@') || f.email.contains(char::is_whitespace) {
        errs.push(
            &file,
            format!("email \"{}\" doesn't look like an email address", f.email),
        );
    }
    let portrait = match &f.portrait {
        Some(p) => Some(
            media(&file, "portrait", p, "media/about", paths, IMAGE_EXT, errs).map(
                |(src, _, w, h)| ImageData {
                    src,
                    width: w,
                    height: h,
                    alt: f
                        .portrait_alt
                        .clone()
                        .unwrap_or_else(|| format!("Portrait of {}", f.name)),
                },
            )?,
        ),
        None => None,
    };
    Some(AboutData {
        body_html: markdown(body),
        stats: f.stats.into_iter().map(|s| (s.value, s.label)).collect(),
        currently: f.currently.into_iter().map(|c| (c.label, c.text)).collect(),
        portrait,
        name: f.name,
        role: f.role,
        location: f.location,
        intro: f.intro,
        placeholder: f.placeholder,
        email: f.email,
        reply_time: f.reply_time,
    })
}

/// Human-readable report of a failed load, for build scripts.
pub fn report(errors: &[String]) -> String {
    let mut s = format!("content/ has {} problem(s):\n", errors.len());
    for e in errors {
        let _ = writeln!(s, "  • {e}");
    }
    s.push_str("See CONTENT.md for the expected format.");
    s
}
