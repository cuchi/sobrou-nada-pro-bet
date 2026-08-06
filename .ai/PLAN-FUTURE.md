# PLAN-FUTURE.md — Sobrou Nada Pro Bet (monetization-gated)

Work that requires upgrading the Render subscription. **Not starting until the app is monetized.**

The Render free tier spins the web service down after 15 minutes of inactivity, which kills long-running Tokio tasks. Anything below needs either a paid Render plan or a deliberate external trigger.

> 🅟️ = paid Render upgrade required before implementation can begin.

---

## 🅟️ Background worker — periodic sync + resolve

The one missing "always-on" piece: periodic sync + resolve instead of manual admin calls.

```text
Worker loop (runs every ~5 min)
  |
  +-- 1. Sync events (the-odds-api.com)
  |     Same logic as POST /admin/events/sync, in-process
  |     UPSERT into events table
  |
  +-- 2. Resolve bets
  |     SELECT bets WHERE status = 'pending'
  |       AND events.status = 'finished'
  |     Compare prediction vs actual score
  |     UPDATE bets.status, group_members.balance
```

- **Idempotent** — safe to re-run
- **Error-resilient** — one failure doesn't stop the loop
- **Fully logged** — `tracing::info!` at each step

> ⚠️ **Render free tier limitation:** the web service spins down after 15 minutes of inactivity, which would kill a long-running Tokio task. Options once monetized:
> - Upgrade to a paid Render plan (keeps the service alive, enables a worker process)
> - Use Render **Cron Jobs** ($0 for simple scheduled HTTP calls — could trigger `/admin/events/sync` + an auto-resolve endpoint periodically)
> - Use an external cron service (e.g. cron-job.org) to ping admin endpoints

---

## 🅟️ Render plan upgrade

- [ ] Evaluate Render paid tiers — Starter vs Standard vs Pro
- [ ] Estimate usage-driven cost (always-on web service + background worker)
- [ ] Migrate off free tier once the app is monetized
- [ ] Review Render pricing regularly as usage grows

---

## Triggered (not blocking)

These items don't require the upgrade on their own, but pair naturally with the worker once it lands:

- **Email triggers** — once a background worker exists, the bet-resolved and new-events email jobs from [PLAN-CURRENT.md](PLAN-CURRENT.md) can fire from the worker loop instead of from manual admin calls. The email feature itself ships first; the worker is just one possible trigger source.