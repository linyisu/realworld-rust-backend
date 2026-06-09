use crate::{
    app_error::AppError,
    dto::{
        article::{
            ArticleListItemResponse, ArticleResponse, CreateRequest, CreateResponse, GetResponse,
            ListRequest, ListResponse, UpdateRequest, UpdateResponse,
        },
        profile::ProfileResponse,
    },
    middleware::auth::{AuthUser, OptionalAuth},
    models::{article_tags, articles, favorites, tags, users},
    utils::{
        article::{decode_slug, slugify},
        follow::is_following,
    },
};

use axum::extract::{Json, Path, Query, State};
use axum::http::StatusCode;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait,
    PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
};

pub async fn create_article(
    State(db): State<DatabaseConnection>,
    auth_user: AuthUser,
    Json(payload): Json<CreateRequest>,
) -> Result<(StatusCode, axum::Json<CreateResponse>), AppError> {
    if payload.article.title.trim().is_empty() {
        return Err(AppError::ValidationError(
            "title".to_string(),
            "can't be blank".to_string(),
        ));
    }

    if payload.article.description.trim().is_empty() {
        return Err(AppError::ValidationError(
            "description".to_string(),
            "can't be blank".to_string(),
        ));
    }

    if payload.article.body.trim().is_empty() {
        return Err(AppError::ValidationError(
            "body".to_string(),
            "can't be blank".to_string(),
        ));
    }

    let slug = slugify(&payload.article.title);

    let mut final_slug = slug.clone();
    let mut counter = 1;

    while articles::Entity::find()
        .filter(articles::Column::Slug.eq(&final_slug))
        .one(&db)
        .await?
        .is_some()
    {
        final_slug = format!("{}-{}", slug, counter);
        counter += 1;
    }

    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);

    let new_article = articles::ActiveModel {
        slug: Set(final_slug.clone()),
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

    let mut saved_tags = Vec::new();
    if let Some(tag_list) = payload.article.tag_list {
        for tag_name in tag_list {
            let tag = match tags::Entity::find()
                .filter(tags::Column::Name.eq(&tag_name))
                .one(&db)
                .await?
            {
                Some(t) => t,
                None => {
                    let new_tag = tags::ActiveModel {
                        name: Set(tag_name.clone()),
                        ..Default::default()
                    };
                    let tag_res = tags::Entity::insert(new_tag).exec(&db).await?;
                    tags::Entity::find_by_id(tag_res.last_insert_id)
                        .one(&db)
                        .await?
                        .ok_or(AppError::NotFound)?
                }
            };

            let article_tag = article_tags::ActiveModel {
                article_id: Set(article.id),
                tag_id: Set(tag.id),
            };
            article_tags::Entity::insert(article_tag).exec(&db).await?;

            saved_tags.push(tag_name);
        }
    }

    let author = users::Entity::find_by_id(auth_user.user_id)
        .one(&db)
        .await?
        .ok_or(AppError::NotFound)?;

    let following = is_following(&db, auth_user.user_id, author.id).await?;

    Ok((
        StatusCode::CREATED,
        axum::Json(CreateResponse {
            article: ArticleResponse {
                slug: article.slug,
                title: article.title,
                description: article.description,
                body: article.body,
                tag_list: saved_tags,
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
        }),
    ))
}

pub async fn update_article(
    State(db): State<DatabaseConnection>,
    auth_user: AuthUser,
    Path(slug): Path<String>,
    body: String,
) -> Result<axum::Json<UpdateResponse>, AppError> {
    if body.contains("\"tagList\":null") || body.contains("\"tagList\": null") {
        return Err(AppError::ValidationError(
            "tagList".to_string(),
            "cannot be null".to_string(),
        ));
    }

    let payload: UpdateRequest = serde_json::from_str(&body).map_err(|_| AppError::BadRequest)?;

    let decoded_slug = decode_slug(&slug)?;

    let article = articles::Entity::find()
        .filter(articles::Column::Slug.eq(decoded_slug))
        .one(&db)
        .await?
        .ok_or(AppError::ArticleNotFound)?;

    if article.author_id != auth_user.user_id {
        return Err(AppError::ArticleForbidden);
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

    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    active_article.updated_at = Set(now.to_string());

    let updated_article = active_article.update(&db).await?;

    if let Some(tag_list) = payload.article.tag_list {
        article_tags::Entity::delete_many()
            .filter(article_tags::Column::ArticleId.eq(updated_article.id))
            .exec(&db)
            .await?;

        for tag_name in tag_list {
            let tag = match tags::Entity::find()
                .filter(tags::Column::Name.eq(&tag_name))
                .one(&db)
                .await?
            {
                Some(t) => t,
                None => {
                    let new_tag = tags::ActiveModel {
                        name: Set(tag_name.clone()),
                        ..Default::default()
                    };
                    let tag_res = tags::Entity::insert(new_tag).exec(&db).await?;
                    tags::Entity::find_by_id(tag_res.last_insert_id)
                        .one(&db)
                        .await?
                        .ok_or(AppError::NotFound)?
                }
            };

            let article_tag = article_tags::ActiveModel {
                article_id: Set(updated_article.id),
                tag_id: Set(tag.id),
            };
            article_tags::Entity::insert(article_tag).exec(&db).await?;
        }
    }

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

    let tag_list = get_article_tags(&db, updated_article.id).await?;

    Ok(Json(UpdateResponse {
        article: ArticleResponse {
            slug: updated_article.slug,
            title: updated_article.title,
            description: updated_article.description,
            body: updated_article.body,
            tag_list,
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

    if let Some(tag_name) = params.tag {
        let tag = tags::Entity::find()
            .filter(tags::Column::Name.eq(&tag_name))
            .one(&db)
            .await?;

        if let Some(t) = tag {
            let article_ids: Vec<u32> = article_tags::Entity::find()
                .filter(article_tags::Column::TagId.eq(t.id))
                .all(&db)
                .await?
                .into_iter()
                .map(|at| at.article_id)
                .collect();

            if article_ids.is_empty() {
                return Ok(Json(ListResponse {
                    articles: vec![],
                    articles_count: 0,
                }));
            }

            query = query.filter(articles::Column::Id.is_in(article_ids));
        } else {
            return Ok(Json(ListResponse {
                articles: vec![],
                articles_count: 0,
            }));
        }
    }

    let limit = params.limit.unwrap_or(20);
    let offset = params.offset.unwrap_or(0);

    let total_count = query.clone().count(&db).await? as usize;

    let articles_list = query
        .order_by_desc(articles::Column::CreatedAt)
        .limit(limit)
        .offset(offset)
        .all(&db)
        .await?;

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

        let tag_list = get_article_tags(&db, article.id).await?;

        articles_response.push(ArticleListItemResponse {
            slug: article.slug,
            title: article.title,
            description: article.description,
            tag_list,
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

pub async fn get_feed(
    State(db): State<DatabaseConnection>,
    auth_user: AuthUser,
    Query(params): Query<ListRequest>,
) -> Result<Json<ListResponse>, AppError> {
    let following_ids: Vec<u32> = crate::models::follows::Entity::find()
        .filter(crate::models::follows::Column::FollowerId.eq(auth_user.user_id))
        .all(&db)
        .await?
        .into_iter()
        .map(|f| f.followee_id)
        .collect();

    if following_ids.is_empty() {
        return Ok(Json(ListResponse {
            articles: vec![],
            articles_count: 0,
        }));
    }

    let limit = params.limit.unwrap_or(20);
    let offset = params.offset.unwrap_or(0);

    let query = articles::Entity::find().filter(articles::Column::AuthorId.is_in(following_ids));

    let total_count = query.clone().count(&db).await? as usize;

    let articles_list = query
        .order_by_desc(articles::Column::CreatedAt)
        .limit(limit)
        .offset(offset)
        .all(&db)
        .await?;

    let mut articles_response = Vec::new();

    for article in articles_list {
        let author = users::Entity::find_by_id(article.author_id)
            .one(&db)
            .await?
            .ok_or(AppError::NotFound)?;

        let following = is_following(&db, auth_user.user_id, author.id).await?;

        let favorited = favorites::Entity::find()
            .filter(favorites::Column::UserId.eq(auth_user.user_id))
            .filter(favorites::Column::ArticleId.eq(article.id))
            .one(&db)
            .await?
            .is_some();

        let favorites_count = favorites::Entity::find()
            .filter(favorites::Column::ArticleId.eq(article.id))
            .count(&db)
            .await? as u32;

        let tag_list = get_article_tags(&db, article.id).await?;

        articles_response.push(ArticleListItemResponse {
            slug: article.slug,
            title: article.title,
            description: article.description,
            tag_list,
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
        .ok_or(AppError::ArticleNotFound)?;

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

    let tag_list = get_article_tags(&db, article.id).await?;

    Ok(Json(GetResponse {
        article: ArticleResponse {
            slug: article.slug,
            title: article.title,
            description: article.description,
            body: article.body,
            tag_list,
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
) -> Result<StatusCode, AppError> {
    let decoded_slug = decode_slug(&slug)?;

    let article = articles::Entity::find()
        .filter(articles::Column::Slug.eq(decoded_slug))
        .one(&db)
        .await?
        .ok_or(AppError::ArticleNotFound)?;

    if article.author_id != auth_user.user_id {
        return Err(AppError::ArticleForbidden);
    }

    // 先删除关联的标签
    article_tags::Entity::delete_many()
        .filter(article_tags::Column::ArticleId.eq(article.id))
        .exec(&db)
        .await?;

    favorites::Entity::delete_many()
        .filter(favorites::Column::ArticleId.eq(article.id))
        .exec(&db)
        .await?;

    articles::Entity::delete_by_id(article.id).exec(&db).await?;

    Ok(StatusCode::NO_CONTENT)
}

async fn get_article_tags(
    db: &DatabaseConnection,
    article_id: u32,
) -> Result<Vec<String>, AppError> {
    let article_tag_records = article_tags::Entity::find()
        .filter(article_tags::Column::ArticleId.eq(article_id))
        .all(db)
        .await?;

    let mut tag_names = Vec::new();
    for article_tag in article_tag_records {
        if let Some(tag) = tags::Entity::find_by_id(article_tag.tag_id).one(db).await? {
            tag_names.push(tag.name);
        }
    }

    Ok(tag_names)
}
