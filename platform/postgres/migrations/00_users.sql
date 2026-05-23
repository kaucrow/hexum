CREATE TABLE "user" (
    id UUID PRIMARY KEY,
    username VARCHAR(255) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    roles TEXT[] NOT NULL DEFAULT '{}',
    is_active BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE user_authenticator (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES "user"(id) ON DELETE CASCADE,
    -- "local", "google", "github"
    provider VARCHAR(50) NOT NULL,
    -- The unique ID from the OAuth provider or NULL for local
    provider_id VARCHAR(255),
    -- The hashed password or NULL for OAuth
    passwd TEXT,
    -- Email verified flag or NULL for OAuth
    is_verified BOOLEAN,

    -- Ensure a user can't have two accounts linked with the same provider
    UNIQUE(user_id, provider),
    -- Ensure a specific OAuth ID isn't linked to two different internal users
    UNIQUE(provider, provider_id)
);