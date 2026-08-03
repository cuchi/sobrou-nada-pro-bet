# PLAN.md — Next Tasks

Roadmap for turning the MVP into a cashless betting app with real data and friend leaderboards.

---

## Phase 1 — Betting core (current)

- [x] Axum backend with Postgres
- [x] Google OAuth login
- [x] Create bets (amount + odds)
- [x] Resolve bets (win/loss)
- [x] Shared bet table
- [x] Production-safe error handling (~5xx never leak)

---

## Phase 2 — Real events & data

- [ ] Add an `events` table (id, name, status, start_time, result, external_id)
- [ ] Bet types evolve: `Bet` gains `event_id` FK, optional `prediction` field
- [ ] Integrate a sports/odds API (e.g. The Odds API, Sofascore, API-Football)
  - Fetch upcoming events
  - Auto-resolve bets when events finish
  - Show real-time odds next to events
- [ ] Background job or cron to sync event results (Tokio spawn or external scheduler)

## Phase 3 — Points & economy

- [ ] Replace dollar amounts with a points system
  - Rename `amount` → `points` or add a new column
  - Every user starts with a configurable points balance (e.g. 1000)
- [ ] Display user balance in the header
- [ ] Show points won/lost in bet history
- [ ] Prevent users from betting more than their balance

## Phase 4 — Groups & leaderboard

- [ ] `groups` table (id, name, invite_code, owner_id)
- [ ] `group_members` join table (group_id, user_id)
- [ ] A user can create a group and share an invite code with friends
- [ ] Leaderboard: rank users within a group by total points
- [ ] Group-scoped bet views (only see bets from your group)

## Phase 5 — UI polish

- [ ] Event picker: browse real events, select one, place a bet
- [ ] Leaderboard page with ranking table and podium
- [ ] Bet history with win/loss streaks
- [ ] Mobile-responsive layout (current dark theme is a good start)
- [ ] Notifications/activity feed: "Alice just won 300 pts on Lakers vs Celtics"

## Phase 6 — Production deployment

- [ ] Dockerize the backend (multi-stage Rust build)
- [ ] Serve frontend via Nginx or embed in Rust binary
- [ ] CI/CD pipeline (GitHub Actions)
- [ ] Managed Postgres (e.g. Neon, Supabase, Railway)
- [ ] Custom domain + HTTPS
- [ ] Rate limiting on auth endpoints
- [ ] Health check endpoint for monitoring

---

## Quick wins (low effort, high impact)

| Task | Why |
|---|---|
| Replace hardcoded UUID input with Google-authed user | Already done ✅ |
| Add a points balance to the user model | Foundation for all game mechanics |
| Show user name next to each bet in the table | Social proof, makes it feel like a group app |
| Add `event_name` string to bets | Even without a real API, manual event names make bets readable |
| Add a simple leaderboard query | `SELECT user_id, SUM(points) GROUP BY user_id ORDER BY SUM DESC` |
