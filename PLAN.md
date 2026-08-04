# PLAN.md — Sobrou Nada Pro Bet

Roadmap for a cashless betting app — closed beta with friends, Brazilian football data.

---

## Phase 1 — Core betting ✅

- [x] Axum backend with Postgres
- [x] Google OAuth login
- [x] Create bets (amount + odds)
- [x] Resolve bets (win/loss)
- [x] Shared bet table
- [x] Production-safe error handling (5xx never leak)

## Phase 2 — Closed beta ✅

- [x] `beta_allowlist` table
- [x] 403 rejection for non-allowlisted emails
- [x] Dismissible error banner on frontend
- [x] Dev login button (creates user + seeds allowlist, dev-only)

## Phase 3 — Groups & scoped balances ✅

- [x] `groups` + `group_members` tables, balances per-group
- [x] Create, join via invite code, regenerate invite
- [x] Group-scoped bets (deduct from group balance, credit on win)
- [x] Leaderboard per group with podium
- [x] Group switcher frontend

## Phase 4 — Real events & bets ✅

- [x] `events` table with external_id, teams, start_time, status, scores, odds
- [x] the-odds-api.com v4 integration (`soccer_brazil_campeonato`)
- [x] Admin sync endpoint (`POST /admin/events/sync`, secret token auth)
- [x] Bets gain `event_id`, `prediction` (home_win / draw / away_win)
- [x] Odds locked from API (user cannot edit)
- [x] 1-hour cutoff before kickoff (server-enforced)
- [x] No duplicate bets on same event/user/group
- [x] Event picker UI with team crests (48px PNG, transparent BG)
- [x] Responsive event cards (white, grid layout, crests-only on mobile)
- [x] Two-line date/time in event cards
- [x] Prediction buttons with team name + odds on separate lines
- [x] Group UUID in URL (`?group=<id>`) — survives refresh
- [x] Invite bar with copy button + dismiss
- [x] Scrollbar styled to match dark theme
- [x] Team name ellipsis on overflow

## Phase 5 — Background worker 🔲

A long-running Tokio task spawned at startup:

```text
Worker loop (runs every ~5 min)
  |
  +-- 1. Sync events (the-odds-api.com)
  |     POST /admin/events/sync equivalent, in-process
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

- **Idempotent** — safe to re-run
- **Error-resilient** — one failure doesn't stop the loop
- **Fully logged** — `tracing::info!` at each step

### Env vars needed

```env
SENDGRID_API_KEY=...   # Not yet used
```

### Render deployment env vars

**Runtime** (set in Web Service → Environment):

```env
DATABASE_URL=postgres://...           # Auto-set by Render PostgreSQL service
GOOGLE_CLIENT_ID=...                   # Same as VITE_GOOGLE_CLIENT_ID
JWT_SECRET=...                         # openssl rand -base64 32
ADMIN_TOKEN=...                        # openssl rand -base64 32
ODDS_API_KEY=...                       # From the-odds-api.com
ENVIRONMENT=production
CORS_ALLOWED_ORIGINS=https://your-app.onrender.com
```

**Build-time** (set in Web Service → Settings → Docker Build Arguments):

```env
VITE_GOOGLE_CLIENT_ID=...              # Embedded by Vite at build time
```

### Migration needed

```sql
ALTER TABLE users ADD COLUMN email_notifications BOOLEAN NOT NULL DEFAULT true;
```

## Phase 6 — Auto-resolve 🔲

- Detect finished matches from the-odds-api.com (or by comparing `start_time` to now)
- Compare `bet.prediction` vs actual score
- Auto-update bet status + group member balances
- Handle edge cases: cancelled matches, ties with no draw prediction

## Phase 7 — UI polish (remaining) 🔲

- [ ] Bet history with win/loss streaks per user
- [ ] Activity feed: "Alice just won 200 pts on Flamengo vs Palmeiras"
- [ ] Odds column: only show odds, not editable

## Phase 8 — Production deployment 🚧

- [x] Dockerfile — multi-stage (Node frontend build + Rust backend build → single image)
- [x] Frontend served by Rust binary (tower-http ServeDir, SPA fallback)
- [x] `.dockerignore` for lean builds
- [x] `libssl3` + `ca-certificates` in runtime image
- [x] `ENVIRONMENT=production` + `CORS_ALLOWED_ORIGINS` support
- [x] Deployed to Render (Docker runtime + managed Postgres)
- [x] CI pipeline — GitHub Actions (build backend + frontend on push/PR)
- [x] Custom domain + HTTPS (sobrounadapro.bet via Cloudflare)
- [ ] Rate limiting on auth endpoints

## Phase 9 — SPA polish 🔲

- [ ] Loading skeletons — shimmer placeholders while data fetches
- [ ] Smooth transitions — fade-in on mount, slide between views
- [ ] Optimistic updates — reflect bet placement instantly, roll back on error
- [ ] Toast notifications — success/error feedback instead of alert()
- [ ] Empty states — illustrations or messages for empty bet lists, groups, etc.
- [ ] Error boundaries — catch component crashes gracefully
- [ ] Offline indicator — show when backend is unreachable
- [ ] Keyboard shortcuts — Enter to submit, Esc to close modals

---

## Schema (current)

```
users                         groups
  id                            id
  username                      name
  email                         invite_code
  google_id                     owner_id FK
  avatar_url                    created_at

beta_allowlist                group_members
  email                         group_id FK
  added_at                      user_id FK
                                balance
bets                            joined_at
  id
  user_id FK                  events
  group_id FK                   id
  event_id FK                   external_id
  prediction                    home_team
  amount                        away_team
  odds                          championship
  status                        start_time
  created_at                    status
                                home_score
                                away_score
                                home_odds
                                draw_odds
                                away_odds
                                raw_data
                                created_at
```

## API Endpoints

| Method | Path | Auth | Description |
|---|---|---|---|
| GET | `/health` | No | `{"status":"ok"}` |
| POST | `/api/auth/google` | No | Google ID token → JWT |
| GET | `/api/auth/me` | Bearer | Current user |
| POST | `/api/dev/login` | No† | Dev-only login |
| GET | `/api/groups` | Bearer | List groups with balances |
| POST | `/api/groups` | Bearer | Create group |
| GET | `/api/groups/:id` | Bearer | Group details |
| GET | `/api/groups/:id/invite` | Bearer | Get invite code |
| POST | `/api/groups/:id/invite` | Bearer | Regenerate invite |
| POST | `/api/groups/join/:code` | Bearer | Join by code |
| GET | `/api/groups/:id/leaderboard` | Bearer | Ranked members |
| GET | `/api/events` | No* | Scheduled + live events |
| GET | `/api/bets` | No* | List bets (by group) |
| POST | `/api/bets` | Bearer | Create bet |
| PATCH | `/api/bets/:id/resolve` | Bearer | Resolve bet |
| POST | `/admin/events/sync` | Admin | Sync from the-odds-api.com |

\* Public for MVP. † Only in non-production.
