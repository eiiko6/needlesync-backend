use axum::{Json, Router, extract::Extension, routing::get};
use serde::Serialize;
use sqlx::{FromRow, PgPool};

#[derive(Serialize, FromRow)]
struct Project {
    id: i32,
    name: String,
    completed: bool,
    time: i32,
}

async fn get_projects(Extension(db): Extension<PgPool>) -> Json<Vec<Project>> {
    let projects = sqlx::query_as::<_, Project>("SELECT * FROM projects")
        .fetch_all(&db)
        .await
        .unwrap_or_default();

    Json(projects)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let db_pool = init_db().await;

    let app = Router::new()
        .route("/projects", get(get_projects))
        .layer(Extension(db_pool));

    let addr = "127.0.0.1:8080";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("Listening on {addr}");

    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn init_db() -> PgPool {
    let database_url = "postgres://needlesync:secret@localhost:5432/needlesync";
    PgPool::connect(database_url)
        .await
        .expect("Failed to connect to database")
}

