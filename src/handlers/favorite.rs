use axum::extract::{Path, State};
use sea_orm::{
    ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
};

use crate::{
    app_error::AppError,
    dto::favorite::{ArticleResponse, FavoriteResponse, UnfavoriteResponse},
    middleware::auth::AuthUser,
    models::{articles, favorites},
    utils::article::decode_slug,
};

pub async fn favorite(
    State(db): State<DatabaseConnection>,
    Path(slug): Path<String>,
    auth_user: AuthUser,
) -> Result<axum::Json<FavoriteResponse>, AppError> {
    let decoded_slug = decode_slug(&slug)?;

    let article = articles::Entity::find()
        .filter(articles::Column::Slug.eq(decoded_slug))
        .one(&db)
        .await?
        .ok_or(AppError::NotFound)?;

    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");

    let new_favorite = favorites::ActiveModel {
        user_id: Set(auth_user.user_id),
        article_id: Set(article.id),
        created_at: Set(now.to_string()),
    };

    favorites::Entity::insert(new_favorite).exec(&db).await?;

    let favorites_count = favorites::Entity::find()
        .filter(favorites::Column::ArticleId.eq(article.id))
        .count(&db)
        .await? as u32;

    Ok(axum::Json(FavoriteResponse {
        article: ArticleResponse {
            favorited: true,
            favorites_count,
        },
    }))
}

pub async fn unfavorite(
    State(db): State<DatabaseConnection>,
    Path(slug): Path<String>,
    auth_user: AuthUser,
) -> Result<axum::Json<UnfavoriteResponse>, AppError> {
    let decoded_slug = decode_slug(&slug)?;

    let article = articles::Entity::find()
        .filter(articles::Column::Slug.eq(decoded_slug))
        .one(&db)
        .await?
        .ok_or(AppError::NotFound)?;

    favorites::Entity::delete_many()
        .filter(favorites::Column::UserId.eq(auth_user.user_id))
        .filter(favorites::Column::ArticleId.eq(article.id))
        .exec(&db)
        .await?;

    let favorites_count = favorites::Entity::find()
        .filter(favorites::Column::ArticleId.eq(article.id))
        .count(&db)
        .await? as u32;

    Ok(axum::Json(UnfavoriteResponse {
        article: ArticleResponse {
            favorited: false,
            favorites_count,
        },
    }))
}
