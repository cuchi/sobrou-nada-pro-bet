-- Add Google OAuth support to users table
ALTER TABLE users
    ALTER COLUMN username DROP NOT NULL,
    ADD COLUMN IF NOT EXISTS email      VARCHAR(255),
    ADD COLUMN IF NOT EXISTS google_id  VARCHAR(255) UNIQUE,
    ADD COLUMN IF NOT EXISTS avatar_url TEXT;

-- Ensure email is populated for Google users
CREATE UNIQUE INDEX IF NOT EXISTS idx_users_email ON users(email) WHERE email IS NOT NULL;
