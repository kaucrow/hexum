CREATE TABLE IF NOT EXISTS platform.platform (
    id UUID PRIMARY KEY,
    external_id INT UNIQUE,
    platform_name VARCHAR(255) NOT NULL,
    generation INT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);