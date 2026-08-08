//! Mailgun v3 HTTP client.
//!
//! When `api_key` or `domain` is `None` (dev / tests), `send()` is a
//! logged no-op returning `Ok(())`. Real sends go to
//! `https://api.mailgun.net/v3/{domain}/messages` with HTTP Basic auth
//! (`api:<key>`) and a multipart form body containing `from`, `to`,
//! `subject`, `text`, and `html`.

use reqwest::{Client, multipart};
use std::time::Duration;

use crate::env::ENV;

/// Thin wrapper around reqwest + the Mailgun Messages API.
/// Cheap to clone (reqwest::Client is Arc-backed).
#[derive(Clone)]
pub struct EmailClient {
    http: Client,
    api_key: Option<String>,
    domain: Option<String>,
    from_email: String,
    from_name: String,
}

impl EmailClient {
    /// Build a client from the global `ENV`. Reads `MAILGUN_API_KEY`,
    /// `MAILGUN_DOMAIN`, and `MAILGUN_FROM`; falls back to a sensible
    /// dev default for the sender address.
    pub fn from_env() -> Self {
        let api_key = ENV.mailgun_api_key.clone();
        let domain = ENV.mailgun_domain.clone();
        let from_email = ENV
            .mailgun_from
            .clone()
            .unwrap_or_else(|| "noreply@sobrou-nada.local".into());
        let from_name = "Sobrou Nada Pro Bet".into();
        let http = Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .expect("reqwest client builder is infallible for these settings");
        Self {
            http,
            api_key,
            domain,
            from_email,
            from_name,
        }
    }

    /// For tests: build with explicit values.
    pub fn new(api_key: Option<String>, domain: Option<String>, from_email: String) -> Self {
        let http = Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .expect("reqwest client builder is infallible for these settings");
        Self {
            http,
            api_key,
            domain,
            from_email,
            from_name: "Sobrou Nada Pro Bet".into(),
        }
    }

    /// Send a single email. Returns `Ok(())` on either success or
    /// "no key/domain configured" (logged as a tracing event). Errors
    /// are returned as `String` so callers can log them at the call
    /// site without pulling reqwest types into their signatures.
    pub async fn send(
        &self,
        to_email: &str,
        to_name: &str,
        subject: &str,
        body_text: &str,
        body_html: &str,
    ) -> Result<(), String> {
        let (Some(key), Some(domain)) = (self.api_key.as_deref(), self.domain.as_deref()) else {
            tracing::info!(
                to = %to_email,
                subject = %subject,
                "Email send skipped (no MAILGUN_API_KEY or MAILGUN_DOMAIN)"
            );
            return Ok(());
        };

        let form = multipart::Form::new()
            .text("from", format!("{} <{}>", self.from_name, self.from_email))
            .text("to", format!("{to_name} <{to_email}>"))
            .text("subject", subject.to_string())
            .text("text", body_text.to_string())
            .text("html", body_html.to_string());

        let resp = self
            .http
            .post(format!("https://api.mailgun.net/v3/{domain}/messages"))
            .basic_auth("api", Some(key))
            .multipart(form)
            .send()
            .await
            .map_err(|e| format!("mailgun request failed: {e}"))?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            tracing::error!(
                to = %to_email,
                subject = %subject,
                status = %status,
                body = %text,
                "Mailgun returned non-2xx"
            );
            return Err(format!("mailgun returned {status}: {text}"));
        }

        tracing::info!(
            to = %to_email,
            subject = %subject,
            "Email sent"
        );
        Ok(())
    }
}
