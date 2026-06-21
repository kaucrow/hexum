CREATE TABLE IF NOT EXISTS game.game_platform (
    game_id UUID NOT NULL REFERENCES game.game(id),
    platform_id UUID NOT NULL REFERENCES platform.platform(id),
    PRIMARY KEY (game_id, platform_id)
);

CREATE INDEX IF NOT EXISTS idx_game_platform_platform_id
ON game.game_platform(platform_id);

CREATE INDEX IF NOT EXISTS idx_game_platform_game_id
ON game.game_platform(game_id);