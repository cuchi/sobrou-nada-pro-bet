/// Centralized environment variable access with defaults and validation.
use std::sync::LazyLock;

use crate::error::AppError;

pub struct Env {
    pub database_url: String,
    pub jwt_secret: String,
    pub google_client_id: Result<String, AppError>,
    pub admin_token: String,
    pub odds_api_key: Result<String, AppError>,
    pub environment: String,
    pub cors_allowed_origins: Option<String>,
    pub rust_log: Option<String>,
    pub port: u16,
}

pub static ENV: LazyLock<Env> = LazyLock::new(Env::load);

fn required(name: &str, hint: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("{name} must be set. {hint}"))
}

fn optional(name: &str, hint: &str) -> Result<String, AppError> {
    std::env::var(name).map_err(|_| AppError::Internal(format!("{name} must be set. {hint}")))
}

impl Env {
    pub fn load() -> Self {
        dotenvy::dotenv().ok();

        let database_url = required("DATABASE_URL", "");
        let jwt_secret = required("JWT_SECRET", "Generate one with: openssl rand -base64 32");
        let admin_token = required("ADMIN_TOKEN", "Generate one with: openssl rand -base64 32");
        let environment = std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".into());

        let google_client_hint = "Get one from Google Cloud Console → Credentials.";
        let google_client_id = if environment == "production" {
            Ok(required("GOOGLE_CLIENT_ID", google_client_hint))
        } else {
            optional("GOOGLE_CLIENT_ID", google_client_hint)
        };

        let odds_api_key = optional(
            "ODDS_API_KEY",
            "Get one from the-odds-api.com (free tier: 500 req/month).",
        );

        let cors_allowed_origins = std::env::var("CORS_ALLOWED_ORIGINS")
            .ok()
            .filter(|s| !s.is_empty());

        let rust_log = std::env::var("RUST_LOG").ok().filter(|s| !s.is_empty());

        let port = std::env::var("PORT")
            .unwrap_or_else(|_| "3000".into())
            .parse()
            .expect("PORT must be a number");

        Self {
            database_url,
            jwt_secret,
            google_client_id,
            admin_token,
            odds_api_key,
            environment,
            cors_allowed_origins,
            rust_log,
            port,
        }
    }

    pub fn is_prod(&self) -> bool {
        self.environment == "production"
    }

    pub fn is_dev(&self) -> bool {
        !self.is_prod()
    }
}
