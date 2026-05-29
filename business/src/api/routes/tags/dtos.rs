use crate::{
    prelude::*,
    api::*,
};

#[derive(Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct TagsAutocompleteQueryParams {
    /// The tag's name (partial or complete).
    #[param(example = "bea")]
    pub query: String,

    /// The max amount of tags to fetch.
    #[param(example = 10)]
    pub limit: usize,
}

#[derive(Serialize, ToSchema)]
#[schema(example = json!(["Pasta", "Italian", "Dessert", "Vegan"]))]
pub struct TagsAutocompleteResponse(pub Vec<String>);