use crate::{
    app_error::AppError,
    dto::auth::{
        GetResponse, LoginRequest, LoginResponse, SignupRequest, SignupResponse, UpdateRequest,
        UpdateResponse, UserResponse,
    },
    middleware::auth::AuthUser,
    models::users,
    utils::{
        jwt::generate_token,
        password::{hash_password, verify_password},
    },
};

use axum::{Json, extract::State};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};

pub async fn signup(
    State(db): State<DatabaseConnection>,
    Json(payload): Json<SignupRequest>,
) -> Result<Json<SignupResponse>, AppError> {
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
    let password_hash = hash_password(&payload.user.password)?;

    let new_user = users::ActiveModel {
        email: Set(payload.user.email),
        username: Set(payload.user.username),
        password_hash: Set(password_hash),
        create_at: Set(now.to_string()),
        update_at: Set(now.to_string()),
        ..Default::default()
    };

    let res = users::Entity::insert(new_user).exec(&db).await?;

    let user = users::Entity::find_by_id(res.last_insert_id)
        .one(&db)
        .await?
        .ok_or(AppError::NotFound)?;

    let token = generate_token(&user)?;

    Ok(Json(SignupResponse {
        user: UserResponse {
            email: user.email,
            token,
            username: user.username,
            bio: user.bio,
            image: user.image,
        },
    }))
}

pub async fn login(
    State(db): State<DatabaseConnection>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    let user = users::Entity::find()
        .filter(users::Column::Email.eq(&payload.user.email))
        .one(&db)
        .await?
        .ok_or(AppError::Unauthorized)?;

    let valid = verify_password(&payload.user.password, &user.password_hash)?;

    if !valid {
        return Err(AppError::Unauthorized);
    }

    let token = generate_token(&user)?;

    Ok(Json(LoginResponse {
        user: UserResponse {
            email: user.email,
            token,
            username: user.username,
            bio: user.bio,
            image: user.image,
        },
    }))
}

pub async fn get_user(
    auth_user: AuthUser,
    State(db): State<DatabaseConnection>,
) -> Result<Json<GetResponse>, AppError> {
    let user = users::Entity::find_by_id(auth_user.user_id)
        .one(&db)
        .await?
        .ok_or(AppError::NotFound)?;

    let token = generate_token(&user)?;

    Ok(Json(GetResponse {
        user: UserResponse {
            email: user.email,
            token,
            username: user.username,
            bio: user.bio,
            image: user.image,
        },
    }))
}

pub async fn update_user(
    auth_user: AuthUser,
    State(db): State<DatabaseConnection>,
    Json(payload): Json<UpdateRequest>,
) -> Result<Json<UpdateResponse>, AppError> {
    let user = users::Entity::find_by_id(auth_user.user_id)
        .one(&db)
        .await?
        .ok_or(AppError::NotFound)?;

    let mut active_user: users::ActiveModel = user.into();

    if let Some(email) = payload.user.email {
        active_user.email = Set(email);
    }

    if let Some(username) = payload.user.username {
        active_user.username = Set(username);
    }

    if let Some(password) = payload.user.password {
        let password_hash = hash_password(&password)?;
        active_user.password_hash = Set(password_hash);
    }

    if let Some(bio) = payload.user.bio {
        active_user.bio = Set(Some(bio));
    }

    if let Some(image) = payload.user.image {
        active_user.image = Set(Some(image));
    }

    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    active_user.update_at = Set(now);

    let updated_user = active_user.update(&db).await?;

    let token = generate_token(&updated_user)?;

    Ok(Json(UpdateResponse {
        user: UserResponse {
            email: updated_user.email,
            token,
            username: updated_user.username,
            bio: updated_user.bio,
            image: updated_user.image,
        },
    }))
}
