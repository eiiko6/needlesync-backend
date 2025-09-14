use sqlx::PgPool;

pub async fn init_db() -> PgPool {
    let database_url = "postgres://needlesync:secret@localhost:5432/needlesync";
    PgPool::connect(database_url)
        .await
        .expect("Failed to connect to database")
}
