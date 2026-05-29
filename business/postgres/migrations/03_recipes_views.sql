CREATE OR REPLACE VIEW recipe.v_all_tags AS
SELECT DISTINCT
    TRIM(tag_name) AS tag_name
FROM recipe.recipe_tag
WHERE tag_name IS NOT NULL AND TRIM(tag_name) != ''
ORDER BY tag_name ASC;