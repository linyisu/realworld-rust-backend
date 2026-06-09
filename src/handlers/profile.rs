use crate::{
    app_error::AppError,
    dto::profile::{FollowResponse, GetResponse, ProfileResponse, UnfollowResponse},
    middleware::auth::{AuthUser, OptionalAuth},
    models::{follows, users},
    utils::follow::is_following,
};

use axum::{extract::Path, extract::State};
use sea_orm::{ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

pub async fn get_profile(
    State(db): State<DatabaseConnection>,
    Path(username): Path<String>,
    OptionalAuth(auth_user): OptionalAuth,
) -> Result<axum::Json<GetResponse>, AppError> {
    println!("get_profile: username={}, auth_user={:?}", username, auth_user.as_ref().map(|u| u.user_id));

    let user = users::Entity::find()
        .filter(users::Column::Username.eq(&username))
        .one(&db)
        .await?
        .ok_or(AppError::NotFound)?;

    let following = if let Some(auth) = auth_user {
        let is_follow = is_following(&db, auth.user_id, user.id).await?;
        println!("is_following: follower_id={}, followee_id={}, result={}", auth.user_id, user.id, is_follow);
        is_follow
    } else {
        println!("No auth user, following=false");
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

    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");

    let new_follow = follows::ActiveModel {
        follower_id: Set(auth_user.user_id),
        followee_id: Set(user.id),
        created_at: Set(now.to_string()),
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
