//! Turn loaded content into Rust source for `'static` data
//! (`pub static SITE: content::Site = …;`).

use std::fmt::Write as _;

use crate::load::{ImageData, SiteData};

/// A Rust string literal (Debug formatting escapes everything needed).
fn s(v: &str) -> String {
    format!("{v:?}")
}

fn strs(v: &[String]) -> String {
    format!(
        "&[{}]",
        v.iter().map(|x| s(x)).collect::<Vec<_>>().join(", ")
    )
}

fn image(i: &ImageData) -> String {
    format!(
        "content::Image {{ src: {}, width: {}, height: {}, alt: {} }}",
        s(&i.src),
        i.width,
        i.height,
        s(&i.alt)
    )
}

fn level(l: &str) -> &'static str {
    match l {
        "daily" => "content::Level::Daily",
        "confident" => "content::Level::Confident",
        _ => "content::Level::Learning",
    }
}

fn kind(k: &str) -> &'static str {
    match k {
        "job" => "content::RoleKind::Job",
        "freelance" => "content::RoleKind::Freelance",
        "education" => "content::RoleKind::Education",
        _ => "content::RoleKind::Milestone",
    }
}

pub fn render(d: &SiteData) -> String {
    let mut o =
        String::from("// @generated from content/ by the site's build script. Do not edit.\n\n");
    o.push_str("pub static SITE: content::Site = content::Site {\n    projects: &[\n");
    for p in &d.projects {
        let gallery: Vec<String> = p
            .gallery
            .iter()
            .map(|m| {
                format!(
                    "content::Media {{ kind: content::MediaKind::{}, src: {}, width: {}, height: {}, alt: {}, caption: {} }}",
                    if m.video { "Video" } else { "Image" },
                    s(&m.src),
                    m.width,
                    m.height,
                    s(&m.alt),
                    s(&m.caption)
                )
            })
            .collect();
        let links: Vec<String> = p
            .links
            .iter()
            .map(|(l, u)| format!("content::Link {{ label: {}, url: {} }}", s(l), s(u)))
            .collect();
        let credits: Vec<String> = p
            .credits
            .iter()
            .map(|(r, n)| format!("content::Credit {{ role: {}, name: {} }}", s(r), s(n)))
            .collect();
        let _ = writeln!(
            o,
            "        content::Project {{ slug: {}, title: {}, year: {}, client: {}, role: {}, categories: {}, tools: {}, \
             featured: {}, accent: {:?}, cover: {}, cover_asset: {}, preview_video: {}, summary: {}, body_html: {}, \
             gallery: &[{}], links: &[{}], credits: &[{}], example: {} }},",
            s(&p.slug),
            s(&p.title),
            p.year,
            s(&p.client),
            s(&p.role),
            strs(&p.categories),
            strs(&p.tools),
            p.featured,
            p.accent,
            image(&p.cover),
            s(&p.cover_asset),
            p.preview_video
                .as_ref()
                .map(|v| format!("Some({})", s(v)))
                .unwrap_or_else(|| "None".into()),
            s(&p.summary),
            s(&p.body_html),
            gallery.join(", "),
            links.join(", "),
            credits.join(", "),
            p.example,
        );
    }
    o.push_str("    ],\n    experience: content::Experience {\n        roles: &[\n");
    for r in &d.roles {
        let _ = writeln!(
            o,
            "            content::Role {{ title: {}, org: {}, kind: {}, start: {}, end: {}, location: {}, summary: {}, placeholder: {} }},",
            s(&r.title),
            s(&r.org),
            kind(&r.kind),
            s(&r.start),
            s(&r.end),
            s(&r.location),
            s(&r.summary),
            r.placeholder
        );
    }
    let _ = writeln!(
        o,
        "        ],\n        cv: {},\n    }},",
        d.cv.as_ref()
            .map(|c| format!("Some({})", s(c)))
            .unwrap_or_else(|| "None".into())
    );
    o.push_str("    skills: &[\n");
    for g in &d.skills {
        let skills: Vec<String> = g
            .skills
            .iter()
            .map(|(n, l, note)| {
                format!(
                    "content::Skill {{ name: {}, level: {}, note: {} }}",
                    s(n),
                    level(l),
                    s(note)
                )
            })
            .collect();
        let _ = writeln!(
            o,
            "        content::SkillGroup {{ name: {}, blurb: {}, skills: &[{}], placeholder: {} }},",
            s(&g.name),
            s(&g.blurb),
            skills.join(", "),
            g.placeholder
        );
    }
    let a = &d.about;
    let stats: Vec<String> = a
        .stats
        .iter()
        .map(|(v, l)| format!("content::Stat {{ value: {}, label: {} }}", s(v), s(l)))
        .collect();
    let currently: Vec<String> = a
        .currently
        .iter()
        .map(|(l, t)| format!("content::Currently {{ label: {}, text: {} }}", s(l), s(t)))
        .collect();
    let _ = writeln!(
        o,
        "    ],\n    about: content::About {{ name: {}, role: {}, location: {}, intro: {}, portrait: {}, body_html: {}, \
         stats: &[{}], currently: &[{}], placeholder: {}, email: {}, reply_time: {} }},\n}};",
        s(&a.name),
        s(&a.role),
        s(&a.location),
        s(&a.intro),
        a.portrait
            .as_ref()
            .map(|p| format!("Some({})", image(p)))
            .unwrap_or_else(|| "None".into()),
        s(&a.body_html),
        stats.join(", "),
        currently.join(", "),
        a.placeholder,
        s(&a.email),
        s(&a.reply_time)
    );
    o
}
