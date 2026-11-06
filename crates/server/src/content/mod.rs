//! ContentModule: read-only JSON over the site's content (projects, skills,
//! experience from the database; about from `content/`). The pages read the
//! same snapshot directly; this API is for the room and other clients.

use content::{Project, Site};
use nestforge::serde_json::{Value, json};
use nestforge::{Container, HttpException, Inject, Provider, controller, module, routes};

use crate::core::JsonBody;

/// The content repository: always the current snapshot (see `site::data`),
/// so dashboard edits show up here as soon as they're saved.
pub struct ContentRepository;

impl ContentRepository {
    pub fn live() -> Self {
        Self
    }

    pub fn site(&self) -> &'static Site {
        site::data::site()
    }

    pub fn project(&self, slug: &str) -> Option<&'static Project> {
        self.site().project(slug)
    }
}

pub fn project_json(p: &Project) -> Value {
    json!({
        "slug": p.slug,
        "title": p.title,
        "year": p.year,
        "client": p.client,
        "role": p.role,
        "categories": p.categories,
        "tools": p.tools,
        "featured": p.featured,
        "accent": format!("#{:02x}{:02x}{:02x}", p.accent[0], p.accent[1], p.accent[2]),
        "cover": { "src": p.cover.src, "width": p.cover.width, "height": p.cover.height, "alt": p.cover.alt },
        "preview_video": p.preview_video,
        "summary": p.summary,
        "url": format!("/work/{}", p.slug),
        "example": p.example,
    })
}

#[controller("/api/content")]
#[derive(Default)]
pub struct ContentController;

#[routes]
impl ContentController {
    #[nestforge::get("/projects")]
    async fn projects(repo: Inject<ContentRepository>) -> Result<JsonBody, HttpException> {
        Ok(JsonBody(Value::Array(
            repo.site().projects.iter().map(project_json).collect(),
        )))
    }

    #[nestforge::get("/projects/{slug}")]
    async fn project(
        repo: Inject<ContentRepository>,
        axum::extract::Path(slug): axum::extract::Path<String>,
    ) -> Result<JsonBody, HttpException> {
        let p = repo
            .project(&slug)
            .ok_or_else(|| HttpException::not_found(format!("No project \"{slug}\"")))?;
        let mut v = project_json(p);
        v["body_html"] = json!(p.body_html);
        v["gallery"] = json!(p.gallery.iter().map(|m| json!({ "src": m.src, "alt": m.alt, "caption": m.caption, "width": m.width, "height": m.height })).collect::<Vec<_>>());
        v["links"] = json!(
            p.links
                .iter()
                .map(|l| json!({ "label": l.label, "url": l.url }))
                .collect::<Vec<_>>()
        );
        v["credits"] = json!(
            p.credits
                .iter()
                .map(|c| json!({ "role": c.role, "name": c.name }))
                .collect::<Vec<_>>()
        );
        Ok(JsonBody(v))
    }

    #[nestforge::get("/skills")]
    async fn skills(repo: Inject<ContentRepository>) -> Result<JsonBody, HttpException> {
        Ok(JsonBody(json!(repo.site().skills.iter().map(|g| json!({
            "name": g.name,
            "blurb": g.blurb,
            "skills": g.skills.iter().map(|s| json!({ "name": s.name, "level": s.level.label(), "note": s.note })).collect::<Vec<_>>(),
        })).collect::<Vec<_>>())))
    }

    #[nestforge::get("/experience")]
    async fn experience(repo: Inject<ContentRepository>) -> Result<JsonBody, HttpException> {
        let e = repo.site().experience;
        Ok(JsonBody(json!({
            "cv": e.cv,
            "roles": e.roles.iter().map(|r| json!({
                "title": r.title, "org": r.org, "kind": r.kind.label(), "start": r.start,
                "end": r.end, "location": r.location, "summary": r.summary,
            })).collect::<Vec<_>>(),
        })))
    }

    #[nestforge::get("/about")]
    async fn about(repo: Inject<ContentRepository>) -> Result<JsonBody, HttpException> {
        let a = repo.site().about;
        Ok(JsonBody(json!({
            "name": a.name, "role": a.role, "location": a.location, "intro": a.intro,
            "email": a.email, "reply_time": a.reply_time,
            "stats": a.stats.iter().map(|s| json!({ "value": s.value, "label": s.label })).collect::<Vec<_>>(),
            "currently": a.currently.iter().map(|c| json!({ "label": c.label, "text": c.text })).collect::<Vec<_>>(),
        })))
    }
}

#[module(
    controllers = [ContentController],
    providers = [Provider::factory(|_: &Container| Ok(ContentRepository::live()))],
    exports = [ContentRepository]
)]
pub struct ContentModule;
