CREATE TABLE IF NOT EXISTS genre.genre (
    id UUID PRIMARY KEY,
    external_id INT UNIQUE,
    genre_name VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS game.game_genre (
    game_id UUID NOT NULL REFERENCES game.game(id),
    genre_id UUID NOT NULL REFERENCES genre.genre(id),
    PRIMARY KEY (game_id, genre_id)
);

CREATE INDEX IF NOT EXISTS idx_game_genre_genre_id
ON game.game_genre(genre_id);

CREATE INDEX IF NOT EXISTS idx_game_genre_game_id
ON game.game_genre(game_id);