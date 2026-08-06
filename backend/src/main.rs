use std::panic;

use sobrou_nada_pro_bet::env::Env;
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

    let env = Env::load();

    let default_filter: String = match &env.rust_log {
        Some(_) => "".into(),
        None => {
            if env.is_prod() {
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

    let pool = sobrou_nada_pro_bet::db::init(&env.database_url).await;
    let app = sobrou_nada_pro_bet::build_app(pool).await;

    let addr = format!("0.0.0.0:{}", env.port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("Listening on http://{addr}");

    axum::serve(listener, app).await.unwrap();
}
