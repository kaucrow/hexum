CREATE TABLE IF NOT EXISTS game.game (
    id UUID PRIMARY KEY,
    external_id INT UNIQUE,
    game_name VARCHAR(255) NOT NULL,
    first_release_date TIMESTAMPTZ,
    cover_url VARCHAR(512),
    rating DOUBLE PRECISION,
    critic_rating DOUBLE PRECISION,
    total_rating_count INT,
    summary TEXT,
    view_count BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_game_name_trgm
ON game.game
USING gin (game_name gin_trgm_ops);

CREATE INDEX IF NOT EXISTS idx_game_view_count ON game.game (view_count DESC);