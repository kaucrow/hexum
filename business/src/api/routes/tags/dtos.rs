use crate::{
    prelude::*,
    api::*,
};

#[derive(Deserialize, IntoParams, ToSchema, Validate)]
#[into_params(parameter_in = Query)]
pub struct TagsAutocompleteQueryParams {
    /// The tag's name (partial or complete).
    #[param(example = "bea")]
    #[validate(length(max = 40))]
    pub query: String,

    /// The max amount of tags to fetch.
    #[param(example = 10, minimum = 1)]
    #[validate(range(min = 1, max = 40))]
    pub limit: usize,
}

#[derive(Serialize, ToSchema)]
#[schema(example = json!(["Pasta", "Italian", "Dessert", "Vegan"]))]
pub struct TagsAutocompleteResponse(pub Vec<String>);