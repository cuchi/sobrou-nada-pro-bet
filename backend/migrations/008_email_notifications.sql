-- Email notifications: per-user opt-in flag, locale for templating, and
-- idempotency timestamps so re-running resolve/sync never double-sends.
ALTER TABLE users
    ADD COLUMN IF NOT EXISTS email_notifications       BOOLEAN     NOT NULL DEFAULT TRUE,
    ADD COLUMN IF NOT EXISTS locale                    VARCHAR(10) NOT NULL DEFAULT 'en',
    ADD COLUMN IF NOT EXISTS new_events_notified_at    TIMESTAMPTZ NULL;

ALTER TABLE bets
    ADD COLUMN IF NOT EXISTS notified_at TIMESTAMPTZ NULL;

-- Helpful partial index for the events digest query: opted-in users
-- whose digest is stale (NULL or older than the last sync).
CREATE INDEX IF NOT EXISTS idx_users_email_digest
    ON users (new_events_notified_at)
    WHERE email_notifications = TRUE;

-- Partial index for the bet-notified query: pending bets that still need
-- notification.
CREATE INDEX IF NOT EXISTS idx_bets_pending_notify
    ON bets (notified_at)
    WHERE status IN ('won', 'lost') AND notified_at IS NULL;