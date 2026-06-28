CREATE TABLE IF NOT EXISTS company.company (
    id UUID PRIMARY KEY,
    external_id INT UNIQUE,
    company_name VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS game.game_company (
    game_id UUID NOT NULL REFERENCES game.game(id),
    company_id UUID NOT NULL REFERENCES company.company(id),
    role VARCHAR(20) NOT NULL CHECK (role IN ('developer', 'publisher')),
    PRIMARY KEY (game_id, company_id, role)
);

CREATE INDEX IF NOT EXISTS idx_game_company_company_id
ON game.game_company(company_id);

CREATE INDEX IF NOT EXISTS idx_game_company_game_id
ON game.game_company(game_id);