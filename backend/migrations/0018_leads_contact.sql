CREATE TABLE leads (
    id          UUID PRIMARY KEY,
    email       CITEXT NOT NULL,
    source      TEXT NOT NULL DEFAULT 'free-guide',
    ip          INET,
    user_agent  TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX leads_created_at_idx ON leads(created_at DESC);
CREATE INDEX leads_email_idx      ON leads(email);

CREATE TABLE contact_messages (
    id          UUID PRIMARY KEY,
    name        TEXT NOT NULL,
    email       CITEXT NOT NULL,
    subject     TEXT NOT NULL,
    body        TEXT NOT NULL,
    ip          INET,
    status      TEXT NOT NULL DEFAULT 'new' CHECK (status IN ('new','read','archived','spam')),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX contact_messages_status_created_idx ON contact_messages(status, created_at DESC);
