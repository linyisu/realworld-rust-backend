mod app_error;
mod dto;
mod handlers;
mod middleware;
mod models;
mod route;
mod utils;

use route::routes;
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbErr, Schema};
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

use crate::models::{articles, favorites, follows, users};

async fn log_request(
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    println!("收到请求: {} {}", req.method(), req.uri());
    next.run(req).await
}

async fn setup(db: &DatabaseConnection) {
    let schema = Schema::new(sea_orm::DbBackend::Sqlite);

    let users_stmt = schema.create_table_from_entity(users::Entity);
    db.execute(db.get_database_backend().build(&users_stmt))
        .await
        .ok();

    let articles_stmt = schema.create_table_from_entity(articles::Entity);
    db.execute(db.get_database_backend().build(&articles_stmt))
        .await
        .ok();

    let follows_stmt = schema.create_table_from_entity(follows::Entity);
    db.execute(db.get_database_backend().build(&follows_stmt))
        .await
        .ok();

    let favorites_stmt = schema.create_table_from_entity(favorites::Entity);
    db.execute(db.get_database_backend().build(&favorites_stmt))
        .await
        .ok();
}

#[tokio::main]
async fn main() -> Result<(), DbErr> {
    dotenv::dotenv().ok();

    let db = Database::connect("sqlite://database.db?mode=rwc").await?;
    setup(&db).await;

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_headers(Any)
        .allow_methods(Any);

    let app = routes()
        .layer(axum::middleware::from_fn(log_request))
        .layer(cors)
        .with_state(db);

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap();

    Ok(())
}
