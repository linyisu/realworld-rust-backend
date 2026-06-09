use crate::handlers::auth::{get_user, login, signup, update_user};

use axum::{
    Router,
    routing::{get, post, put},
};
use sea_orm::DatabaseConnection;

pub fn routes() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/users/login", post(login))
        .route("/api/users", post(signup))
        .route("/api/user", get(get_user))
        .route("/api/user", put(update_user))
}
