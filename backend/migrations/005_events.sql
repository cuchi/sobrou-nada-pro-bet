-- Real events from api-futebol.com.br
CREATE TABLE IF NOT EXISTS events (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    external_id   VARCHAR(100) UNIQUE NOT NULL,
    home_team     VARCHAR(200) NOT NULL,
    away_team     VARCHAR(200) NOT NULL,
    championship  VARCHAR(200) NOT NULL,
    start_time    TIMESTAMPTZ NOT NULL,
    status        VARCHAR(20) NOT NULL DEFAULT 'scheduled'
                    CHECK (status IN ('scheduled', 'live', 'finished', 'cancelled')),
    home_score    INT,
    away_score    INT,
    raw_data      JSONB,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Bet gains event reference and prediction
ALTER TABLE bets
    ADD COLUMN IF NOT EXISTS event_id   UUID REFERENCES events(id),
    ADD COLUMN IF NOT EXISTS prediction VARCHAR(20)
        CHECK (prediction IS NULL OR prediction IN ('home_win', 'away_win', 'draw'));
