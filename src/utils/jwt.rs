use crate::{app_error::AppError, models::users};

use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub user_id: u32,
    pub exp: usize,
}

pub fn generate_token(user: &users::Model) -> Result<String, AppError> {
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| String::from("DEFAULT_SECRET_KEY"));

    let expiration = Utc::now()
        .checked_add_signed(Duration::days(7))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        user_id: user.id,
        exp: expiration,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| {
        eprintln!("JWT 生成失败: {}", e);
        AppError::InternalServerError
    })?;

    Ok(token)
}

pub fn verify_token(token: &str) -> Result<Claims, AppError> {
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| String::from("DEFAULT_SECRET_KEY"));

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| {
        eprintln!("JWT 验证失败: {}", e);
        AppError::Unauthorized
    })?;

    Ok(token_data.claims)
}
