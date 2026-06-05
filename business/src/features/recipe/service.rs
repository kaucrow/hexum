use async_trait::async_trait;

use crate::prelude::*;
use super::*;

#[derive(Clone)]
pub struct Service {
    local_repo: Arc<dyn LocalRepository>,
    cache_repo: Arc<dyn CacheRepository>,
}

impl Service {
    pub fn new(
        local_repo: Arc<dyn LocalRepository>,
        cache_repo: Arc<dyn CacheRepository>,
    ) -> Self {
        Self { local_repo, cache_repo }
    }
}

#[async_trait]
impl UseCase for Service {
    async fn search_recipe(&self,
        query: Option<&str>,
        tags: Option<&[String]>,
        page: usize,
        limit: usize,
        search_id: Option<Uuid>,
    ) -> Result<SearchResultsPage, UseCaseError>
    {
        // Validate that at least one search parameter is provided
        if query.is_none() && tags.is_none() {
            return Err(UseCaseError::MissingSearchParams);
        }

        let safe_page = if page == 0 { 1 } else { page };
        let offset = (safe_page - 1) * limit;

        let (search_id, matching_ids) = match search_id {
            // ─── Brand New Search ───
            None => {
                let new_id = Uuid::new_v4();
                let search_cache_key = format!("search:{}", new_id);

                // Get all matching recipe IDs, sorted by similarity to the query
                let matching_ids = self.local_repo.get_recipe_search_ids(query, tags).await?;

                if !matching_ids.is_empty() {
                    // Set the recipe search cache
                    self.cache_repo.set_recipe_ids(&search_cache_key, &matching_ids, 300).await?;   // 5 minutes
                }

                (new_id, matching_ids)
            }

            // ─── Fetching an existing page jump ───
            Some(existing_id) => {
                let search_cache_key = format!("search:{}", existing_id);

                match self.cache_repo.get_recipe_ids(&search_cache_key).await? {
                    Some(ids) => (existing_id, ids),
                    None => {
                        // The search cache expired, set it again
                        let ids = self.local_repo.get_recipe_search_ids(query, tags).await?;
                        if !ids.is_empty() {
                            self.cache_repo.set_recipe_ids(&search_cache_key, &ids, 300).await?;
                        }
                        (existing_id, ids)
                    }
                }
            }
        };

        if matching_ids.is_empty() {
            return Ok(SearchResultsPage { items: Vec::new(), total_items: 0, search_id: Uuid::new_v4() });
        }

        let total_items = matching_ids.len();

        // Slice the matching_ids vector based on offset and limit
        let page_ids: Vec<Uuid> = matching_ids
            .into_iter()
            .skip(offset)
            .take(limit)
            .collect();

        // Get the actual recipe search result data
        let items = self.local_repo.get_recipe_previews_by_ids(&page_ids).await?;

        Ok(SearchResultsPage {
            items,
            total_items,
            search_id,
        })
    }

    async fn get_recipe_by_id(&self, id: &Uuid) -> Result<Option<Recipe>, UseCaseError> {
        let recipe = if let Some(recipe) = self.cache_repo.get_recipe(&id).await? {
            // ─── Cache Hit ───
            // Return the recipe data from cache
            Some(recipe)
        } else {
            // ─── Cache Miss ───
            // Search for the recipe in DB & return it
            let recipe = self.local_repo.get_recipe_by_id(id).await?;

            // If the recipe exists, set the recipe cache
            if let Some(data) = &recipe {
                self.cache_repo.set_recipe(&id, data, 1800).await?; // 30 min
            }

            recipe
        };

        if let Some(ref r) = recipe {
            let recipe_id = r.id;

            let cache_repo = self.cache_repo.clone(); 

            tokio::spawn(async move {
                if let Err(e) = cache_repo.track_recipe_views(&recipe_id).await {
                    error!("Failed to update recipe view count: {:?}", e);
                }
            });
        }

        Ok(recipe)
    }

    async fn get_popular_recipes(&self, limit: usize) -> Result<Vec<RecipePreview>, UseCaseError> {
        let cache_popular_recipes = self.cache_repo
            .get_yesterday_most_viewed_recipe_ids(limit).await?;

        if let Some(recipe_ids) = cache_popular_recipes {
            // ─── Popular Recipes Found in Cache ───
            // Hydrate full records from DB matching the recipes' IDs
            let mut popular_recipes = self.local_repo
                .get_recipe_previews_by_ids(&recipe_ids).await?;

            // Sort back into the original order
            popular_recipes.sort_by_key(|r| recipe_ids.iter().position(|&id| id == r.id));

            Ok(popular_recipes)
        } else {
            // ─── Fallback ───
            // If cache is empty (first day running or no traffic yesterday), get random recipes
            let random_recipes = self.local_repo.get_random_recipe_previews(limit).await?;

            Ok(random_recipes)
        }
    }

    async fn get_latest_recipes(&self, limit: usize) -> Result<Vec<RecipePreview>, UseCaseError> {
        let latest_recipes = self.local_repo.get_latest_recipe_previews(limit).await?;

        Ok(latest_recipes)
    }

    async fn get_search_tag_matches(&self, query: &str, limit: usize) -> Result<Vec<String>, UseCaseError> {
        let tag_matches = self.local_repo.get_tag_search_matches(query, limit).await?;

        Ok(tag_matches)
    }

    async fn get_top_tags_recipes(&self, tags_limit: usize, recipes_limit: usize) -> Result<Vec<TagRecipes>, UseCaseError> {
        let tag_names = self.local_repo.get_top_tag_names(tags_limit).await?;

        let mut tag_groups = Vec::with_capacity(tag_names.len());
        for tag_name in tag_names {
            let recipes = self.local_repo.get_recipe_previews_by_tag_name(&tag_name, recipes_limit).await?;
            tag_groups.push(TagRecipes { tag_name, recipes });
        }

        Ok(tag_groups)
    }

    async fn get_latest_recipe_history(&self, user_id: &Uuid, limit: usize, offset: usize) -> Result<Vec<RecipePreview>, UseCaseError> {
        let history = self.local_repo.get_latest_recipe_history(&user_id, limit, offset).await?;

        Ok(history)
    }

    async fn get_recipes_created_by_user(&self, user_id: &Uuid, limit: usize, offset: usize) -> Result<UserCreatedRecipesPage, UseCaseError> {
        let (items, total_items) = self.local_repo
            .get_recipe_previews_by_creator(user_id, limit, offset).await?;

        Ok(UserCreatedRecipesPage { items, total_items })
    }

    async fn create_recipe(&self, input: CreateRecipeInput) -> Result<Recipe, UseCaseError> {
        // ─── Validation ───
        if input.name.trim().is_empty() {
            return Err(UseCaseError::EmptyName);
        }
        if input.instructions.trim().is_empty() {
            return Err(UseCaseError::EmptyInstructions);
        }

        let recipe_id = Uuid::new_v4();

        let data = CreateRecipeData {
            id: recipe_id,
            name: input.name,
            description: input.description,
            tags: input.tags,
            ingredients: input.ingredients,
            instructions: input.instructions,
            thumbnail_url: input.thumbnail_url,
            created_by: input.created_by,
        };

        let recipe = self.local_repo.create_recipe(data).await?;

        Ok(recipe)
    }

    async fn delete_recipe(&self, id: &Uuid, user_id: &Uuid) -> Result<Option<Uuid>, UseCaseError> {
        let deleted_recipe_id = self.local_repo.delete_recipe(id, user_id).await?;

        Ok(deleted_recipe_id)
    }

    async fn record_recipe_history(&self, user_id: &Uuid, recipe_id: &Uuid) -> Result<(), UseCaseError> {
        self.local_repo.record_recipe_history(user_id, recipe_id).await?;

        Ok(())
    }
}

impl From<LocalRepositoryError> for UseCaseError {
    fn from(e: LocalRepositoryError) -> Self {
        Self::Internal(e.to_string())
    }
}

impl From<CacheRepositoryError> for UseCaseError {
    fn from(e: CacheRepositoryError) -> Self {
        Self::Internal(e.to_string())
    }
}