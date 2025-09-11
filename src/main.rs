use axum::http::StatusCode;
use axum::{Json, Router, extract::Extension, routing::get};
use serde::Deserialize;
use serde::Serialize;
use sqlx::{FromRow, PgPool};
use tower_http::cors::{Any, CorsLayer};

#[derive(Serialize, FromRow)]
struct Project {
    id: i32,
    name: String,
    completed: bool,
    time: i32,
}

#[derive(Deserialize)]
struct NewProject {
    name: String,
    completed: bool,
    time: i32,
}

async fn init_db() -> PgPool {
    let database_url = "postgres://needlesync:secret@localhost:5432/needlesync";
    PgPool::connect(database_url)
        .await
        .expect("Failed to connect to database")
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let db_pool = init_db().await;

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/projects", get(get_projects).post(create_project))
        .layer(Extension(db_pool))
        .layer(cors);

    let addr = "127.0.0.1:8080";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("Listening on {addr}");

    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn get_projects(Extension(db): Extension<PgPool>) -> Json<Vec<Project>> {
    let projects = sqlx::query_as::<_, Project>("SELECT * FROM projects")
        .fetch_all(&db)
        .await
        .unwrap_or_default();

    Json(projects)
}

async fn create_project(
    Extension(db): Extension<PgPool>,
    Json(payload): Json<NewProject>,
) -> Result<StatusCode, (StatusCode, String)> {
    let result = sqlx::query("INSERT INTO projects (name, completed, time) VALUES ($1, $2, $3)")
        .bind(payload.name)
        .bind(payload.completed)
        .bind(payload.time)
        .execute(&db)
        .await;

    match result {
        Ok(_) => Ok(StatusCode::CREATED),
        Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string())),
    }
}
