//! Email templates.
//!
//! Each template returns `(subject, text, html)` and picks the body by
//! the user's locale. The default fallback is English.

use super::{NewEvent, NewEventsPayload, ResolvedBetPayload};

const BRAND_NAME: &str = "Sobrou Nada Pro Bet";

/// Pick locale, falling back to English. Accepts `pt-BR`, `pt`, `en`.
/// Anything else → `en`.
fn pick(locale: &str) -> &'static str {
    let lower = locale.to_ascii_lowercase();
    if lower.starts_with("pt") {
        "pt-BR"
    } else {
        "en"
    }
}

pub fn render_bet_resolved(p: &ResolvedBetPayload) -> (String, String, String) {
    match pick(&p.user_locale) {
        "pt-BR" => render_bet_resolved_pt(p),
        _ => render_bet_resolved_en(p),
    }
}

fn render_bet_resolved_en(p: &ResolvedBetPayload) -> (String, String, String) {
    let subject = match p.outcome {
        super::BetOutcome::Won => format!("You won on {} vs {}!", p.home_team, p.away_team),
        super::BetOutcome::Lost => {
            format!("Better luck next time — {} vs {}", p.home_team, p.away_team)
        }
    };

    let (verdict, headline) = match p.outcome {
        super::BetOutcome::Won => (
            "You won!",
            format!(
                "Your bet on {prediction} paid out {payout:.0} pts.",
                prediction = human_prediction_en(&p.prediction, &p.home_team, &p.away_team),
                payout = p.amount * p.odds,
            ),
        ),
        super::BetOutcome::Lost => (
            "You lost this one.",
            format!(
                "Your bet on {prediction} didn't land.",
                prediction = human_prediction_en(&p.prediction, &p.home_team, &p.away_team),
            ),
        ),
    };

    let text = format!(
        "{verdict}\n\n\
         {home} vs {away}\n\
         Final score: {score}\n\n\
         {headline}\n\n\
         — {brand}",
        home = p.home_team,
        away = p.away_team,
        score = p.final_score,
        brand = BRAND_NAME,
    );

    let html = format!(
        "<p><strong>{verdict}</strong></p>\
         <p>{home} vs {away}<br>Final score: {score}</p>\
         <p>{headline}</p>\
         <p>— {brand}</p>",
        home = p.home_team,
        away = p.away_team,
        score = p.final_score,
        brand = BRAND_NAME,
    );

    (subject, text, html)
}

fn render_bet_resolved_pt(p: &ResolvedBetPayload) -> (String, String, String) {
    let subject = match p.outcome {
        super::BetOutcome::Won => format!("Você ganhou em {} vs {}!", p.home_team, p.away_team),
        super::BetOutcome::Lost => {
            format!("Não foi dessa vez — {} vs {}", p.home_team, p.away_team)
        }
    };

    let (verdict, headline) = match p.outcome {
        super::BetOutcome::Won => (
            "Você ganhou!",
            format!(
                "Sua aposta em {prediction} rendeu {payout:.0} pts.",
                prediction = human_prediction_pt(&p.prediction, &p.home_team, &p.away_team),
                payout = p.amount * p.odds,
            ),
        ),
        super::BetOutcome::Lost => (
            "Não foi dessa vez.",
            format!(
                "Sua aposta em {prediction} não se confirmou.",
                prediction = human_prediction_pt(&p.prediction, &p.home_team, &p.away_team),
            ),
        ),
    };

    let text = format!(
        "{verdict}\n\n\
         {home} vs {away}\n\
         Placar final: {score}\n\n\
         {headline}\n\n\
         — {brand}",
        home = p.home_team,
        away = p.away_team,
        score = p.final_score,
        brand = BRAND_NAME,
    );

    let html = format!(
        "<p><strong>{verdict}</strong></p>\
         <p>{home} vs {away}<br>Placar final: {score}</p>\
         <p>{headline}</p>\
         <p>— {brand}</p>",
        home = p.home_team,
        away = p.away_team,
        score = p.final_score,
        brand = BRAND_NAME,
    );

    (subject, text, html)
}

fn human_prediction_en(prediction: &str, home: &str, away: &str) -> String {
    match prediction {
        "home_win" => format!("{home} win"),
        "away_win" => format!("{away} win"),
        "draw" => "draw".to_string(),
        other => other.to_string(),
    }
}

fn human_prediction_pt(prediction: &str, home: &str, away: &str) -> String {
    match prediction {
        "home_win" => format!("vitória do {home}"),
        "away_win" => format!("vitória do {away}"),
        "draw" => "empate".to_string(),
        other => other.to_string(),
    }
}

pub fn render_new_events(p: &NewEventsPayload) -> (String, String, String) {
    match pick(&p.user_locale) {
        "pt-BR" => render_new_events_pt(p),
        _ => render_new_events_en(p),
    }
}

fn render_new_events_en(p: &NewEventsPayload) -> (String, String, String) {
    let count = p.events.len();
    let subject = format!(
        "{count} new match{} available",
        if count == 1 { "" } else { "es" }
    );
    let lines: Vec<String> = p.events.iter().map(format_new_event_en).collect();
    let body = lines.join("\n");
    let text = format!(
        "New matches are up for betting:\n\n{body}\n\n— {brand}",
        body = body,
        brand = BRAND_NAME
    );
    let html = format!(
        "<p>New matches are up for betting:</p><ul>{}</ul><p>— {brand}</p>",
        lines
            .iter()
            .map(|l| format!("<li>{l}</li>"))
            .collect::<Vec<_>>()
            .join(""),
        brand = BRAND_NAME
    );
    (subject, text, html)
}

fn render_new_events_pt(p: &NewEventsPayload) -> (String, String, String) {
    let count = p.events.len();
    let subject = if count == 1 {
        "1 nova partida disponível".to_string()
    } else {
        format!("{count} novas partidas disponíveis")
    };
    let lines: Vec<String> = p.events.iter().map(format_new_event_pt).collect();
    let body = lines.join("\n");
    let text = format!(
        "Novas partidas disponíveis para aposta:\n\n{body}\n\n— {brand}",
        body = body,
        brand = BRAND_NAME
    );
    let html = format!(
        "<p>Novas partidas disponíveis para aposta:</p><ul>{}</ul><p>— {brand}</p>",
        lines
            .iter()
            .map(|l| format!("<li>{l}</li>"))
            .collect::<Vec<_>>()
            .join(""),
        brand = BRAND_NAME
    );
    (subject, text, html)
}

fn format_new_event_en(e: &NewEvent) -> String {
    format!(
        "{} vs {} ({} — {})",
        e.home_team,
        e.away_team,
        e.championship,
        e.start_time.format("%Y-%m-%d %H:%M UTC")
    )
}

fn format_new_event_pt(e: &NewEvent) -> String {
    format!(
        "{} vs {} ({} — {})",
        e.home_team,
        e.away_team,
        e.championship,
        e.start_time.format("%d/%m/%Y %H:%M UTC")
    )
}
