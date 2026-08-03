# PLAN.md — Next Tasks

Roadmap for a cashless betting app — closed beta with friends, Brazilian football data.

---

## Phase 1 — Betting core

- [x] Axum backend with Postgres
- [x] Google OAuth login
- [x] Create bets (amount + odds)
- [x] Resolve bets (win/loss)
- [x] Shared bet table
- [x] Production-safe error handling (5xx never leak)

## Phase 2 — Closed beta access

- [x] `beta_allowlist` table: `email VARCHAR(255) PRIMARY KEY`
- [x] Seed the table with allowed emails (manual SQL)
- [x] During `POST /api/auth/google`, after Google verification: if the email is **not** in `beta_allowlist`, return `403 Forbidden` with a friendly message
- [x] Frontend: dismissible red banner showing the rejection reason

## Phase 3 — Groups & scoped balances (current)

- [x] `groups` table (id, name, invite_code, owner_id, created_at)
- [x] `group_members` table (group_id, user_id, balance, joined_at)
- [x] Drop `balance` column from `users` (balances are now **per-group**)
- [x] Owner-only invite management:
  - `POST /api/groups` — create group (caller becomes owner + member with 1000 pts)
  - `GET /api/groups` — list user's groups with balances
  - `GET /api/groups/:id` — group details + member list
  - `GET /api/groups/:id/invite` — get invite code (owner only)
  - `POST /api/groups/:id/invite` — regenerate invite code (owner only)
  - `POST /api/groups/join/:code` — join by invite code
- [x] `bets` table gains `group_id` column
- [x] `POST /api/bets` deducts from user's balance, validates membership
- [x] `PATCH /api/bets/:id/resolve` credits payout (`amount × odds`) to winner's group balance
- [x] `GET /api/bets` requires auth, scoped to user's groups (403 if not a member)
- [x] Bet list shows user names (JOIN users table)
- [x] Bet list shows color-coded payout column
- [x] Leaderboard: `GET /api/groups/:id/leaderboard` → ranked by balance DESC
- [x] Frontend: group switcher with separate +Create / +Join / Invite buttons
- [x] Frontend: leaderboard component with 🥇🥈🥉 podium
- [x] Frontend: Cancel buttons on create/join inline forms
- [x] Dev-only login button (creates random test user, bypasses GCP)

## Phase 4 — Real events (api-futebol.com.br)

- [ ] `events` table — mirrors the external API

  ```
  id            UUID PRIMARY KEY
  external_id   VARCHAR(100) UNIQUE NOT NULL   (api-futebol match ID)
  home_team     VARCHAR(200) NOT NULL
  away_team     VARCHAR(200) NOT NULL
  start_time    TIMESTAMPTZ NOT NULL
  status        VARCHAR(20) NOT NULL DEFAULT 'scheduled'
                  CHECK (status IN ('scheduled', 'live', 'finished', 'cancelled'))
  home_score    INT
  away_score    INT
  raw_data      JSONB
  ```

- [ ] api-futebol.com.br integration
  - API docs: https://api-futebol.com.br/documentacao
  - Endpoints needed: `GET /campeonatos/:id` (competitions), `GET /campeonatos/:id/partidas` (matches)
  - Store API key in env (`FUTEBOL_API_KEY`)
  - Sync incoming matches to `events` table
- [ ] Bet types evolve: `Bet` gains `event_id FK`, `prediction` field (e.g. "home_win", "away_win", "draw")
- [ ] Show real match info next to bets: teams, score, kickoff time

### Background worker

A long-running Tokio task spawned at startup that runs periodic jobs:

- **Every ~5 minutes**: sync match results, auto-resolve bets, send emails
- **Idempotent** — safe to re-run, never double-resolves a bet
- **Error-resilient** — one failure doesn't stop the loop
- **Fully logged** — `tracing::info!` at each step for visibility

```text
Worker loop (runs every 5 min)
  |
  +-- 1. Sync events (api-futebol)
  |     GET /campeonatos/:id/partidas
  |     UPSERT into events table
  |
  +-- 2. Resolve bets
  |     SELECT bets WHERE status = 'pending'
  |       AND events.status = 'finished'
  |     Compare prediction vs actual score
  |     UPDATE bets.status, group_members.balance
  |
  +-- 3. Send emails (SendGrid)
        For each newly-resolved bet:
          Skip if user.email_notifications = false
          Send transactional email with result
```

### Email notifications (SendGrid)

- [ ] Add `SENDGRID_API_KEY` env var
- [ ] Add `sendgrid` crate or use `reqwest` to their Mail Send API
- [ ] `users` table: add `email_notifications BOOLEAN NOT NULL DEFAULT true`
- [ ] Email content: plain-text, includes bet details, result, points change, new balance
- [ ] Send only on bet resolution, not on creation
- [ ] Respect the `email_notifications` opt-out flag

## Phase 5 — UI polish

- [x] Show user name next to each bet in the table
- [x] Leaderboard with top-3 podium per group
- [ ] Event picker: browse upcoming matches, select one, place a prediction bet
- [ ] Bet history with win/loss streaks per user
- [ ] Mobile-responsive layout (current dark theme is a good start)
- [ ] Activity feed: "Alice just won 200 pts on Flamengo vs Palmeiras"

## Phase 6 — Production deployment

- [ ] Dockerize the backend (multi-stage Rust build)
- [ ] Serve frontend via Nginx or embed in Rust binary
- [ ] CI/CD pipeline (GitHub Actions)
- [ ] Managed Postgres (e.g. Neon, Supabase, Railway)
- [ ] Custom domain + HTTPS
- [ ] Rate limiting on auth endpoints

---

## Schema evolution summary

```
Phase 1                   Phase 3 (current)          Phase 4
───────                   ─────────────────          ───────
users                     users                      users
  id                        id                          id
  username                  username                    username
  balance  ← dropped        email                       email
  email                     google_id                   google_id
  google_id                 avatar_url                  avatar_url
  avatar_url                                            email_notifications ← new

                          groups                     groups (unchanged)
bets                        id                        bets
  id                        name                        + event_id FK
  user_id                   invite_code                 + prediction
  amount                    owner_id FK
  odds                                                 events
  status                   group_members                 id
                             group_id FK                 external_id
                             user_id FK                  home_team
                             balance                     away_team
                             joined_at                   start_time
                                                        status
                          bets                          home_score
                            + group_id FK               away_score
                                                        raw_data
```
