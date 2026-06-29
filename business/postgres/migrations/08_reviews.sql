CREATE TABLE game.review (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES platform.user(id) ON DELETE CASCADE,
    game_id UUID NOT NULL REFERENCES game.game(id) ON DELETE CASCADE,
    review_type VARCHAR(10) NOT NULL CHECK (review_type IN ('user', 'critic')),
    rating INT NOT NULL CHECK (rating >= 0 AND rating <= 100),
    title VARCHAR(255) NOT NULL,
    content TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, game_id)
);

CREATE INDEX idx_review_game_id ON game.review (game_id);
CREATE INDEX idx_review_game_id_type ON game.review (game_id, review_type);