use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use sea_orm::DbErr;
use serde_json::json;

#[derive(Debug)]
pub enum AppError {
    BadRequest,          // 400 - 错误的请求
    Unauthorized,        // 401 - 未认证
    Forbidden,           // 403 - 拒绝访问
    NotFound,            // 404 - 未找到
    InternalServerError, // 500 - 服务器错误
    Database(DbErr),     // 500 - 数据库错误
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::BadRequest => (StatusCode::BAD_REQUEST, "Bad request"),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized"),
            AppError::Forbidden => (StatusCode::FORBIDDEN, "Forbidden"),
            AppError::NotFound => (StatusCode::NOT_FOUND, "Not found"),
            AppError::InternalServerError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
            AppError::Database(ref e) => {
                eprintln!("数据库错误: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error")
            }
        };

        let body = Json(json!({
            "errors": {
                "body": [message]
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
