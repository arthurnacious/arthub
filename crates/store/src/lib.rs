//! SQLite persistence for the dashboard.
//!
//! * Content: projects, experience roles and skill groups, each row one
//!   [`content::owned`] item as JSON, ordered by `position`. An empty
//!   database is seeded once from the compiled `content/` files.
//! * Contact messages and the replies sent from the dashboard.
//! * Admin login: magic-link tokens and sessions (stored as hashes; the
//!   caller hashes).
//!
//! Times are Unix seconds.

use content::owned::{Catalog, Project, Role, SkillGroup};
use serde::{Serialize, de::DeserializeOwned};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use sqlx::{Row, SqlitePool};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("database: {0}")]
    Db(#[from] sqlx::Error),
    #[error("migrating the database: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),
    #[error("stored data: {0}")]
    Json(#[from] serde_json::Error),
    #[error("not found")]
    NotFound,
    #[error("a project with the slug \"{0}\" already exists")]
    SlugTaken(String),
}

pub type Result<T> = std::result::Result<T, Error>;

/// Login tokens and session ids: random, and only ever stored hashed.
pub mod secret {
    use sha2::{Digest, Sha256};

    fn hex(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    }

    /// 32 random bytes as hex.
    pub fn new_token() -> String {
        let mut b = [0u8; 32];
        getrandom::fill(&mut b).expect("system randomness");
        hex(&b)
    }

    /// What gets stored for a token.
    pub fn hash(token: &str) -> String {
        hex(&Sha256::digest(token.as_bytes()))
    }
}

pub fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// The content collections, one table each.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Collection {
    Projects,
    Roles,
    SkillGroups,
}

impl Collection {
    fn table(self) -> &'static str {
        match self {
            Collection::Projects => "projects",
            Collection::Roles => "roles",
            Collection::SkillGroups => "skill_groups",
        }
    }
}

/// A stored item with its row id.
#[derive(Clone, Debug, PartialEq)]
pub struct Item<T> {
    pub id: i64,
    pub data: T,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Message {
    pub id: i64,
    pub created_at: i64,
    pub name: String,
    pub email: String,
    pub project_type: String,
    pub budget: String,
    pub body: String,
    /// `new`, `read`, `replied` or `archived`.
    pub status: String,
    pub replies: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Reply {
    pub id: i64,
    pub created_at: i64,
    pub body: String,
    pub email_id: String,
    pub error: Option<String>,
}

pub const STATUSES: [&str; 4] = ["new", "read", "replied", "archived"];

#[derive(Clone)]
pub struct Store {
    pool: SqlitePool,
}

impl Store {
    /// Open (creating if needed) and migrate. `path` is a file, or
    /// `:memory:` for tests.
    pub async fn open(path: &str) -> Result<Self> {
        let opts = if path == ":memory:" {
            SqliteConnectOptions::new().in_memory(true)
        } else {
            SqliteConnectOptions::new()
                .filename(path)
                .create_if_missing(true)
                .journal_mode(SqliteJournalMode::Wal)
        }
        .foreign_keys(true);
        // One connection in memory, or each would get its own database.
        let max = if path == ":memory:" { 1 } else { 4 };
        let pool = SqlitePoolOptions::new()
            .max_connections(max)
            .connect_with(opts)
            .await?;
        sqlx::migrate!("./migrations").run(&pool).await?;
        Ok(Self { pool })
    }

    // ---- content ---------------------------------------------------------

    /// Fill an empty database from the compiled content, once. Returns
    /// whether it seeded.
    pub async fn seed_if_empty(&self, seed: &Catalog) -> Result<bool> {
        if self.setting("seeded").await?.is_some() {
            return Ok(false);
        }
        let mut tx = self.pool.begin().await?;
        let t = now();
        for (i, p) in seed.projects.iter().enumerate() {
            sqlx::query(
                "INSERT INTO projects (position, slug, data, updated_at) VALUES (?, ?, ?, ?)",
            )
            .bind(i as i64)
            .bind(&p.slug)
            .bind(serde_json::to_string(p)?)
            .bind(t)
            .execute(&mut *tx)
            .await?;
        }
        for (table, items) in [
            ("roles", to_json(&seed.roles)?),
            ("skill_groups", to_json(&seed.skills)?),
        ] {
            for (i, data) in items.into_iter().enumerate() {
                sqlx::query(&format!(
                    "INSERT INTO {table} (position, data, updated_at) VALUES (?, ?, ?)"
                ))
                .bind(i as i64)
                .bind(data)
                .bind(t)
                .execute(&mut *tx)
                .await?;
            }
        }
        if let Some(cv) = &seed.cv {
            set_setting(&mut tx, "cv", cv).await?;
        }
        set_setting(&mut tx, "seeded", &t.to_string()).await?;
        tx.commit().await?;
        Ok(true)
    }

    /// Everything the public site shows, in display order.
    pub async fn catalog(&self) -> Result<Catalog> {
        Ok(Catalog {
            projects: self
                .list(Collection::Projects)
                .await?
                .into_iter()
                .map(|i| i.data)
                .collect(),
            roles: self
                .list(Collection::Roles)
                .await?
                .into_iter()
                .map(|i| i.data)
                .collect(),
            cv: self.setting("cv").await?.filter(|s| !s.is_empty()),
            skills: self
                .list(Collection::SkillGroups)
                .await?
                .into_iter()
                .map(|i| i.data)
                .collect(),
        })
    }

    pub async fn list<T: DeserializeOwned>(&self, c: Collection) -> Result<Vec<Item<T>>> {
        let rows = sqlx::query(&format!(
            "SELECT id, data FROM {} ORDER BY position, id",
            c.table()
        ))
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(|r| {
                Ok(Item {
                    id: r.get("id"),
                    data: serde_json::from_str(r.get::<&str, _>("data"))?,
                })
            })
            .collect()
    }

    pub async fn get<T: DeserializeOwned>(&self, c: Collection, id: i64) -> Result<T> {
        let row = sqlx::query(&format!("SELECT data FROM {} WHERE id = ?", c.table()))
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or(Error::NotFound)?;
        Ok(serde_json::from_str(row.get::<&str, _>("data"))?)
    }

    /// Insert (`id: None`, appended at the end; projects go first, newest
    /// first) or update. Returns the id.
    pub async fn save_project(&self, id: Option<i64>, p: &Project) -> Result<i64> {
        let taken: Option<i64> = sqlx::query_scalar("SELECT id FROM projects WHERE slug = ?")
            .bind(&p.slug)
            .fetch_optional(&self.pool)
            .await?;
        if taken.is_some_and(|t| Some(t) != id) {
            return Err(Error::SlugTaken(p.slug.clone()));
        }
        let data = serde_json::to_string(p)?;
        match id {
            Some(id) => {
                let n = sqlx::query(
                    "UPDATE projects SET slug = ?, data = ?, updated_at = ? WHERE id = ?",
                )
                .bind(&p.slug)
                .bind(data)
                .bind(now())
                .bind(id)
                .execute(&self.pool)
                .await?
                .rows_affected();
                if n == 0 {
                    return Err(Error::NotFound);
                }
                Ok(id)
            }
            None => {
                let first: i64 =
                    sqlx::query_scalar("SELECT COALESCE(MIN(position), 0) - 1 FROM projects")
                        .fetch_one(&self.pool)
                        .await?;
                Ok(sqlx::query(
                    "INSERT INTO projects (position, slug, data, updated_at) VALUES (?, ?, ?, ?)",
                )
                .bind(first)
                .bind(&p.slug)
                .bind(data)
                .bind(now())
                .execute(&self.pool)
                .await?
                .last_insert_rowid())
            }
        }
    }

    pub async fn save_role(&self, id: Option<i64>, r: &Role) -> Result<i64> {
        self.save_plain(Collection::Roles, id, r).await
    }

    pub async fn save_skill_group(&self, id: Option<i64>, g: &SkillGroup) -> Result<i64> {
        self.save_plain(Collection::SkillGroups, id, g).await
    }

    async fn save_plain<T: Serialize>(&self, c: Collection, id: Option<i64>, v: &T) -> Result<i64> {
        let data = serde_json::to_string(v)?;
        let table = c.table();
        match id {
            Some(id) => {
                let n = sqlx::query(&format!(
                    "UPDATE {table} SET data = ?, updated_at = ? WHERE id = ?"
                ))
                .bind(data)
                .bind(now())
                .bind(id)
                .execute(&self.pool)
                .await?
                .rows_affected();
                if n == 0 {
                    return Err(Error::NotFound);
                }
                Ok(id)
            }
            None => {
                let last: i64 = sqlx::query_scalar(&format!(
                    "SELECT COALESCE(MAX(position), -1) + 1 FROM {table}"
                ))
                .fetch_one(&self.pool)
                .await?;
                Ok(sqlx::query(&format!(
                    "INSERT INTO {table} (position, data, updated_at) VALUES (?, ?, ?)"
                ))
                .bind(last)
                .bind(data)
                .bind(now())
                .execute(&self.pool)
                .await?
                .last_insert_rowid())
            }
        }
    }

    pub async fn delete(&self, c: Collection, id: i64) -> Result<()> {
        let n = sqlx::query(&format!("DELETE FROM {} WHERE id = ?", c.table()))
            .bind(id)
            .execute(&self.pool)
            .await?
            .rows_affected();
        if n == 0 { Err(Error::NotFound) } else { Ok(()) }
    }

    /// Swap an item with its neighbour (`up` = earlier in the list).
    pub async fn move_item(&self, c: Collection, id: i64, up: bool) -> Result<()> {
        let table = c.table();
        let mut tx = self.pool.begin().await?;
        let ids: Vec<i64> =
            sqlx::query_scalar(&format!("SELECT id FROM {table} ORDER BY position, id"))
                .fetch_all(&mut *tx)
                .await?;
        let i = ids.iter().position(|x| *x == id).ok_or(Error::NotFound)?;
        let j = if up {
            i.checked_sub(1)
        } else {
            Some(i + 1).filter(|j| *j < ids.len())
        };
        if let Some(j) = j {
            let mut order = ids.clone();
            order.swap(i, j);
            // Renumber densely; keeps positions tidy whatever came before.
            for (pos, id) in order.iter().enumerate() {
                sqlx::query(&format!("UPDATE {table} SET position = ? WHERE id = ?"))
                    .bind(pos as i64)
                    .bind(id)
                    .execute(&mut *tx)
                    .await?;
            }
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn setting(&self, key: &str) -> Result<Option<String>> {
        Ok(
            sqlx::query_scalar("SELECT value FROM settings WHERE key = ?")
                .bind(key)
                .fetch_optional(&self.pool)
                .await?,
        )
    }

    pub async fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        let mut conn = self.pool.acquire().await?;
        set_setting(&mut conn, key, value).await
    }

    // ---- messages --------------------------------------------------------

    pub async fn add_message(
        &self,
        name: &str,
        email: &str,
        project_type: &str,
        budget: &str,
        body: &str,
    ) -> Result<i64> {
        Ok(sqlx::query(
            "INSERT INTO messages (created_at, name, email, project_type, budget, body) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(now())
        .bind(name)
        .bind(email)
        .bind(project_type)
        .bind(budget)
        .bind(body)
        .execute(&self.pool)
        .await?
        .last_insert_rowid())
    }

    /// Newest first. `status: None` lists everything except archived.
    pub async fn messages(&self, status: Option<&str>) -> Result<Vec<Message>> {
        let sql = "SELECT m.*, (SELECT COUNT(*) FROM replies r WHERE r.message_id = m.id) AS replies \
                   FROM messages m WHERE (?1 IS NULL AND m.status != 'archived') OR m.status = ?1 \
                   ORDER BY m.created_at DESC, m.id DESC";
        let rows = sqlx::query(sql).bind(status).fetch_all(&self.pool).await?;
        Ok(rows.iter().map(message).collect())
    }

    pub async fn message(&self, id: i64) -> Result<Message> {
        let row = sqlx::query(
            "SELECT m.*, (SELECT COUNT(*) FROM replies r WHERE r.message_id = m.id) AS replies \
             FROM messages m WHERE m.id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(Error::NotFound)?;
        Ok(message(&row))
    }

    pub async fn count_new(&self) -> Result<i64> {
        Ok(
            sqlx::query_scalar("SELECT COUNT(*) FROM messages WHERE status = 'new'")
                .fetch_one(&self.pool)
                .await?,
        )
    }

    pub async fn set_status(&self, id: i64, status: &str) -> Result<()> {
        let n = sqlx::query("UPDATE messages SET status = ? WHERE id = ?")
            .bind(status)
            .bind(id)
            .execute(&self.pool)
            .await?
            .rows_affected();
        if n == 0 { Err(Error::NotFound) } else { Ok(()) }
    }

    pub async fn delete_message(&self, id: i64) -> Result<()> {
        let n = sqlx::query("DELETE FROM messages WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?
            .rows_affected();
        if n == 0 { Err(Error::NotFound) } else { Ok(()) }
    }

    pub async fn replies(&self, message_id: i64) -> Result<Vec<Reply>> {
        let rows = sqlx::query(
            "SELECT id, created_at, body, email_id, error FROM replies WHERE message_id = ? ORDER BY created_at, id",
        )
        .bind(message_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .iter()
            .map(|r| Reply {
                id: r.get("id"),
                created_at: r.get("created_at"),
                body: r.get("body"),
                email_id: r.get("email_id"),
                error: r.get("error"),
            })
            .collect())
    }

    /// Record a reply (sent, or failed with `error`). A sent reply marks the
    /// message `replied`.
    pub async fn add_reply(
        &self,
        message_id: i64,
        body: &str,
        email_id: &str,
        error: Option<&str>,
    ) -> Result<i64> {
        let mut tx = self.pool.begin().await?;
        let id = sqlx::query(
            "INSERT INTO replies (message_id, created_at, body, email_id, error) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(message_id)
        .bind(now())
        .bind(body)
        .bind(email_id)
        .bind(error)
        .execute(&mut *tx)
        .await?
        .last_insert_rowid();
        if error.is_none() {
            sqlx::query("UPDATE messages SET status = 'replied' WHERE id = ?")
                .bind(message_id)
                .execute(&mut *tx)
                .await?;
        }
        tx.commit().await?;
        Ok(id)
    }

    // ---- admin login -----------------------------------------------------

    pub async fn add_login_token(&self, hash: &str, expires_at: i64) -> Result<()> {
        sqlx::query("INSERT INTO login_tokens (hash, expires_at) VALUES (?, ?)")
            .bind(hash)
            .bind(expires_at)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Use a token: true once, if it exists and hasn't expired.
    pub async fn take_login_token(&self, hash: &str) -> Result<bool> {
        let n = sqlx::query(
            "UPDATE login_tokens SET used = 1 WHERE hash = ? AND used = 0 AND expires_at > ?",
        )
        .bind(hash)
        .bind(now())
        .execute(&self.pool)
        .await?
        .rows_affected();
        Ok(n == 1)
    }

    pub async fn add_session(&self, hash: &str, expires_at: i64) -> Result<()> {
        sqlx::query("INSERT INTO sessions (hash, created_at, expires_at) VALUES (?, ?, ?)")
            .bind(hash)
            .bind(now())
            .bind(expires_at)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn session_valid(&self, hash: &str) -> Result<bool> {
        let n: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM sessions WHERE hash = ? AND expires_at > ?")
                .bind(hash)
                .bind(now())
                .fetch_one(&self.pool)
                .await?;
        Ok(n == 1)
    }

    pub async fn end_session(&self, hash: &str) -> Result<()> {
        sqlx::query("DELETE FROM sessions WHERE hash = ?")
            .bind(hash)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Drop expired tokens and sessions.
    pub async fn prune(&self) -> Result<()> {
        let t = now();
        sqlx::query("DELETE FROM login_tokens WHERE expires_at <= ? OR used = 1")
            .bind(t)
            .execute(&self.pool)
            .await?;
        sqlx::query("DELETE FROM sessions WHERE expires_at <= ?")
            .bind(t)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

fn to_json<T: Serialize>(v: &[T]) -> Result<Vec<String>> {
    v.iter().map(|x| Ok(serde_json::to_string(x)?)).collect()
}

async fn set_setting(conn: &mut sqlx::SqliteConnection, key: &str, value: &str) -> Result<()> {
    sqlx::query("INSERT INTO settings (key, value) VALUES (?, ?) ON CONFLICT (key) DO UPDATE SET value = excluded.value")
        .bind(key)
        .bind(value)
        .execute(conn)
        .await?;
    Ok(())
}

fn message(r: &sqlx::sqlite::SqliteRow) -> Message {
    Message {
        id: r.get("id"),
        created_at: r.get("created_at"),
        name: r.get("name"),
        email: r.get("email"),
        project_type: r.get("project_type"),
        budget: r.get("budget"),
        body: r.get("body"),
        status: r.get("status"),
        replies: r.get("replies"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use content::owned::{Image, Skill};

    fn project(slug: &str) -> Project {
        Project {
            slug: slug.into(),
            title: slug.to_uppercase(),
            year: 2026,
            client: String::new(),
            role: "Dev".into(),
            categories: vec!["web".into()],
            tools: vec![],
            featured: true,
            accent: [1, 2, 3],
            cover: Image {
                src: "/media/x.jpg".into(),
                width: 1600,
                height: 1000,
                alt: String::new(),
            },
            cover_asset: "media/x.jpg".into(),
            preview_video: None,
            summary: "s".into(),
            body_md: "b".into(),
            body_html: "<p>b</p>".into(),
            gallery: vec![],
            links: vec![],
            credits: vec![],
            example: false,
        }
    }

    fn seed() -> Catalog {
        Catalog {
            projects: vec![project("one"), project("two")],
            roles: vec![Role {
                title: "Engineer".into(),
                org: "Acme".into(),
                kind: "job".into(),
                start: "2024".into(),
                end: String::new(),
                location: "Bloemfontein".into(),
                summary: String::new(),
                placeholder: false,
            }],
            cv: Some("/cv.pdf".into()),
            skills: vec![SkillGroup {
                name: "Rust".into(),
                blurb: String::new(),
                skills: vec![Skill {
                    name: "Leptos".into(),
                    level: "daily".into(),
                    note: String::new(),
                }],
                placeholder: false,
            }],
        }
    }

    #[tokio::test]
    async fn seeds_once_and_reads_back() {
        let s = Store::open(":memory:").await.unwrap();
        assert!(s.seed_if_empty(&seed()).await.unwrap());
        assert!(!s.seed_if_empty(&Catalog::default()).await.unwrap());
        assert_eq!(s.catalog().await.unwrap(), seed());
    }

    #[tokio::test]
    async fn projects_insert_first_reject_duplicate_slugs_and_reorder() {
        let s = Store::open(":memory:").await.unwrap();
        s.seed_if_empty(&seed()).await.unwrap();
        let id = s.save_project(None, &project("three")).await.unwrap();
        let slugs = |c: Catalog| c.projects.into_iter().map(|p| p.slug).collect::<Vec<_>>();
        assert_eq!(slugs(s.catalog().await.unwrap()), ["three", "one", "two"]);
        assert!(matches!(
            s.save_project(None, &project("one")).await,
            Err(Error::SlugTaken(_))
        ));
        // Saving a project under its own slug is fine.
        s.save_project(Some(id), &project("three")).await.unwrap();
        s.move_item(Collection::Projects, id, false).await.unwrap();
        assert_eq!(slugs(s.catalog().await.unwrap()), ["one", "three", "two"]);
        s.delete(Collection::Projects, id).await.unwrap();
        assert_eq!(slugs(s.catalog().await.unwrap()), ["one", "two"]);
    }

    #[tokio::test]
    async fn messages_and_replies() {
        let s = Store::open(":memory:").await.unwrap();
        let id = s
            .add_message("Ann", "ann@example.com", "Website", "R10k", "Hi")
            .await
            .unwrap();
        assert_eq!(s.count_new().await.unwrap(), 1);
        s.add_reply(id, "Failed one", "<a@x>", Some("SMTP down"))
            .await
            .unwrap();
        assert_eq!(s.message(id).await.unwrap().status, "new");
        s.add_reply(id, "Thanks!", "<b@x>", None).await.unwrap();
        let m = s.message(id).await.unwrap();
        assert_eq!((m.status.as_str(), m.replies), ("replied", 2));
        assert_eq!(s.replies(id).await.unwrap()[1].body, "Thanks!");
        s.set_status(id, "archived").await.unwrap();
        assert!(s.messages(None).await.unwrap().is_empty());
        assert_eq!(s.messages(Some("archived")).await.unwrap().len(), 1);
        s.delete_message(id).await.unwrap();
        assert!(s.replies(id).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn login_tokens_are_single_use_and_expire() {
        let s = Store::open(":memory:").await.unwrap();
        s.add_login_token("h1", now() + 600).await.unwrap();
        s.add_login_token("h2", now() - 1).await.unwrap();
        assert!(s.take_login_token("h1").await.unwrap());
        assert!(!s.take_login_token("h1").await.unwrap());
        assert!(!s.take_login_token("h2").await.unwrap());
        s.add_session("s1", now() + 600).await.unwrap();
        assert!(s.session_valid("s1").await.unwrap());
        s.end_session("s1").await.unwrap();
        assert!(!s.session_valid("s1").await.unwrap());
    }
}
