use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use sea_orm::DbErr;
use serde_json::json;

#[derive(Debug)]
pub enum AppError {
    BadRequest,                      // 400 - 错误的请求
    Unauthorized,                    // 401 - 未认证
    InvalidCredentials,              // 401 - 凭据无效
    NotFound,                        // 404 - 未找到
    Conflict(String, String),        // 409 - 冲突 (字段名, 错误消息)
    InternalServerError,             // 500 - 服务器错误
    Database(DbErr),                 // 数据库错误
    ValidationError(String, String), // 422 - 验证错误 (字段名, 错误消息)

    // 特定错误类型
    TokenMissing,     // Token 缺失
    ProfileNotFound,  // Profile 未找到
    ArticleNotFound,  // Article 未找到
    ArticleForbidden, // Article 操作被禁止
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_key, message) = match self {
            AppError::BadRequest => (StatusCode::BAD_REQUEST, "body", "Bad request"),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "body", "Unauthorized"),
            AppError::InvalidCredentials => {
                let body = Json(json!({
                    "errors": {
                        "credentials": ["invalid"]
                    }
                }));
                return (StatusCode::UNAUTHORIZED, body).into_response();
            }
            AppError::NotFound => (StatusCode::NOT_FOUND, "body", "Not found"),
            AppError::InternalServerError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "body",
                "Internal server error",
            ),
            AppError::Database(ref e) => {
                eprintln!("数据库错误: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "body", "Database error")
            }
            AppError::ValidationError(ref field, ref msg) => {
                let body = Json(json!({
                    "errors": {
                        field: [msg]
                    }
                }));
                return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
            }
            AppError::Conflict(ref field, ref msg) => {
                let body = Json(json!({
                    "errors": {
                        field: [msg]
                    }
                }));
                return (StatusCode::CONFLICT, body).into_response();
            }
            AppError::TokenMissing => (StatusCode::UNAUTHORIZED, "token", "is missing"),
            AppError::ProfileNotFound => (StatusCode::NOT_FOUND, "profile", "not found"),
            AppError::ArticleNotFound => (StatusCode::NOT_FOUND, "article", "not found"),
            AppError::ArticleForbidden => (StatusCode::FORBIDDEN, "article", "forbidden"),
        };

        let body = Json(json!({
            "errors": {
                error_key: [message]
            }
        }));

        (status, body).into_response()
    }
}

impl From<DbErr> for AppError {
    fn from(e: DbErr) -> Self {
        AppError::Database(e)
    }
}
