CREATE TABLE IF NOT EXISTS game.game (
    id UUID PRIMARY KEY,
    external_id INT UNIQUE,
    game_name VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_game_external_id
ON game.game (external_id);

CREATE INDEX IF NOT EXISTS idx_game_name_trgm
ON game.game
USING gin (game_name gin_trgm_ops);