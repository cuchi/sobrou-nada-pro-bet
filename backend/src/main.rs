use std::panic;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    panic::set_hook(Box::new(|info| {
        let payload = info
            .payload()
            .downcast_ref::<&str>()
            .copied()
            .or_else(|| info.payload().downcast_ref::<String>().map(|s| s.as_str()))
            .unwrap_or("unknown panic");
        let location = info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown location".into());
        tracing::error!(%payload, %location, "Panic caught");
    }));

    let _ = dotenvy::dotenv();

    let default_filter: String = match std::env::var("RUST_LOG") {
        Ok(_) => "".into(),
        Err(_) => {
            let is_prod = std::env::var("ENVIRONMENT")
                .map(|v| v == "production")
                .unwrap_or(false);
            if is_prod {
                "sobrou_nada_pro_bet=info,tower_http=info".into()
            } else {
                "sobrou_nada_pro_bet=debug,tower_http=debug".into()
            }
        }
    };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| default_filter.into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set (check .env file)");

    if std::env::var("JWT_SECRET").is_err() {
        panic!("JWT_SECRET must be set. Generate one with: openssl rand -base64 32");
    }

    let is_prod = std::env::var("ENVIRONMENT")
        .map(|v| v == "production")
        .unwrap_or(false);

    if is_prod && std::env::var("GOOGLE_CLIENT_ID").is_err() {
        panic!("GOOGLE_CLIENT_ID must be set in production");
    }
    if std::env::var("ADMIN_TOKEN").is_err() {
        panic!("ADMIN_TOKEN must be set. Generate one with: openssl rand -base64 32");
    }

    let pool = sobrou_nada_pro_bet::db::init(&database_url).await;
    let app = sobrou_nada_pro_bet::build_app(pool).await;

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".into());
    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("Listening on http://{addr}");

    axum::serve(listener, app).await.unwrap();
}
