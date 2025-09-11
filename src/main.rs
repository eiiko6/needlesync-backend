use axum::extract::Path;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Extension, Json, Router, routing::get};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::PgPool;
use tower_http::cors::{Any, CorsLayer};

#[derive(Serialize, FromRow)]
struct User {
    id: i32,
    username: String,
    password_hash: String,
    email: Option<String>,
}

#[derive(Serialize, FromRow)]
struct Project {
    id: i32,
    user_id: i32,
    name: String,
    completed: bool,
    time: i32,
}

#[derive(Deserialize)]
struct NewProject {
    user_id: i32,
    name: String,
    completed: bool,
    time: i32,
}

#[derive(Deserialize)]
struct LoginPayload {
    username: String,
    password: String,
}

#[derive(Serialize)]
struct LoginResponse {
    id: i32,
    username: String,
    token: String,
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
        .route("/login", post(login))
        .route("/projects/{user_id}", get(get_projects_for_user))
        .route("/projects", post(create_project))
        .layer(Extension(db_pool))
        .layer(cors);

    let addr = "127.0.0.1:8080";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("Listening on {addr}");

    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn login(
    Extension(db): Extension<PgPool>,
    Json(payload): Json<LoginPayload>,
) -> Result<Json<LoginResponse>, (StatusCode, String)> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1")
        .bind(payload.username)
        .fetch_all(&db)
        .await;

    let user = match user {
        Ok(ref u) => match u.first() {
            Some(first_user) => Ok(first_user), // or clone/borrow depending on your type
            None => Err((StatusCode::UNAUTHORIZED, "No users found".to_string())),
        },
        Err(e) => {
            println!("Error in login: {e}");
            Err((
                StatusCode::UNAUTHORIZED,
                "Invalid credentials 1".to_string(),
            ))
        }
    }
    .unwrap();

    // Temporary
    if user.password_hash != payload.password {
        return Err((
            StatusCode::UNAUTHORIZED,
            "Invalid credentials 2".to_string(),
        ));
    }

    // Temporary
    let token = format!("token-for-user-{}", user.id);

    Ok(Json(LoginResponse {
        id: user.id,
        username: user.username.clone(),
        token,
    }))
}

// Temporary function for token -> user_id
fn verify_token(token: &str) -> Option<i32> {
    // In a real app, verify the JWT signature and extract the user id
    if token == "testtoken" { Some(1) } else { None }
}

// async fn get_all_projects(Extension(db): Extension<PgPool>) -> Json<Vec<Project>> {
//     let projects = sqlx::query_as::<_, Project>("SELECT * FROM projects")
//         .fetch_all(&db)
//         .await
//         .unwrap_or_default();
//
//     Json(projects)
// }

async fn get_projects_for_user(
    Path(user_id): Path<i32>,
    Extension(db): Extension<PgPool>,
) -> Json<Vec<Project>> {
    let projects = sqlx::query_as::<_, Project>("SELECT * FROM projects WHERE user_id = $1")
        .bind(user_id)
        .fetch_all(&db)
        .await
        .unwrap_or_default();
    Json(projects)
}

async fn create_project(
    Extension(db): Extension<PgPool>,
    Json(payload): Json<NewProject>,
) -> Result<StatusCode, (StatusCode, String)> {
    let result = sqlx::query(
        "INSERT INTO projects (user_id, name, completed, time) VALUES ($1, $2, $3, $4)",
    )
    .bind(payload.user_id)
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
