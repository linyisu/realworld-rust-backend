use crate::{
    app_error::AppError,
    dto::{
        article::{
            ArticleResponse, CreateRequest, CreateResponse, GetResponse, ListRequest, ListResponse,
            UpdateRequest, UpdateResponse,
        },
        profile::ProfileResponse,
    },
    middleware::auth::{AuthUser, OptionalAuth},
    models::{articles, favorites, users},
    utils::{
        article::{decode_slug, slugify},
        follow::is_following,
    },
};

use axum::extract::{Json, Path, Query, State};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait,
    PaginatorTrait, QueryFilter, QuerySelect,
};

pub async fn create_article(
    State(db): State<DatabaseConnection>,
    auth_user: AuthUser,
    Json(payload): Json<CreateRequest>,
) -> Result<axum::Json<CreateResponse>, AppError> {
    let slug = slugify(&payload.article.title);

    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");

    let new_article = articles::ActiveModel {
        slug: Set(slug.clone()),
        title: Set(payload.article.title.clone()),
        description: Set(payload.article.description.clone()),
        body: Set(payload.article.body.clone()),
        author_id: Set(auth_user.user_id),
        created_at: Set(now.to_string()),
        updated_at: Set(now.to_string()),
        ..Default::default()
    };

    let res = articles::Entity::insert(new_article).exec(&db).await?;

    let article = articles::Entity::find_by_id(res.last_insert_id)
        .one(&db)
        .await?
        .ok_or(AppError::NotFound)?;

    let author = users::Entity::find_by_id(auth_user.user_id)
        .one(&db)
        .await?
        .ok_or(AppError::NotFound)?;

    let following = is_following(&db, auth_user.user_id, author.id).await?;

    Ok(axum::Json(CreateResponse {
        article: ArticleResponse {
            slug: article.slug,
            title: article.title,
            description: article.description,
            body: article.body,
            tag_list: payload.article.tag_list.unwrap_or_default(),
            created_at: article.created_at,
            updated_at: article.updated_at,
            favorited: false,
            favorites_count: 0,
            author: ProfileResponse {
                username: author.username,
                bio: author.bio,
                image: author.image,
                following,
            },
        },
    }))
}

pub async fn update_article(
    State(db): State<DatabaseConnection>,
    auth_user: AuthUser,
    Path(slug): Path<String>,
    Json(payload): Json<UpdateRequest>,
) -> Result<axum::Json<UpdateResponse>, AppError> {
    let decoded_slug = decode_slug(&slug)?;

    let article = articles::Entity::find()
        .filter(articles::Column::Slug.eq(decoded_slug))
        .one(&db)
        .await?
        .ok_or(AppError::NotFound)?;

    if article.author_id != auth_user.user_id {
        return Err(AppError::Forbidden);
    }

    let mut active_article: articles::ActiveModel = article.into();

    if let Some(title) = payload.article.title {
        active_article.title = Set(title);
    }

    if let Some(description) = payload.article.description {
        active_article.description = Set(description);
    }

    if let Some(body) = payload.article.body {
        active_article.body = Set(body);
    }

    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
    active_article.updated_at = Set(now.to_string());

    let updated_article = active_article.update(&db).await?;

    let author = users::Entity::find_by_id(updated_article.author_id)
        .one(&db)
        .await?
        .ok_or(AppError::NotFound)?;

    let favorited = favorites::Entity::find()
        .filter(favorites::Column::UserId.eq(auth_user.user_id))
        .filter(favorites::Column::ArticleId.eq(updated_article.id))
        .one(&db)
        .await?
        .is_some();

    let favorites_count = favorites::Entity::find()
        .filter(favorites::Column::ArticleId.eq(updated_article.id))
        .count(&db)
        .await? as u32;

    Ok(Json(UpdateResponse {
        article: ArticleResponse {
            slug: updated_article.slug,
            title: updated_article.title,
            description: updated_article.description,
            body: updated_article.body,
            // NOTE tag_list
            tag_list: vec![],
            created_at: updated_article.created_at,
            updated_at: updated_article.updated_at,
            favorited,
            favorites_count,
            author: ProfileResponse {
                username: author.username,
                bio: author.bio,
                image: author.image,
                following: true,
            },
        },
    }))
}

pub async fn list_articles(
    State(db): State<DatabaseConnection>,
    Query(params): Query<ListRequest>,
    OptionalAuth(auth_user): OptionalAuth,
) -> Result<Json<ListResponse>, AppError> {
    let mut query = articles::Entity::find();

    if let Some(author) = params.author {
        let author_user = users::Entity::find()
            .filter(users::Column::Username.eq(&author))
            .one(&db)
            .await?;

        if let Some(user) = author_user {
            query = query.filter(articles::Column::AuthorId.eq(user.id));
        } else {
            return Ok(Json(ListResponse {
                articles: vec![],
                articles_count: 0,
            }));
        }
    }

    if let Some(favorited_by) = params.favorited {
        let favorited_user = users::Entity::find()
            .filter(users::Column::Username.eq(&favorited_by))
            .one(&db)
            .await?;

        if let Some(user) = favorited_user {
            let favorited_article_ids: Vec<u32> = favorites::Entity::find()
                .filter(favorites::Column::UserId.eq(user.id))
                .all(&db)
                .await?
                .into_iter()
                .map(|f| f.article_id)
                .collect();

            if favorited_article_ids.is_empty() {
                return Ok(Json(ListResponse {
                    articles: vec![],
                    articles_count: 0,
                }));
            }

            query = query.filter(articles::Column::Id.is_in(favorited_article_ids));
        } else {
            return Ok(Json(ListResponse {
                articles: vec![],
                articles_count: 0,
            }));
        }
    }

    // NOTE: select tag

    let limit = params.limit.unwrap_or(20);
    let offset = params.offset.unwrap_or(0);

    let articles_list = query.limit(limit).offset(offset).all(&db).await?;

    let total_count = articles_list.len();

    let mut articles_response = Vec::new();

    for article in articles_list {
        let author = users::Entity::find_by_id(article.author_id)
            .one(&db)
            .await?
            .ok_or(AppError::NotFound)?;

        let following = if let Some(auth) = auth_user.as_ref() {
            is_following(&db, auth.user_id, author.id).await?
        } else {
            false
        };

        let favorited = if let Some(auth) = auth_user.as_ref() {
            favorites::Entity::find()
                .filter(favorites::Column::UserId.eq(auth.user_id))
                .filter(favorites::Column::ArticleId.eq(article.id))
                .one(&db)
                .await?
                .is_some()
        } else {
            false
        };

        let favorites_count = favorites::Entity::find()
            .filter(favorites::Column::ArticleId.eq(article.id))
            .count(&db)
            .await? as u32;

        articles_response.push(ArticleResponse {
            slug: article.slug,
            title: article.title,
            description: article.description,
            body: article.body,
            tag_list: vec![],
            created_at: article.created_at,
            updated_at: article.updated_at,
            favorited,
            favorites_count,
            author: ProfileResponse {
                username: author.username,
                bio: author.bio,
                image: author.image,
                following,
            },
        });
    }

    Ok(Json(ListResponse {
        articles: articles_response,
        articles_count: total_count,
    }))
}

pub async fn get_article(
    State(db): State<DatabaseConnection>,
    Path(slug): Path<String>,
    OptionalAuth(auth_user): OptionalAuth,
) -> Result<Json<GetResponse>, AppError> {
    let decoded_slug = decode_slug(&slug)?;

    let article = articles::Entity::find()
        .filter(articles::Column::Slug.eq(decoded_slug))
        .one(&db)
        .await?
        .ok_or(AppError::NotFound)?;

    let author = users::Entity::find_by_id(article.author_id)
        .one(&db)
        .await?
        .ok_or(AppError::NotFound)?;

    let following = if let Some(auth) = auth_user.as_ref() {
        is_following(&db, auth.user_id, author.id).await?
    } else {
        false
    };

    let favorited = if let Some(auth) = auth_user.as_ref() {
        favorites::Entity::find()
            .filter(favorites::Column::UserId.eq(auth.user_id))
            .filter(favorites::Column::ArticleId.eq(article.id))
            .one(&db)
            .await?
            .is_some()
    } else {
        false
    };

    let favorites_count = favorites::Entity::find()
        .filter(favorites::Column::ArticleId.eq(article.id))
        .count(&db)
        .await? as u32;

    Ok(Json(GetResponse {
        article: ArticleResponse {
            slug: article.slug,
            title: article.title,
            description: article.description,
            body: article.body,
            tag_list: vec![],
            created_at: article.created_at,
            updated_at: article.updated_at,
            favorited,
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

pub async fn delete_article(
    State(db): State<DatabaseConnection>,
    auth_user: AuthUser,
    Path(slug): Path<String>,
) -> Result<Json<()>, AppError> {
    let decoded_slug = decode_slug(&slug)?;

    let article = articles::Entity::find()
        .filter(articles::Column::Slug.eq(decoded_slug))
        .one(&db)
        .await?
        .ok_or(AppError::NotFound)?;

    if article.author_id != auth_user.user_id {
        return Err(AppError::Forbidden);
    }

    articles::Entity::delete_by_id(article.id).exec(&db).await?;

    Ok(Json(()))
}
