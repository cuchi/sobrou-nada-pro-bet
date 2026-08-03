use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

/// Build a connection pool and run pending migrations.
pub async fn init(database_url: &str) -> PgPool {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
        .expect("Failed to connect to Postgres — is Docker running?");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run database migrations");

    tracing::info!("Database connected and migrations applied");
    pool
}
