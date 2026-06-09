use crate::handlers::{
    article::{create_article, delete_article, get_article, list_articles, update_article},
    auth::{get_user, login, signup, update_user},
    profile::{follow, get_profile, unfollow},
};

use axum::{
    Router,
    routing::{delete, get, post, put},
};
use sea_orm::DatabaseConnection;

pub fn routes() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/users/login", post(login))
        .route("/api/users", post(signup))
        .route("/api/user", get(get_user))
        .route("/api/user", put(update_user))
        .route("/api/profiles/{username}", get(get_profile))
        .route("/api/profiles/{username}/follow", post(follow))
        .route("/api/profiles/{username}/follow", delete(unfollow))
        .route("/api/articles", post(create_article))
        .route("/api/articles", get(list_articles))
        .route("/api/articles/{slug}", get(get_article))
        .route("/api/articles/{slug}", put(update_article))
        .route("/api/articles/{slug}", delete(delete_article))
}
