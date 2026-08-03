# PLAN.md — Next Tasks

Roadmap for a cashless betting app — closed beta with friends, Brazilian football data.

---

## Phase 1 — Betting core (current)

- [x] Axum backend with Postgres
- [x] Google OAuth login
- [x] Create bets (amount + odds)
- [x] Resolve bets (win/loss)
- [x] Shared bet table
- [x] Production-safe error handling (5xx never leak)

---

## Phase 2 — Closed beta access

- [ ] `beta_allowlist` table: `email VARCHAR(255) PRIMARY KEY`
- [ ] Seed the table with allowed emails (manual SQL or a small admin panel)
- [ ] During `POST /api/auth/google`, after Google verification: if the email is **not** in `beta_allowlist`, return `403 Forbidden` with a friendly message ("You're not on the beta list yet")
- [ ] Frontend: show the rejection reason clearly (not a generic error)

## Phase 3 — Groups & scoped balances

- [ ] `groups` table

  ```
  id           UUID PRIMARY KEY
  name         VARCHAR(200) NOT NULL
  invite_code  VARCHAR(20) UNIQUE NOT NULL  (short, URL-safe)
  owner_id     UUID NOT NULL REFERENCES users(id)
  created_at   TIMESTAMPTZ DEFAULT NOW()
  ```

- [ ] `group_members` table

  ```
  group_id  UUID REFERENCES groups(id) ON DELETE CASCADE
  user_id   UUID REFERENCES users(id) ON DELETE CASCADE
  balance   DOUBLE PRECISION NOT NULL DEFAULT 1000
  joined_at TIMESTAMPTZ DEFAULT NOW()
  PRIMARY KEY (group_id, user_id)
  ```

- [ ] Drop `balance` column from `users` (balances are now **per-group**)
- [ ] Owner-only invite management:
  - `POST /api/groups` — create a group (caller becomes owner)
  - `GET /api/groups/:id/invite` — returns `invite_code` (owner only)
  - `POST /api/groups/:id/regenerate-invite` — rotates the invite code (owner only)
  - `POST /api/groups/join/:invite_code` — join a group via invite link
- [ ] Bets become group-scoped: `bets` table gains `group_id UUID NOT NULL REFERENCES groups(id)`
- [ ] `POST /api/bets` deducts from the user's balance in that group
- [ ] Resolving a bet updates the user's balance in that group
- [ ] Leaderboard: `GET /api/groups/:id/leaderboard` → rank members by balance DESC
- [ ] Frontend: group switcher in the header, group-scoped bet views

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
  raw_data      JSONB                              (full API response for flexibility)
  ```

- [ ] api-futebol.com.br integration
  - API docs: https://api-futebol.com.br/documentacao
  - Endpoints needed: `GET /campeonatos/:id` (competitions), `GET /campeonatos/:id/partidas` (matches)
  - Store API key in env (`FUTEBOL_API_KEY`)
  - Sync incoming matches → `events` table (scheduled cron or on-demand)
- [ ] Bet types evolve: `Bet` gains `event_id FK`, optional `prediction` field (e.g. "home_win", "away_win", "draw", "over_2.5_goals")
- [ ] Background job to auto-resolve bets when events finish (check `status = 'finished'` + `home_score`/`away_score`)
- [ ] Show real match info next to bets: teams, score, kickoff time

## Phase 5 — UI polish

- [ ] Event picker: browse upcoming matches, select one, place a prediction bet
- [ ] Leaderboard page: rank table + top-3 podium per group
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

## Quick wins (low effort, high impact)

| Task | Why |
|---|---|
| Add `event_name` string to bets | Even without the real API, manual event names make bets human-readable |
| Show user name next to each bet in the table | Social proof — feels like a group app |
| Seed the `beta_allowlist` table with your friends' emails | Instant access control, no code needed |
| Add create‑group button + group switcher dropdown | Foundation for all Phase 3 features |

---

## Schema evolution summary

```
Phase 1                      Phase 3                      Phase 4
───────                      ───────                      ───────
users                        users                        users
  id                          id                            id
  username                    username                      username
  balance  ← dropped          email                         email
  email                       google_id                     google_id
  google_id                   avatar_url                    avatar_url
  avatar_url
                             groups                       groups (unchanged)
bets                          id                          bets
  id                          name                          + event_id FK
  user_id                     invite_code                   + prediction
  amount                      owner_id FK
  odds                                                      events
  status                     group_members                   id
                               group_id FK                   external_id
                               user_id FK                    home_team
                               balance ← new                 away_team
                               joined_at                     start_time
                                                            status
                             bets                           home_score
                               + group_id FK                away_score
                                                            raw_data
```
