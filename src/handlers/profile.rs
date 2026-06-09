use crate::{
    app_error::AppError,
    dto::profile::{FollowResponse, GetResponse, ProfileResponse, UnfollowResponse},
    middleware::auth::AuthUser,
    models::{follows, users},
};

use axum::{extract::Path, extract::State};
use sea_orm::{ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

pub async fn get_profile(
    State(db): State<DatabaseConnection>,
    auth_user: Option<AuthUser>,
    Path(username): Path<String>,
) -> Result<axum::Json<GetResponse>, AppError> {
    let user = users::Entity::find()
        .filter(users::Column::Username.eq(&username))
        .one(&db)
        .await?
        .ok_or(AppError::NotFound)?;

    let following = if let Some(auth) = auth_user {
        follows::Entity::find()
            .filter(follows::Column::FollowerId.eq(auth.user_id))
            .filter(follows::Column::FolloweeId.eq(user.id))
            .one(&db)
            .await?
            .is_some()
    } else {
        false
    };

    Ok(axum::Json(GetResponse {
        profile: ProfileResponse {
            username: user.username,
            bio: user.bio,
            image: user.image,
            following,
        },
    }))
}

pub async fn follow(
    State(db): State<DatabaseConnection>,
    auth_user: AuthUser,
    Path(username): Path<String>,
) -> Result<axum::Json<FollowResponse>, AppError> {
    let user = users::Entity::find()
        .filter(users::Column::Username.eq(&username))
        .one(&db)
        .await?
        .ok_or(AppError::NotFound)?;

    let new_follow = follows::ActiveModel {
        follower_id: Set(auth_user.user_id),
        followee_id: Set(user.id),
        ..Default::default()
    };

    follows::Entity::insert(new_follow).exec(&db).await?;

    Ok(axum::Json(FollowResponse {
        profile: ProfileResponse {
            username: user.username,
            bio: user.bio,
            image: user.image,
            following: true,
        },
    }))
}

pub async fn unfollow(
    State(db): State<DatabaseConnection>,
    auth_user: AuthUser,
    Path(username): Path<String>,
) -> Result<axum::Json<UnfollowResponse>, AppError> {
    let user = users::Entity::find()
        .filter(users::Column::Username.eq(&username))
        .one(&db)
        .await?
        .ok_or(AppError::NotFound)?;

    follows::Entity::delete_many()
        .filter(follows::Column::FollowerId.eq(auth_user.user_id))
        .filter(follows::Column::FolloweeId.eq(user.id))
        .exec(&db)
        .await?;

    Ok(axum::Json(UnfollowResponse {
        profile: ProfileResponse {
            username: user.username,
            bio: user.bio,
            image: user.image,
            following: false,
        },
    }))
}
