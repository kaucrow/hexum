CREATE TABLE IF NOT EXISTS recipe.recipe (
    id UUID PRIMARY KEY,
    external_id INT UNIQUE,
    recipe_name VARCHAR(255) NOT NULL,
    recipe_description TEXT,
    instructions TEXT NOT NULL,
    thumbnail_url TEXT,
    video_url TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID REFERENCES platform.user(id)
);

CREATE TABLE IF NOT EXISTS recipe.recipe_tag (
    id UUID PRIMARY KEY,
    recipe_id UUID REFERENCES recipe.recipe(id) ON DELETE CASCADE,
    tag_name VARCHAR(50) NOT NULL,
    UNIQUE(recipe_id, tag_name)
);

CREATE TABLE IF NOT EXISTS recipe.recipe_ingredient (
    id UUID PRIMARY KEY,
    recipe_id UUID REFERENCES recipe.recipe(id) ON DELETE CASCADE,
    ingredient_name VARCHAR(255) NOT NULL,
    measure VARCHAR(255) NOT NULL,
    UNIQUE(recipe_id, ingredient_name)
);

CREATE TABLE IF NOT EXISTS recipe.history (
    user_id UUID NOT NULL REFERENCES platform.user(id) ON DELETE CASCADE,
    recipe_id UUID NOT NULL REFERENCES recipe.recipe(id) ON DELETE CASCADE,
    viewed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (user_id, recipe_id)
);

CREATE INDEX IF NOT EXISTS idx_recipe_external_id
ON recipe.recipe (external_id);

CREATE INDEX IF NOT EXISTS idx_recipe_name_trgm
ON recipe.recipe
USING gin (recipe_name gin_trgm_ops);

CREATE INDEX idx_history_user_recent
ON recipe.history (user_id, viewed_at DESC);