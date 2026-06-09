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

use crate::models::users;

async fn setup(db: &DatabaseConnection) {
    let schema = Schema::new(sea_orm::DbBackend::Sqlite);
    let stmt = schema.create_table_from_entity(users::Entity);
    db.execute(db.get_database_backend().build(&stmt))
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

    let app = routes().layer(cors).with_state(db);

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap();

    Ok(())
}
