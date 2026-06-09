use crate::{app_error::AppError, utils::jwt::verify_token};

use axum::{extract::FromRequestParts, http::request::Parts};

pub struct AuthUser {
    pub user_id: u32,
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or(AppError::Unauthorized)?;

        let token = auth_header
            .strip_prefix("Token ")
            .ok_or(AppError::Unauthorized)?;

        let claims = verify_token(token)?;

        Ok(AuthUser {
            user_id: claims.user_id,
        })
    }
}

pub struct OptionalAuth(pub Option<AuthUser>);

impl<S> FromRequestParts<S> for OptionalAuth
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        Ok(OptionalAuth(
            AuthUser::from_request_parts(parts, state).await.ok(),
        ))
    }
}
