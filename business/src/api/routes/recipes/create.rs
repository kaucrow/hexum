use platform::api::extractors::AuthenticatedUser;

use crate::{
    prelude::*,
    api::*,
    features::recipe::CreateRecipeInput,
};
use super::dtos::*;

#[utoipa::path(
    post,
    path = "/recipes",
    request_body(content = CreateRecipeRequest, content_type = "multipart/form-data"),
    responses(
        (status = 201, description = "Recipe created successfully", body = CreateRecipeResponse),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error"),
    ),
    security(("cookie_auth" = [])),
    tags = ["Recipes"]
)]
pub async fn create(
    auth: AuthenticatedUser,
    State(state): State<BusinessState>,
    multipart: Multipart,
) -> Result<impl IntoResponse, ApiError> {
    let user_id = auth.user_id;

    info!("Recipe creation requested for user '{}'", user_id);

    // ─── Parse & validate multipart fields ───
    let form = ParsedRecipeForm::from_multipart(multipart).await?;

    // ─── Handle image upload ───
    let thumbnail_url: Option<String> = if let Some((bytes, content_type)) = form.image {
        let ext = mime_to_extension(&content_type);
        let filename = format!("{}.{}", Uuid::new_v4(), ext);
        let upload_path = PathBuf::from(&state.config.storage.upload_dir).join(&filename);

        // Ensure the upload directory exists
        if let Some(parent) = upload_path.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                error!("Failed to create upload directory: {e}");
                ApiError::Internal("Failed to save image".to_string())
            })?;
        }

        // Write the file
        tokio::fs::write(&upload_path, &bytes).await.map_err(|e| {
            error!("Failed to write image file: {e}");
            ApiError::Internal("Failed to save image".to_string())
        })?;

        info!("Image saved to {:?}", upload_path);

        Some(format!("/uploads/{}", filename))
    } else {
        None
    };

    // ─── Create the recipe ───
    let input = CreateRecipeInput {
        name: form.name,
        description: form.description,
        tags: form.tags,
        ingredients: form.ingredients,
        instructions: form.instructions,
        thumbnail_url,
        created_by: user_id,
    };

    let recipe = state.recipe.create_recipe(input).await?;

    let response = CreateRecipeResponse::from(recipe);

    Ok((StatusCode::CREATED, Json(response)))
}

/// Maps a MIME content type to a file extension string.
fn mime_to_extension(mime: &str) -> &'static str {
    match mime {
        "image/jpeg" | "image/jpg" => "jpg",
        "image/png" => "png",
        "image/gif" => "gif",
        "image/webp" => "webp",
        "image/svg+xml" => "svg",
        "image/bmp" => "bmp",
        _ => "bin", // fallback for unknown types
    }
}

struct ParsedRecipeForm {
    pub name: String,
    pub description: Option<String>,
    pub instructions: String,
    pub tags: Vec<String>,
    pub ingredients: BTreeMap<String, String>,
    pub image: Option<(Bytes, String)>,
}

impl ParsedRecipeForm {
    // Limits media payloads to 20mb to avoid memory exhaustion
    const MAX_FILE_SIZE: usize = 20 * 1024 * 1024;

    pub async fn from_multipart(mut multipart: Multipart) -> Result<Self, ApiError> {
        let mut name: Option<String> = None;
        let mut description: Option<String> = None;
        let mut instructions: Option<String> = None;
        let mut tags_json: Option<String> = None;
        let mut ingredients_json: Option<String> = None;
        let mut image_data: Option<(Bytes, String)> = None;

        while let Some(field) = multipart.next_field().await.map_err(|e| {
            ApiError::BadRequest(format!("Multipart processing stream failed: {e}"))
        })? {
            let field_name: String = field.name().map(|s| s.to_string()).unwrap_or_default();

            match field_name.as_str() {
                "name" => {
                    name = Some(field.text().await.map_err(|e| {
                        ApiError::BadRequest(format!("Failed to read 'name' field: {e}"))
                    })?);
                }
                "description" => {
                    description = Some(field.text().await.map_err(|e| {
                        ApiError::BadRequest(format!("Failed to read 'description' field: {e}"))
                    })?);
                }
                "instructions" => {
                    instructions = Some(field.text().await.map_err(|e| {
                        ApiError::BadRequest(format!("Failed to read 'instructions' field: {e}"))
                    })?);
                }
                "tags" => {
                    tags_json = Some(field.text().await.map_err(|e| {
                        ApiError::BadRequest(format!("Failed to read 'tags' field: {e}"))
                    })?);
                }
                "ingredients" => {
                    ingredients_json = Some(field.text().await.map_err(|e| {
                        ApiError::BadRequest(format!("Failed to read 'ingredients' field: {e}"))
                    })?);
                }
                "image" => {
                    let content_type: String = field
                        .content_type()
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| "application/octet-stream".to_string());
                    let bytes = field.bytes().await.map_err(|e| {
                        ApiError::BadRequest(format!("Failed to read image data: {e}"))
                    })?;

                    if bytes.len() > Self::MAX_FILE_SIZE {
                        return Err(ApiError::BadRequest("Image payload size limit exceeded (10MB Max)".to_string()));
                    }

                    if !bytes.is_empty() {
                        image_data = Some((bytes, content_type));
                    }
                }
                _ => {
                    warn!("Unexpected field in multipart form: {field_name}");
                }
            }
        }

        let name = name.ok_or_else(|| ApiError::BadRequest("Missing required field: 'name'".to_string()))?;
        let instructions = instructions.ok_or_else(|| {
            ApiError::BadRequest("Missing required field: 'instructions'".to_string())
        })?;

        // Parse tags JSON (default to empty array if not provided)
        let tags: Vec<String> = match tags_json {
            Some(json_str) => serde_json::from_str(&json_str).map_err(|e| {
                ApiError::BadRequest(format!("Invalid JSON for 'tags': {e}"))
            })?,
            None => Vec::new(),
        };

        // Parse ingredients JSON (default to empty map if not provided)
        let ingredients: BTreeMap<String, String> = match ingredients_json {
            Some(json_str) => serde_json::from_str(&json_str).map_err(|e| {
                ApiError::BadRequest(format!("Invalid JSON for 'ingredients': {e}"))
            })?,
            None => BTreeMap::new(),
        };

        Ok(Self {
            name,
            description,
            instructions,
            tags,
            ingredients,
            image: image_data,
        })
    }
}