-- Groups: invite-only rooms with scoped balances
CREATE TABLE IF NOT EXISTS groups (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name        VARCHAR(200) NOT NULL,
    invite_code VARCHAR(16) UNIQUE NOT NULL,
    owner_id    UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Membership — balance is per-group, per-user
CREATE TABLE IF NOT EXISTS group_members (
    group_id   UUID REFERENCES groups(id) ON DELETE CASCADE,
    user_id    UUID REFERENCES users(id) ON DELETE CASCADE,
    balance    DOUBLE PRECISION NOT NULL DEFAULT 1000,
    joined_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (group_id, user_id)
);

-- Bets are now scoped to a group
ALTER TABLE bets ADD COLUMN IF NOT EXISTS group_id UUID REFERENCES groups(id);

-- Drop the global balance — it lives in group_members now
ALTER TABLE users DROP COLUMN IF EXISTS balance;
