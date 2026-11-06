-- Content the dashboard edits. Each row is one item as JSON (content::owned
-- types); `position` is the display order.
CREATE TABLE projects (
    id        INTEGER PRIMARY KEY,
    position  INTEGER NOT NULL,
    slug      TEXT NOT NULL UNIQUE,
    data      TEXT NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE roles (
    id        INTEGER PRIMARY KEY,
    position  INTEGER NOT NULL,
    data      TEXT NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE skill_groups (
    id        INTEGER PRIMARY KEY,
    position  INTEGER NOT NULL,
    data      TEXT NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- Contact form enquiries and the replies sent from the dashboard.
CREATE TABLE messages (
    id           INTEGER PRIMARY KEY,
    created_at   INTEGER NOT NULL,
    name         TEXT NOT NULL,
    email        TEXT NOT NULL,
    project_type TEXT NOT NULL,
    budget       TEXT NOT NULL,
    body         TEXT NOT NULL,
    -- new | read | replied | archived
    status       TEXT NOT NULL DEFAULT 'new'
);
CREATE INDEX messages_by_date ON messages (created_at DESC);

CREATE TABLE replies (
    id          INTEGER PRIMARY KEY,
    message_id  INTEGER NOT NULL REFERENCES messages (id) ON DELETE CASCADE,
    created_at  INTEGER NOT NULL,
    body        TEXT NOT NULL,
    -- The email's Message-ID, so later replies thread under it.
    email_id    TEXT NOT NULL,
    -- Set if delivery failed.
    error       TEXT
);
CREATE INDEX replies_by_message ON replies (message_id, created_at);

-- Admin login: single-use magic-link tokens and sessions, both stored as
-- SHA-256 hashes so a leaked database can't be used to sign in.
CREATE TABLE login_tokens (
    hash       TEXT PRIMARY KEY,
    expires_at INTEGER NOT NULL,
    used       INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE sessions (
    hash       TEXT PRIMARY KEY,
    created_at INTEGER NOT NULL,
    expires_at INTEGER NOT NULL
);
