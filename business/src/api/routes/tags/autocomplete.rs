use crate::{
    prelude::*,
    api::*,
    features::recipe,
};
use super::dtos::*;

#[utoipa::path(
    get,
    path = "/tags/autocomplete",
    description = "Returns the matching tags for a query, for autocompletion.",
    params(TagsAutocompleteQueryParams),
    responses(
        (status = 200, description = "Recipe search results", body = TagsAutocompleteResponse),
        (status = 500, description = "Internal Server Error")
    ),
    tags = ["Tags"]
)]
pub async fn autocomplete(
    State(recipe_service): State<Arc<dyn recipe::UseCase>>,
    ValidatedQuery(queries): ValidatedQuery<TagsAutocompleteQueryParams>,
) -> Result<Json<TagsAutocompleteResponse>, ApiError> {
    info!("Getting tag matches for query '{}'", queries.query);

    let search_result = recipe_service
        .get_search_tag_matches(&queries.query, queries.limit).await?;

    Ok(Json(TagsAutocompleteResponse(search_result)))
}