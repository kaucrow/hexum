use crate::{
    prelude::*,
    api::*,
    api::routes::recipes::dtos::RecipePreviewItem,
};

#[derive(Deserialize, IntoParams, ToSchema, Validate)]
#[serde(rename_all = "snake_case")]
#[into_params(parameter_in = Path)]
pub struct GroupIdPathParams {
    /// The Group's ID (UUID).
    #[schema(format = "uuid", example = "05639468-710b-44fe-9fc7-372514e95c37")]
    #[validate(length(equal = 36))]
    pub group_id: String,
}

#[derive(Deserialize, IntoParams, ToSchema, Validate)]
#[serde(rename_all = "snake_case")]
#[into_params(parameter_in = Path)]
pub struct GroupRecipePathParams {
    /// The Group's ID (UUID).
    #[schema(format = "uuid", example = "05639468-710b-44fe-9fc7-372514e95c37")]
    #[validate(length(equal = 36))]
    pub group_id: String,

    /// The Recipe's ID (UUID).
    #[schema(format = "uuid", example = "f47ac10b-58cc-4372-a567-0e02b2c3d479")]
    #[validate(length(equal = 36))]
    pub recipe_id: String,
}

#[derive(Deserialize, IntoParams, ToSchema, Validate)]
#[serde(rename_all = "snake_case")]
#[into_params(parameter_in = Query)]
pub struct UserGroupsQueryParams {
    /// The max amount of groups to fetch.
    #[param(example = 10, minimum = 1)]
    #[validate(range(min = 1, max = 20))]
    pub groups_limit: usize,

    /// The max amount of recipes to fetch per group.
    #[param(example = 5, minimum = 1, maximum = 20)]
    #[validate(range(min = 1, max = 20))]
    pub recipes_limit: usize,
}

#[derive(Deserialize, IntoParams, ToSchema, Validate)]
#[serde(rename_all = "snake_case")]
#[into_params(parameter_in = Query)]
pub struct GroupRecipesQueryParams {
    /// The max amount of recipes to fetch.
    #[param(example = 10, minimum = 1)]
    #[validate(range(min = 1, max = 40))]
    pub recipes_limit: usize,

    /// The pagination offset.
    #[param(example = 0, minimum = 0)]
    #[validate(range(min = 0))]
    pub offset: usize,
}

#[derive(Serialize, ToSchema)]
pub struct RecipesGroupItem {
    /// The group's ID (UUID).
    #[schema(format = "uuid")]
    pub group_id: String,

    /// The group's name.
    pub group_name: String,

    /// The recipes within this group.
    pub recipes: Vec<RecipePreviewItem>,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(example = json!({
    "groups": [
        {
            "group_id": "05639468-710b-44fe-9fc7-372514e95c37",
            "group_name": "Favorites",
            "recipes": [
                {
                    "id": "05639468-710b-44fe-9fc7-372514e95c37",
                    "origin": "external",
                    "name": "Spaghetti Carbonara",
                    "tags": ["Pasta", "Italian"],
                    "thumbnailUrl": "https://www.themealdb.com/images/media/meals/llc9is1557421634.jpg",
                }
            ]
        }
    ]
}))]
pub struct UserGroupsResponse {
    pub groups: Vec<RecipesGroupItem>,
}

#[derive(Serialize, ToSchema)]
#[schema(example = json!({
    "recipes": [
        {
            "id": "05639468-710b-44fe-9fc7-372514e95c37",
            "origin": "external",
            "name": "Spaghetti Carbonara",
            "tags": ["Pasta", "Italian"],
            "thumbnailUrl": "https://www.themealdb.com/images/media/meals/llc9is1557421634.jpg",
        }
    ],
}))]
pub struct GroupRecipesResponse {
    pub recipes: Vec<RecipePreviewItem>,
}

#[derive(Deserialize, ToSchema, Validate)]
#[serde(rename_all = "snake_case")]
pub struct CreateGroupRequest {
    /// The group name.
    #[schema(example = "Favorites")]
    #[validate(length(min = 1, max = 255))]
    pub name: String,

    /// Optional description of the group.
    #[schema(example = "My favorite recipes")]
    #[validate(length(max = 1000))]
    pub description: Option<String>,
}