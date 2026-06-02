CREATE TABLE IF NOT EXISTS recipe.group (
    id UUID PRIMARY KEY,
    group_name VARCHAR(255) NOT NULL,
    group_description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID REFERENCES platform.user(id)
);

CREATE TABLE IF NOT EXISTS recipe.group_recipe (
    group_id UUID REFERENCES recipe.group(id) ON DELETE CASCADE,
    recipe_id UUID REFERENCES recipe.recipe(id) ON DELETE CASCADE,
    PRIMARY KEY (group_id, recipe_id)
);

CREATE INDEX IF NOT EXISTS idx_group_created_by
ON recipe.group (created_by);

CREATE INDEX IF NOT EXISTS idx_group_recipe_recipe_id
ON recipe.group_recipe (recipe_id);