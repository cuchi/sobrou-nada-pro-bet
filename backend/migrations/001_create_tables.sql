-- Migration: create_users
CREATE TABLE IF NOT EXISTS users (
    id         UUID PRIMARY KEY,
    username   VARCHAR(100) NOT NULL UNIQUE,
    balance    DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Migration: create_bets
CREATE TABLE IF NOT EXISTS bets (
    id         UUID PRIMARY KEY,
    user_id    UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    amount     DOUBLE PRECISION NOT NULL CHECK (amount > 0),
    odds       DOUBLE PRECISION NOT NULL CHECK (odds > 0),
    status     VARCHAR(20) NOT NULL DEFAULT 'pending'
                   CHECK (status IN ('pending', 'won', 'lost')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_bets_user_id ON bets(user_id);
CREATE INDEX IF NOT EXISTS idx_bets_status   ON bets(status);
