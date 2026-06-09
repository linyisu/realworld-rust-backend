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

use axum::{Json, extract::State, http::StatusCode};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};

fn empty_string_to_none(s: Option<String>) -> Option<String> {
    s.and_then(|s| if s.is_empty() { None } else { Some(s) })
}

pub async fn signup(
    State(db): State<DatabaseConnection>,
    Json(payload): Json<SignupRequest>,
) -> Result<(StatusCode, Json<SignupResponse>), AppError> {
    if payload.user.username.trim().is_empty() {
        return Err(AppError::ValidationError(
            "username".to_string(),
            "can't be blank".to_string(),
        ));
    }

    if payload.user.email.trim().is_empty() {
        return Err(AppError::ValidationError(
            "email".to_string(),
            "can't be blank".to_string(),
        ));
    }

    if payload.user.password.trim().is_empty() {
        return Err(AppError::ValidationError(
            "password".to_string(),
            "can't be blank".to_string(),
        ));
    }

    let existing_user = users::Entity::find()
        .filter(
            users::Column::Username
                .eq(&payload.user.username)
                .or(users::Column::Email.eq(&payload.user.email)),
        )
        .one(&db)
        .await?;

    if let Some(user) = existing_user {
        if user.username == payload.user.username {
            return Err(AppError::Conflict(
                "username".to_string(),
                "has already been taken".to_string(),
            ));
        }
        if user.email == payload.user.email {
            return Err(AppError::Conflict(
                "email".to_string(),
                "has already been taken".to_string(),
            ));
        }
    }

    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    let password_hash = hash_password(&payload.user.password)?;

    let new_user = users::ActiveModel {
        email: Set(payload.user.email),
        username: Set(payload.user.username),
        password_hash: Set(password_hash),
        created_at: Set(now.to_string()),
        updated_at: Set(now.to_string()),
        ..Default::default()
    };

    let res = users::Entity::insert(new_user).exec(&db).await?;

    let user = users::Entity::find_by_id(res.last_insert_id)
        .one(&db)
        .await?
        .ok_or(AppError::NotFound)?;

    let token = generate_token(&user)?;

    Ok((
        StatusCode::CREATED,
        Json(SignupResponse {
            user: UserResponse {
                email: user.email,
                token,
                username: user.username,
                bio: empty_string_to_none(user.bio),
                image: empty_string_to_none(user.image),
            },
        }),
    ))
}

pub async fn login(
    State(db): State<DatabaseConnection>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    if payload.user.email.trim().is_empty() {
        return Err(AppError::ValidationError(
            "email".to_string(),
            "can't be blank".to_string(),
        ));
    }

    if payload.user.password.trim().is_empty() {
        return Err(AppError::ValidationError(
            "password".to_string(),
            "can't be blank".to_string(),
        ));
    }

    let user = users::Entity::find()
        .filter(users::Column::Email.eq(&payload.user.email))
        .one(&db)
        .await?
        .ok_or(AppError::InvalidCredentials)?;

    let valid = verify_password(&payload.user.password, &user.password_hash)?;

    if !valid {
        return Err(AppError::InvalidCredentials);
    }

    let token = generate_token(&user)?;

    Ok(Json(LoginResponse {
        user: UserResponse {
            email: user.email,
            token,
            username: user.username,
            bio: empty_string_to_none(user.bio),
            image: empty_string_to_none(user.image),
        },
    }))
}

pub async fn get_user(
    State(db): State<DatabaseConnection>,
    auth_user: AuthUser,
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
            bio: empty_string_to_none(user.bio),
            image: empty_string_to_none(user.image),
        },
    }))
}

pub async fn update_user(
    State(db): State<DatabaseConnection>,
    auth_user: AuthUser,
    body: String,
) -> Result<Json<UpdateResponse>, AppError> {
    if body.contains("\"email\":null") || body.contains("\"email\": null") {
        return Err(AppError::ValidationError(
            "email".to_string(),
            "cannot be null".to_string(),
        ));
    }

    if body.contains("\"username\":null") || body.contains("\"username\": null") {
        return Err(AppError::ValidationError(
            "username".to_string(),
            "cannot be null".to_string(),
        ));
    }

    if body.contains("\"password\":null") || body.contains("\"password\": null") {
        return Err(AppError::ValidationError(
            "password".to_string(),
            "cannot be null".to_string(),
        ));
    }

    let payload: UpdateRequest = serde_json::from_str(&body).map_err(|_| AppError::BadRequest)?;

    if let Some(ref email) = payload.user.email {
        if email.trim().is_empty() {
            return Err(AppError::ValidationError(
                "email".to_string(),
                "can't be blank".to_string(),
            ));
        }
    }

    if let Some(ref username) = payload.user.username {
        if username.trim().is_empty() {
            return Err(AppError::ValidationError(
                "username".to_string(),
                "can't be blank".to_string(),
            ));
        }
    }

    if let Some(ref password) = payload.user.password {
        if password.trim().is_empty() {
            return Err(AppError::ValidationError(
                "password".to_string(),
                "can't be blank".to_string(),
            ));
        }
        if password.len() < 8 {
            return Err(AppError::ValidationError(
                "password".to_string(),
                "is too short (minimum is 8 characters)".to_string(),
            ));
        }
    }

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

    if let Some(bio_opt) = payload.user.bio {
        active_user.bio = Set(bio_opt);
    }

    if let Some(image_opt) = payload.user.image {
        active_user.image = Set(image_opt);
    }

    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    active_user.updated_at = Set(now.to_string());

    let updated_user = active_user.update(&db).await?;

    let token = generate_token(&updated_user)?;

    Ok(Json(UpdateResponse {
        user: UserResponse {
            email: updated_user.email,
            token,
            username: updated_user.username,
            bio: empty_string_to_none(updated_user.bio),
            image: empty_string_to_none(updated_user.image),
        },
    }))
}
