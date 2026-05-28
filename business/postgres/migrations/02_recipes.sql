CREATE TABLE IF NOT EXISTS recipe.recipe (
    id UUID PRIMARY KEY,
    recipe_name VARCHAR(255) NOT NULL,
    recipe_description TEXT NOT NULL UNIQUE,
    instructions TEXT NOT NULL,
    thumbnail_url TEXT,
    video_url TEXT
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

CREATE INDEX IF NOT EXISTS idx_recipe_name_trgm
ON recipe.recipe
USING gin (recipe_name gin_trgm_ops);