use axum::extract::Path;
use axum::http::{HeaderMap, StatusCode};
use axum::routing::post;
use axum::{Extension, Json, Router, routing::get};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, Validation, decode};
use jsonwebtoken::{EncodingKey, Header, encode};
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

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: i32, // user_id
    exp: usize,
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
        .bind(&payload.username)
        .fetch_optional(&db)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "DB error".into()))?
        .ok_or((StatusCode::UNAUTHORIZED, "Invalid credentials".into()))?;

    if user.password_hash != payload.password {
        return Err((StatusCode::UNAUTHORIZED, "Invalid credentials".into()));
    }

    let expiration = Utc::now()
        .checked_add_signed(Duration::minutes(15))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: user.id,
        exp: expiration as usize,
    };

    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secretkey123".to_string());

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Token creation failed".into(),
        )
    })?;

    Ok(Json(LoginResponse {
        id: user.id,
        username: user.username,
        token,
    }))
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
    headers: HeaderMap,
    Extension(db): Extension<PgPool>,
) -> Result<Json<Vec<Project>>, (StatusCode, String)> {
    let token = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or((StatusCode::UNAUTHORIZED, "Missing token".to_string()))?;

    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secretkey123".to_string());

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )
    .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token".to_string()))?;

    if token_data.claims.sub != user_id {
        return Err((StatusCode::FORBIDDEN, "Forbidden".to_string()));
    }

    let projects = sqlx::query_as::<_, Project>("SELECT * FROM projects WHERE user_id = $1")
        .bind(user_id)
        .fetch_all(&db)
        .await
        .unwrap_or_default();

    Ok(Json(projects))
}

async fn create_project(
    Extension(db): Extension<PgPool>,
    headers: HeaderMap,
    Json(payload): Json<NewProject>,
) -> Result<StatusCode, (StatusCode, String)> {
    let token = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or((StatusCode::UNAUTHORIZED, "Missing token".to_string()))?;

    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secretkey123".to_string());

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )
    .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token".to_string()))?;

    if token_data.claims.sub != payload.user_id {
        return Err((StatusCode::FORBIDDEN, "Forbidden".to_string()));
    }

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
