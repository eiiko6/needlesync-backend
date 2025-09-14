use axum::{Extension, Json, Router, http::StatusCode, routing::post};
use sqlx::PgPool;

use crate::auth::{LoginPayload, LoginResponse, create_jwt, hash_password, verify_password};

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password_hash: String,
    pub email: String,
}

#[derive(serde::Deserialize)]
pub struct NewUserPayload {
    pub email: String,
    pub username: String,
    pub password: String,
}

pub fn routes() -> Router {
    Router::new()
        .route("/login", post(login))
        .route("/register", post(register_user))
}

pub async fn login(
    Extension(db): Extension<PgPool>,
    Json(payload): Json<LoginPayload>,
) -> Result<Json<LoginResponse>, (StatusCode, String)> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
        .bind(&payload.email)
        .fetch_optional(&db)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "DB error".into()))?
        .ok_or((StatusCode::UNAUTHORIZED, "Invalid credentials".into()))?;

    if !verify_password(&user.password_hash, &payload.password) {
        return Err((StatusCode::UNAUTHORIZED, "Invalid credentials".into()));
    }

    let token = create_jwt(user.id).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(LoginResponse {
        id: user.id,
        email: user.email,
        token,
    }))
}

pub async fn register_user(
    Extension(db): Extension<PgPool>,
    Json(payload): Json<NewUserPayload>,
) -> Result<StatusCode, (StatusCode, String)> {
    if payload.email.is_empty() || payload.username.is_empty() || payload.password.is_empty() {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Cannot create a user with empty fields".into(),
        ));
    }

    let password_hash =
        hash_password(&payload.password).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let result =
        sqlx::query("INSERT INTO users (username, email, password_hash) VALUES ($1, $2, $3)")
            .bind(&payload.username)
            .bind(&payload.email)
            .bind(&password_hash)
            .execute(&db)
            .await;

    match result {
        Ok(_) => Ok(StatusCode::CREATED),
        Err(e) => {
            if let Some(db_err) = e.as_database_error() {
                if db_err.code().map(|c| c == "23505").unwrap_or(false) {
                    return Err((
                        StatusCode::CONFLICT,
                        "Email or username already taken".into(),
                    ));
                }
            }
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}
