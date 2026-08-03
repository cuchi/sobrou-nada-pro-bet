-- Store odds from footballdata.io for auto-fill
ALTER TABLE events
    ADD COLUMN IF NOT EXISTS home_odds DOUBLE PRECISION,
    ADD COLUMN IF NOT EXISTS draw_odds DOUBLE PRECISION,
    ADD COLUMN IF NOT EXISTS away_odds DOUBLE PRECISION;
