//! Email module tests — no Mailgun key/domain configured, so every send
//! is the logged no-op path. Verifies templating for both locales and
//! the client construction.

use chrono::Utc;
use sobrou_nada_pro_bet::email::{
    BetOutcome, NewEvent, NewEventsPayload, ResolvedBetPayload, client::EmailClient,
};

fn make_resolved_payload(email: &str, locale: &str) -> ResolvedBetPayload {
    ResolvedBetPayload {
        user_email: email.into(),
        user_locale: locale.into(),
        user_name: "Tester".into(),
        home_team: "Flamengo".into(),
        away_team: "Vasco".into(),
        prediction: "home_win".into(),
        amount: 100.0,
        odds: 1.5,
        outcome: BetOutcome::Won,
        final_score: "2 \u{2013} 1".into(),
    }
}

fn make_events_payload(email: &str, locale: &str) -> NewEventsPayload {
    NewEventsPayload {
        user_email: email.into(),
        user_locale: locale.into(),
        user_name: "Tester".into(),
        events: vec![NewEvent {
            home_team: "Palmeiras".into(),
            away_team: "Corinthians".into(),
            championship: "Brasileirão".into(),
            start_time: Utc::now(),
        }],
        since: Utc::now(),
    }
}

#[test]
fn client_with_no_key_constructs() {
    let client = EmailClient::new(None, None, "noreply@test".into());
    // Just exercising the constructor; .send() would need a runtime.
    drop(client);
}

#[test]
fn render_bet_resolved_en_works() {
    let p = make_resolved_payload("a@b.com", "en");
    let (subject, text, html) = sobrou_nada_pro_bet::email::templates::render_bet_resolved(&p);
    assert!(subject.contains("won"));
    assert!(text.contains("Flamengo"));
    assert!(text.contains("2 \u{2013} 1"));
    assert!(html.contains("<p"));
}

#[test]
fn render_bet_resolved_pt_br_works() {
    let p = make_resolved_payload("a@b.com", "pt-BR");
    let (subject, text, html) = sobrou_nada_pro_bet::email::templates::render_bet_resolved(&p);
    assert!(subject.contains("ganhou") || subject.contains("Não foi"));
    assert!(text.contains("Flamengo"));
    assert!(text.contains("Placar final"));
    assert!(html.contains("<p"));
}

#[test]
fn render_bet_resolved_lost_subject_in_pt_br() {
    let mut p = make_resolved_payload("a@b.com", "pt-BR");
    p.outcome = BetOutcome::Lost;
    let (subject, _, _) = sobrou_nada_pro_bet::email::templates::render_bet_resolved(&p);
    assert!(subject.contains("Não foi"));
}

#[test]
fn render_new_events_en_works() {
    let p = make_events_payload("a@b.com", "en");
    let (subject, text, html) = sobrou_nada_pro_bet::email::templates::render_new_events(&p);
    assert!(subject.contains("match"));
    assert!(text.contains("Palmeiras"));
    assert!(text.contains("Corinthians"));
    assert!(html.contains("<ul"));
}

#[test]
fn render_new_events_pt_br_works() {
    let p = make_events_payload("a@b.com", "pt-BR");
    let (subject, text, _) = sobrou_nada_pro_bet::email::templates::render_new_events(&p);
    assert!(subject.contains("partida"));
    assert!(text.contains("Novas partidas"));
}

#[test]
fn unknown_locale_falls_back_to_english() {
    let p = make_resolved_payload("a@b.com", "xx-YY");
    let (subject, _, _) = sobrou_nada_pro_bet::email::templates::render_bet_resolved(&p);
    assert!(subject.starts_with("You") || subject.starts_with("Better"));
}
