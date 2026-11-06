//! The dashboard's server functions. Every one checks the session cookie
//! first (except [`admin_me`], which reports it), and every content edit
//! republishes the snapshot the public pages read.

use content::owned::{Project, Role, SkillGroup};
use leptos::prelude::*;
use leptos::server_fn::codec::Json;

#[cfg(feature = "ssr")]
use super::types::ReplyRow;
use super::types::{Kind, MessageRow, Row, Thread};

#[cfg(feature = "ssr")]
mod backend {
    use leptos::prelude::*;

    use crate::admin::{Backend, session_from};

    /// Is this request signed in?
    pub async fn signed_in(b: &Backend) -> bool {
        let Ok(headers) = leptos_axum::extract::<http::HeaderMap>().await else {
            return false;
        };
        match session_from(&headers) {
            Some(s) => b
                .store
                .session_valid(&store::secret::hash(&s))
                .await
                .unwrap_or(false),
            None => false,
        }
    }

    /// The backend, if this request is signed in.
    pub async fn admin() -> Result<Backend, ServerFnError> {
        let b = use_context::<Backend>()
            .ok_or_else(|| ServerFnError::new("The dashboard isn't configured on this server."))?;
        if !signed_in(&b).await {
            return Err(ServerFnError::new("Signed out. Sign in again."));
        }
        Ok(b)
    }

    pub fn fail(e: impl std::fmt::Display) -> ServerFnError {
        ServerFnError::new(e.to_string())
    }

    /// After a content edit: the public pages read the new snapshot.
    pub async fn publish(b: &Backend) -> Result<(), ServerFnError> {
        crate::data::set_site(b.store.catalog().await.map_err(fail)?);
        Ok(())
    }

    pub fn collection(k: crate::admin::types::Kind) -> store::Collection {
        use crate::admin::types::Kind;
        match k {
            Kind::Projects => store::Collection::Projects,
            Kind::Roles => store::Collection::Roles,
            Kind::SkillGroups => store::Collection::SkillGroups,
        }
    }
}

#[cfg(feature = "ssr")]
fn row(m: store::Message) -> MessageRow {
    MessageRow {
        id: m.id,
        created_at: m.created_at,
        name: m.name,
        email: m.email,
        project_type: m.project_type,
        budget: m.budget,
        body: m.body,
        status: m.status,
        replies: m.replies,
    }
}

/// Whether this browser is signed in.
#[server]
pub async fn admin_me() -> Result<bool, ServerFnError> {
    match use_context::<super::Backend>() {
        Some(b) => Ok(backend::signed_in(&b).await),
        None => Ok(false),
    }
}

/// Unread enquiries (for the sidebar badge).
#[server]
pub async fn new_count() -> Result<i64, ServerFnError> {
    let b = backend::admin().await?;
    b.store.count_new().await.map_err(backend::fail)
}

/// `filter`: `open` (everything not archived), `new`, `replied` or `archived`.
#[server]
pub async fn list_messages(filter: String) -> Result<Vec<MessageRow>, ServerFnError> {
    let b = backend::admin().await?;
    let status = match filter.as_str() {
        "new" | "read" | "replied" | "archived" => Some(filter.as_str()),
        _ => None,
    };
    Ok(b.store
        .messages(status)
        .await
        .map_err(backend::fail)?
        .into_iter()
        .map(row)
        .collect())
}

/// A message and its replies. Opening a new message marks it read.
#[server]
pub async fn get_thread(id: i64) -> Result<Thread, ServerFnError> {
    let b = backend::admin().await?;
    let mut m = b.store.message(id).await.map_err(backend::fail)?;
    if m.status == "new" {
        b.store
            .set_status(id, "read")
            .await
            .map_err(backend::fail)?;
        m.status = "read".into();
    }
    let replies = b
        .store
        .replies(id)
        .await
        .map_err(backend::fail)?
        .into_iter()
        .map(|r| ReplyRow {
            id: r.id,
            created_at: r.created_at,
            body: r.body,
            error: r.error,
        })
        .collect();
    Ok(Thread {
        message: row(m),
        replies,
    })
}

/// Email a reply to the visitor, threaded under the earlier replies, and
/// record it (a failed send is recorded with its error, and reported).
#[server]
pub async fn reply(id: i64, body: String) -> Result<(), ServerFnError> {
    let b = backend::admin().await?;
    let body = body.trim().to_string();
    if body.is_empty() {
        return Err(ServerFnError::new("Write something first."));
    }
    let m = b.store.message(id).await.map_err(backend::fail)?;
    let earlier: Vec<String> = b
        .store
        .replies(id)
        .await
        .map_err(backend::fail)?
        .into_iter()
        .filter(|r| r.error.is_none())
        .map(|r| r.email_id)
        .collect();
    // Message-IDs on the sending domain: <arthub.12.3.ab12cd34@example.com>.
    let from = &b.mailer.config().from;
    let domain = from
        .rsplit_once('@')
        .map(|(_, d)| d.trim_end_matches('>').trim())
        .filter(|d| !d.is_empty())
        .unwrap_or("arthub.local");
    let email_id = format!(
        "<arthub.{id}.{}.{}@{domain}>",
        earlier.len() + 1,
        &store::secret::new_token()[..8]
    );
    let text = format!(
        "{body}\n\n—\n{}\n\n> On {} you wrote:\n{}\n",
        crate::config::NAME,
        super::types::when(m.created_at),
        m.body
            .lines()
            .map(|l| format!("> {l}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
    let sent = b
        .mailer
        .send_mail(&contact::mail::Outgoing {
            to: m.email.clone(),
            subject: format!("Re: your enquiry ({})", m.project_type),
            text,
            reply_to: Some(b.reply_to.clone()),
            message_id: Some(email_id.clone()),
            in_reply_to: earlier.last().cloned(),
            references: earlier,
        })
        .await;
    let error = sent.as_ref().err().cloned();
    b.store
        .add_reply(id, &body, &email_id, error.as_deref())
        .await
        .map_err(backend::fail)?;
    match error {
        None => Ok(()),
        Some(e) => Err(ServerFnError::new(format!(
            "Saved, but the email didn't send: {e}"
        ))),
    }
}

#[server]
pub async fn set_status(id: i64, status: String) -> Result<(), ServerFnError> {
    let b = backend::admin().await?;
    if !store::STATUSES.contains(&status.as_str()) {
        return Err(ServerFnError::new("Unknown status."));
    }
    b.store.set_status(id, &status).await.map_err(backend::fail)
}

#[server]
pub async fn delete_message(id: i64) -> Result<(), ServerFnError> {
    let b = backend::admin().await?;
    b.store.delete_message(id).await.map_err(backend::fail)
}

// ---- content ---------------------------------------------------------------

#[server]
pub async fn list_projects() -> Result<Vec<Row<Project>>, ServerFnError> {
    let b = backend::admin().await?;
    Ok(b.store
        .list::<Project>(store::Collection::Projects)
        .await
        .map_err(backend::fail)?
        .into_iter()
        .map(|i| Row {
            id: i.id,
            data: i.data,
        })
        .collect())
}

#[server]
pub async fn get_project(id: i64) -> Result<Project, ServerFnError> {
    let b = backend::admin().await?;
    b.store
        .get(store::Collection::Projects, id)
        .await
        .map_err(backend::fail)
}

/// Problems with a project, if any (shown above the form).
pub fn check_project(p: &Project) -> Vec<String> {
    let mut e = Vec::new();
    if p.title.trim().is_empty() {
        e.push("It needs a title.".into());
    }
    if p.slug.is_empty()
        || !p
            .slug
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        e.push("The slug is the URL part: lowercase letters, numbers and dashes.".into());
    }
    if p.cover.src.trim().is_empty() {
        e.push("Upload a cover image.".into());
    }
    if !(1990..=2100).contains(&p.year) {
        e.push("The year looks off.".into());
    }
    e
}

/// Create (`id: None`) or update a project. Returns its id.
#[server(input = Json)]
pub async fn save_project(id: Option<i64>, project: Project) -> Result<i64, ServerFnError> {
    let b = backend::admin().await?;
    let mut p = project;
    let problems = check_project(&p);
    if !problems.is_empty() {
        return Err(ServerFnError::new(problems.join(" ")));
    }
    p.body_html = content::load::markdown(&p.body_md);
    let id = b.store.save_project(id, &p).await.map_err(backend::fail)?;
    backend::publish(&b).await?;
    Ok(id)
}

#[server]
pub async fn list_roles() -> Result<Vec<Row<Role>>, ServerFnError> {
    let b = backend::admin().await?;
    Ok(b.store
        .list::<Role>(store::Collection::Roles)
        .await
        .map_err(backend::fail)?
        .into_iter()
        .map(|i| Row {
            id: i.id,
            data: i.data,
        })
        .collect())
}

#[server(input = Json)]
pub async fn save_role(id: Option<i64>, role: Role) -> Result<i64, ServerFnError> {
    let b = backend::admin().await?;
    if role.title.trim().is_empty() || role.org.trim().is_empty() {
        return Err(ServerFnError::new(
            "A role needs a title and an organisation.",
        ));
    }
    let id = b.store.save_role(id, &role).await.map_err(backend::fail)?;
    backend::publish(&b).await?;
    Ok(id)
}

#[server]
pub async fn list_skill_groups() -> Result<Vec<Row<SkillGroup>>, ServerFnError> {
    let b = backend::admin().await?;
    Ok(b.store
        .list::<SkillGroup>(store::Collection::SkillGroups)
        .await
        .map_err(backend::fail)?
        .into_iter()
        .map(|i| Row {
            id: i.id,
            data: i.data,
        })
        .collect())
}

#[server(input = Json)]
pub async fn save_skill_group(id: Option<i64>, group: SkillGroup) -> Result<i64, ServerFnError> {
    let b = backend::admin().await?;
    if group.name.trim().is_empty() {
        return Err(ServerFnError::new("A group needs a name."));
    }
    let id = b
        .store
        .save_skill_group(id, &group)
        .await
        .map_err(backend::fail)?;
    backend::publish(&b).await?;
    Ok(id)
}

#[server(input = Json)]
pub async fn delete_item(kind: Kind, id: i64) -> Result<(), ServerFnError> {
    let b = backend::admin().await?;
    b.store
        .delete(backend::collection(kind), id)
        .await
        .map_err(backend::fail)?;
    backend::publish(&b).await
}

#[server(input = Json)]
pub async fn move_item(kind: Kind, id: i64, up: bool) -> Result<(), ServerFnError> {
    let b = backend::admin().await?;
    b.store
        .move_item(backend::collection(kind), id, up)
        .await
        .map_err(backend::fail)?;
    backend::publish(&b).await
}
