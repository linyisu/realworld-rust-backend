use axum::extract::{Path, State};
use sea_orm::{
    ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
};

use crate::{
    app_error::AppError,
    dto::{
        article::ArticleResponse,
        favorite::{FavoriteResponse, UnfavoriteResponse},
        profile::ProfileResponse,
    },
    middleware::auth::AuthUser,
    models::{article_tags, articles, favorites, tags, users},
    utils::{article::decode_slug, follow::is_following},
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
        .ok_or(AppError::ArticleNotFound)?;

    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);

    let new_favorite = favorites::ActiveModel {
        user_id: Set(auth_user.user_id),
        article_id: Set(article.id),
        created_at: Set(now),
    };

    favorites::Entity::insert(new_favorite).exec(&db).await?;

    // 获取文章标签
    let article_tag_records = article_tags::Entity::find()
        .filter(article_tags::Column::ArticleId.eq(article.id))
        .all(&db)
        .await?;

    let mut tag_list = Vec::new();
    for at in article_tag_records {
        if let Some(tag) = tags::Entity::find_by_id(at.tag_id).one(&db).await? {
            tag_list.push(tag.name);
        }
    }

    // 获取作者信息
    let author = users::Entity::find_by_id(article.author_id)
        .one(&db)
        .await?
        .ok_or(AppError::NotFound)?;

    let following = is_following(&db, auth_user.user_id, author.id).await?;

    // 获取收藏数
    let favorites_count = favorites::Entity::find()
        .filter(favorites::Column::ArticleId.eq(article.id))
        .count(&db)
        .await? as u32;

    Ok(axum::Json(FavoriteResponse {
        article: ArticleResponse {
            slug: article.slug,
            title: article.title,
            description: article.description,
            body: article.body,
            tag_list,
            created_at: article.created_at,
            updated_at: article.updated_at,
            favorited: true,
            favorites_count,
            author: ProfileResponse {
                username: author.username,
                bio: author.bio,
                image: author.image,
                following,
            },
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
        .ok_or(AppError::ArticleNotFound)?;

    favorites::Entity::delete_many()
        .filter(favorites::Column::UserId.eq(auth_user.user_id))
        .filter(favorites::Column::ArticleId.eq(article.id))
        .exec(&db)
        .await?;

    // 获取文章标签
    let article_tag_records = article_tags::Entity::find()
        .filter(article_tags::Column::ArticleId.eq(article.id))
        .all(&db)
        .await?;

    let mut tag_list = Vec::new();
    for at in article_tag_records {
        if let Some(tag) = tags::Entity::find_by_id(at.tag_id).one(&db).await? {
            tag_list.push(tag.name);
        }
    }

    // 获取作者信息
    let author = users::Entity::find_by_id(article.author_id)
        .one(&db)
        .await?
        .ok_or(AppError::NotFound)?;

    let following = is_following(&db, auth_user.user_id, author.id).await?;

    // 获取收藏数
    let favorites_count = favorites::Entity::find()
        .filter(favorites::Column::ArticleId.eq(article.id))
        .count(&db)
        .await? as u32;

    Ok(axum::Json(UnfavoriteResponse {
        article: ArticleResponse {
            slug: article.slug,
            title: article.title,
            description: article.description,
            body: article.body,
            tag_list,
            created_at: article.created_at,
            updated_at: article.updated_at,
            favorited: false,
            favorites_count,
            author: ProfileResponse {
                username: author.username,
                bio: author.bio,
                image: author.image,
                following,
            },
        },
    }))
}
