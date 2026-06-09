use axum::{Json, extract::State};
use sea_orm::{DatabaseConnection, EntityTrait};

use crate::{app_error::AppError, dto::tag::TagsResponse, models::tags};

pub async fn get_tags(
    State(db): State<DatabaseConnection>,
) -> Result<Json<TagsResponse>, AppError> {
    let all_tags = tags::Entity::find().all(&db).await?;

    let tag_names: Vec<String> = all_tags.into_iter().map(|tag| tag.name).collect();

    Ok(Json(TagsResponse { tags: tag_names }))
}
