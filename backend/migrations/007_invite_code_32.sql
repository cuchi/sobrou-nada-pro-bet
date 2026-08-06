-- Widen invite codes to 32 chars (was 16, generator produces 8-char codes)
ALTER TABLE groups ALTER COLUMN invite_code TYPE VARCHAR(32);
