-- Allowlist for closed beta — only emails in this table can sign in
CREATE TABLE IF NOT EXISTS beta_allowlist (
    email      VARCHAR(255) PRIMARY KEY,
    added_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
