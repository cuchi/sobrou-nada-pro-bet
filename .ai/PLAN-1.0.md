# PLAN-1.0.md — Sobrou Nada Pro Bet (v1.0)

Snapshot of everything that shipped in the 1.0 release of the cashless betting app.

---

## Auth

- [x] Google OAuth login
- [x] `beta_allowlist` table — 403 rejection for non-allowlisted emails
- [x] Dev login button (creates user + seeds allowlist, dev-only)
- [x] JWT-based session (7-day expiry)
- [x] Dismissible error banner on frontend

## Backend infra

- [x] Axum backend with Postgres (SQLx)
- [x] Production-safe error handling (5xx never leak)
- [x] Panics caught — custom hook logs internally, connection drops
- [x] Tracing with header redaction (Authorization never logged)

## Groups

- [x] `groups` + `group_members` tables, balances per-group
- [x] Create, join via invite code, regenerate invite (32-char alphanumeric codes)
- [x] Group-scoped bets (deduct from group balance, credit on win)
- [x] Leaderboard per group with podium
- [x] Group switcher frontend
- [x] Group UUID in URL (`?group=<id>`) — survives refresh
- [x] Invite shows just the code (copy + dismiss)

## Events

- [x] `events` table with external_id, teams, start_time, status, scores, odds
- [x] the-odds-api.com v4 integration (`soccer_brazil_campeonato`)
- [x] Bets gain `event_id`, `prediction` (home_win / draw / away_win)
- [x] Odds locked from API (user cannot edit)
- [x] 1-hour cutoff before kickoff (server-enforced)
- [x] No duplicate bets on same event/user/group
- [x] Admin sync endpoint (`POST /admin/events/sync`, secret token auth)

### Derived event statuses

Sync only stores `scheduled` events (manual script). `GET /api/events` derives the rest on the fly from `start_time`:

- `scheduled` — starts in the future
- `live` — started, within the ~2h match window
- `finished` — results synced, **or** the match window elapsed but results aren't resolved yet
- `cancelled` — stored as such

## Bets

- [x] Create bets (amount + odds)
- [x] Shared bet table
- [x] Bet list: avatar with initials fallback, prediction shows team name, odds tooltip with payout
- [x] Bet list: frontend-only pagination (10 per page)
- [x] Bet list: "Betted at" column — date/time on two lines (DD/MM/YYYY + HH:SS)

## Auto-resolve

- [x] `POST /admin/bets/resolve` — fetches scores from the-odds-api.com `/scores/` endpoint
- [x] Updates event status to `finished` + stores `home_score` / `away_score`
- [x] Resolves pending bets: compares prediction vs actual result, sets won/lost
- [x] Credits payout (`amount × odds`) to winner's group balance on win
- [x] No manual resolve route — resolution is fully admin-driven
- [x] Mock-tested via `tests/sync.rs` (sync odds JSON, resolve bets, verify balance updates)

## UI

- [x] Event picker UI with team crests (SVG, transparent BG)
- [x] Responsive event cards (dark theme, grid layout, crests-only on mobile)
- [x] Two-line date/time in event cards
- [x] Prediction buttons with team name + odds on separate lines
- [x] Crest visibility — light drop-shadow halo to lift dark crests off the dark card
- [x] Already-bet matches disabled at reduced opacity
- [x] Leaderboard shows at-risk points, tiebreaker by risk amount, ellipsis on long names
- [x] Scrollbar styled to match dark theme
- [x] Team name ellipsis on overflow
- [x] Match search (accent-insensitive, frontend-only)

## SPA polish

- [x] Loading spinners — animated spinner with optional label, replaces text-based "Loading..."
- [x] Smooth transitions — fade-in on mount
- [x] Optimistic updates — reflect bet placement instantly, roll back on error
- [x] Toast notifications — success/error feedback instead of alert(), auto-dismiss
- [x] Polling hook (`usePolling`) — background refresh every 60s, deep-compares, no flicker on no-diff

## Deployment

- [x] Dockerfile — multi-stage (Node frontend build + Rust backend build → single image)
- [x] Frontend served by Rust binary (tower-http ServeDir, SPA fallback)
- [x] `.dockerignore` for lean builds
- [x] `libssl3` + `ca-certificates` in runtime image
- [x] `ENVIRONMENT=production` + `CORS_ALLOWED_ORIGINS` support
- [x] Deployed to Render (Docker runtime + managed Postgres)
- [x] CI pipeline — GitHub Actions (build backend + frontend, tests, coverage summary)
- [x] Custom domain + HTTPS (sobrounadapro.bet via Cloudflare)

## Tests

Integration tests live in `backend/tests/` (isolated via auto-created `snpb_test` DB, advisory-lock serialized).

- [x] Test harness — `tests/common/mod.rs`: auto-create + migrate + truncate, run in any order
- [x] Auth — health check, dev login, 401 without token
- [x] Groups — create/view, list, join (invalid/already member), invite owner-only, regenerate, leaderboard (+ pending-bet sums)
- [x] Bets — place & list, no duplicates, insufficient balance, non-member 403/400, 1h cutoff
- [x] Events — derived statuses (scheduled/live/finished/cancelled), filters, auth required
- [x] Admin — secret-token validation for sync/resolve
- [x] Sync/resolve — mock JSON: sync odds, resolve bets, verify balance updates
- [x] Coverage tool — `cargo-llvm-cov` (works with cargo test/nextest)
- [x] Coverage CI step — installs llvm-cov and prints summary in CI
- [x] **28 integration tests** across all suites
- [x] Line coverage: **76%** total — bets 93%, groups 92%, events 98%, models 95%