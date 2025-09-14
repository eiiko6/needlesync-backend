use axum::{Extension, Router};
use tower_http::cors::{Any, CorsLayer};

mod auth;
mod db;
mod routes;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let db_pool = db::init_db().await;

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .merge(routes::users::routes())
        .merge(routes::projects::routes())
        .layer(Extension(db_pool))
        .layer(cors);

    let addr = "127.0.0.1:8080";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("Listening on {addr}");

    axum::serve(listener, app).await.unwrap();

    Ok(())
}

