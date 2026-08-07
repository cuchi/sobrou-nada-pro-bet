# AGENTS.md — Sobrou Nada Pro Bet

Instructions for AI coding agents working on this codebase.

---

## Overview

Full-stack web application for a cashless betting system (closed beta with friends). Users sign in with Google, place points-based bets on Brazilian football matches (amount + locked odds), and are auto-resolved against real match scores. Compete in private groups and climb a per-group leaderboard.

## Plan / Progress

| Phase | Feature | Status |
|---|---|---|
| 1 | Auth — Google OAuth, JWT, beta allowlist | ✅ Shipped |
| 2 | Groups — create, join via invite code, per-group balances | ✅ Shipped |
| 3 | Events — sync from the-odds-api.com, store odds | ✅ Shipped |
| 4 | Bets — locked odds, 1h cutoff, no duplicates, auto-resolve | ✅ Shipped |
| 4b | Admin — secret-token-protected `/admin/` endpoints | ✅ Shipped |
| 4c | UI polish — SVG crests, event cards, responsive layout | ✅ Shipped |
| 6 | Auto-resolve — compare prediction vs score (mock-tested) | ✅ Shipped |
| 8 | Deploy — Dockerfile, Render, managed Postgres, prod CORS, HTTPS, CI | ✅ Shipped |
| 9 | SPA polish — spinners, toasts, optimistic updates, polling | ✅ Shipped |
| 12 | Backend tests & coverage — 28 integration tests, 76% line coverage | ✅ Shipped |
| — | Match card event status label & styling | 🔲 Todo |
| 10 | Internationalization (en / pt-BR) | 🔲 Todo |
| — | SPA polish (remaining) | 🔲 Todo |
| — | Emails — bet resolved + new events | 🔲 Todo |
| — | Hardening — rate limits, security headers, auto-resolve polish | 🔲 Todo |

See **[.ai/PLAN-CURRENT.md](PLAN-CURRENT.md)** for the full remaining roadmap.
See **[.ai/PLAN-1.0.md](PLAN-1.0.md)** for what shipped in 1.0.
See **[.ai/PLAN-FUTURE.md](PLAN-FUTURE.md)** for monetization-gated work (Render upgrade, background worker).

## Stack

| Layer | Technology |
|---|---|
| Frontend | React 19.2, TypeScript 5.8, Vite 8.2 |
| Backend | Rust 1.97 (2024 edition), Axum 0.8, SQLx 0.9 |
| Database | PostgreSQL 17 (via Docker Compose) |
| Auth | Google OAuth 2.0 (implicit flow) + JWT |
| Dev proxy | Vite proxies `/api` and `/health` to backend |

## Color Palette

All colors are defined as CSS custom properties on `:root` in `App.css`.

```
── Backgrounds ──────────────────────────────────
--bg-body:       #0b0e14    page background
--bg-card:       #121721    event cards, bet list
--bg-panel:      #181e2a    bet form, leaderboard panels
--bg-hover:      #1c2331    card hover state
--bg-input:      #161b26    form inputs, selects
--bg-selected:   #1c2436    card selected state

── Borders ─────────────────────────────────────
--border-card:   #232d3f    card borders
--border-input:  #2c374d    input borders
--border-hover:  #3b485d    hover accent

── Text ────────────────────────────────────────
--text-body:     #f3f4f6    primary text
--text-secondary:#9ca3af    secondary text
--text-muted:    #6b7280    muted labels
--text-dim:      #4b5563    dim text
--text-subtle:   #374151    very subtle

── Brand ───────────────────────────────────────
--gold:          #f0c040    primary accent
--gold-dark:     #785600    dark gold (active odds pill text)

── Semantic ────────────────────────────────────
--green:         #34d399    success / online
--green-bg:      #0b2e21    success background
--red:           #f87171    error / danger
--red-bg:        #3b1418    error background
--red-banner:    #fca5a5    error banner text
--blue:          #60a5fa    selection / info
--pending-bg:    #382d0c    pending badge background
--dev-bg:        #2d2309    dev login button

── Misc ────────────────────────────────────────
--pill-bg:       rgba(255,255,255,0.05)   odds pill
--pill-hover:    rgba(255,255,255,0.06)   odds pill hover
--crest-shadow:  rgba(0,0,0,0.50)         crest drop-shadow
--crest-halo:    rgba(255,255,255,0.60)   light halo so dark crests pop on dark bg
--active-pill:   rgba(240,192,64,0.15)    active odds pill
--podium-bg:     rgba(240,192,64,0.08)    podium row
--live-glow:     rgba(52,211,153,0.55)    LIVE badge pulse (alpha channel for box-shadow ring)
```

## Project Structure

```
├── docker-compose.yml          # PostgreSQL 17
├── .ai/
│   ├── AGENTS.md               # This file
│   ├── PLAN-1.0.md             # What shipped in 1.0
│   └── PLAN-CURRENT.md         # Remaining roadmap
├── .github/workflows/ci.yml    # Build + test + coverage on push/PR
├── .gitignore
├── backend/
│   ├── Cargo.toml
│   ├── .env                    # Secrets (gitignored)
│   ├── .env.example            # Template for .env
│   ├── migrations/             # SQLx migrations (auto-run at startup)
│   ├── tests/                  # Integration tests (snpb_test DB, auto-isolated)
│   │   ├── common/mod.rs       # Shared harness: test_db(), app(), get/post_json
│   │   ├── auth.rs / groups.rs / bets.rs / events.rs / admin.rs / sync.rs
│   └── src/
│       ├── main.rs             # Entrypoint: panic hook, logging, serve
│       ├── lib.rs              # build_app() router assembly, CORS
│       ├── db.rs               # PgPool init + migration runner
│       ├── env.rs              # LazyLock<Env> — required/optional env vars
│       ├── error.rs            # AppError — 5xx messages never leak to client
│       ├── models.rs           # User, Bet, BetStatus, Event, JWT claims, Google payloads
│       ├── auth.rs             # AuthUser extractor (Bearer → JWT decode → user ID)
│       └── routes/
│           ├── mod.rs          # Re-exports + health_check
│           ├── auth.rs         # google_login, me, dev_login
│           ├── groups.rs       # create, list, get, invite, join, leaderboard
│           ├── events.rs       # list_events (derives live/finished on the fly)
│           ├── bets.rs         # create, list (group-scoped)
│           └── admin.rs        # sync_events, resolve_bets (+ AdminAuth extractor)
└── frontend/
    ├── package.json
    ├── tsconfig.json / .app.json / .node.json
    ├── vite.config.ts          # Dev proxy to backend :3000
    ├── .env.example            # Template for .env.local
    ├── index.html
    └── src/
        ├── main.tsx
        ├── App.tsx             # Root: providers, layout, auth gating, group in URL
        ├── App.css             # Dark theme, responsive, color palette
        ├── crests.ts           # Team name → local SVG mapping
        ├── usePolling.ts       # Polling hook (deep-compare, no flicker on no-diff)
        ├── vite-env.d.ts
        ├── types/
        │   └── index.ts        # Bet, BetStatus, EventStatus, PublicUser, AuthResponse, ...
        ├── api/
        │   └── client.ts       # fetch wrappers with auto JWT injection
        ├── context/
        │   └── AuthContext.tsx  # useAuth(): user, groups, login, logout
        └── components/
            ├── BetForm.tsx      # Embeds EventPicker, amount input, place bet
            ├── BetList.tsx      # Table: avatar, event, pick, odds tooltip, status, betted-at + pagination
            ├── EventPicker.tsx  # Scrollable match list with SVG crests, odds, search
            ├── GroupSwitcher.tsx # Dropdown + create/join/invite-code buttons
            ├── Leaderboard.tsx  # Podium + ranking table (balance + at-risk)
            ├── Toast.tsx        # ToastProvider + useToast hook
            ├── GoogleLoginButton.tsx
            ├── DevLoginButton.tsx
            └── Spinner.tsx
```

## API Endpoints

| Method | Path | Auth | Description |
|---|---|---|---|
| GET | `/health` | No | `{"status":"ok"}` |
| POST | `/api/auth/google` | No | Body: `{credential: "google_id_token"}` → JWT + user |
| GET | `/api/auth/me` | Bearer | Returns current user |
| GET | `/api/groups` | Bearer | List user's groups with balances |
| POST | `/api/groups` | Bearer | Body: `{name}` → creates a group |
| GET | `/api/groups/:id` | Bearer | Group details + member count |
| GET | `/api/groups/:id/invite` | Bearer | Get invite code |
| POST | `/api/groups/:id/invite` | Bearer | Regenerate invite code |
| POST | `/api/groups/join/:code` | Bearer | Join group by invite code |
| GET | `/api/groups/:id/leaderboard` | Bearer | Ranked members by balance |
| GET | `/api/events` | No* | List events (status derived on the fly: scheduled / live / finished / cancelled) |
| GET | `/api/bets?group_id=:id` | Bearer | List bets for a group (ordered newest) |
| POST | `/api/bets` | Bearer | Body: `{group_id, event_id, prediction, amount, odds}` → creates bet |
| POST | `/api/dev/login` | No† | Body: `{email}` → dev-only login (creates user + allowlist) |
| POST | `/admin/events/sync` | Admin | Sync events from the-odds-api.com |
| POST | `/admin/bets/resolve` | Admin | Fetch scores and resolve pending bets automatically |

\* `GET /api/events` is public for the MVP.

† `POST /api/dev/login` only works when `ENVIRONMENT != "production"`.

### Admin Auth

Admin endpoints use a secret token passed via the `X-Admin-Token` header (not JWT). The `AdminAuth` extractor in `routes/admin.rs` validates it against the `ADMIN_TOKEN` env var:
- Missing header → 401
- Wrong token → 403

### Event statuses

`/admin/events/sync` only stores `scheduled` events (it's a manual script, so the transient `live` state never persists). Resolution (`/admin/bets/resolve`) flips stored events to `finished` or `cancelled`. `GET /api/events` **derives** the transient states on the fly from `start_time`:
- `scheduled` — starts in the future
- `live` — started, within ~2h match window
- `finished` — results already synced, or the match window elapsed but results aren't resolved yet
- `cancelled` — stored as such

## Auth Flow

1. User clicks Google Sign-In → `@react-oauth/google` returns a credential (ID token).
2. Frontend calls `POST /api/auth/google` with the credential.
3. Backend verifies via `https://oauth2.googleapis.com/tokeninfo?id_token=...`, validates `aud` matches `GOOGLE_CLIENT_ID`, checks `email_verified`.
4. User is upserted into the `users` table (`ON CONFLICT google_id`).
5. Backend returns a JWT (signed with `JWT_SECRET`, 7-day expiry) + `PublicUser`.
6. Frontend stores JWT in `localStorage` and attaches it as `Authorization: Bearer <token>` on all subsequent requests.
7. `GET /api/auth/me` validates the stored token on page load.

## Running Locally

```sh
# Terminal 1 — Database
docker-compose up -d

# Terminal 2 — Backend
cd backend
cp .env.example .env          # Fill in JWT_SECRET, ADMIN_TOKEN, ODDS_API_KEY
cargo run                      # http://localhost:3000

# Terminal 3 — Frontend
cd frontend
cp .env.example .env.local    # Fill in VITE_GOOGLE_CLIENT_ID (optional in dev)
npm install
npm run dev                    # http://localhost:5173
```

Migrations run automatically on backend startup.

**Tip:** Google OAuth is optional for local testing — use the dev login button instead (works only when `ENVIRONMENT != "production"`).

## Environment Variables

### Backend (`backend/.env`)

| Variable | Required | Default | Description |
|---|---|---|---|
| `DATABASE_URL` | Yes | — | Postgres connection string |
| `GOOGLE_CLIENT_ID` | Yes (production) | — | From GCP → Credentials → OAuth 2.0 Client ID |
| `JWT_SECRET` | Yes | — | Random base64 string (`openssl rand -base64 32`) |
| `ADMIN_TOKEN` | Yes | — | Random base64 string (`openssl rand -base64 32`) |
| `ODDS_API_KEY` | Yes | — | API key from the-odds-api.com |
| `ENVIRONMENT` | No | `development` | Set to `production` to harden logging/CORS |
| `CORS_ALLOWED_ORIGINS` | No | `*` (dev) | Comma-separated origins (required in prod) |
| `PORT` | No | `3000` | HTTP listen port |
| `RUST_LOG` | No | `debug` (dev) / `info` (prod) | Tracing filter |

### Frontend (`frontend/.env.local`)

| Variable | Required | Description |
|---|---|---|
| `VITE_GOOGLE_CLIENT_ID` | No (dev) | Same Google Client ID (must match backend in prod) |

## Security Rules

- **Never expose internal errors to the client.** `AppError::Internal` logs the full detail via `tracing::error!` but returns only `"Internal server error"` over HTTP.
- **Never hardcode secrets.** All secrets come from environment variables. The app panics at startup if required vars are missing.
- **Tokens are never logged.** `TraceLayer` redacts the `Authorization` header.
- **CORS must be explicit in production.** Set `CORS_ALLOWED_ORIGINS` to your domain(s).
- **Panics are caught.** A custom panic hook logs the details internally; the connection simply drops.

## Code Conventions

- **Backend:** Standard Rust 2024. Modules: `models`, `routes`, `auth`, `error`, `db`, `env`. Routes are split per domain under `routes/` (`auth`, `groups`, `events`, `bets`, `admin`).
- **Frontend:** Functional components with hooks. Auth state lives in `AuthContext` via React context. API calls live in `api/client.ts` with JWT auto-injection. Background data refresh uses `usePolling`.
- **SQL:** Migrations in `backend/migrations/` — numbered sequentially. `sqlx::migrate!()` runs them at compile time. All queries use `$1, $2` bind parameters (no string interpolation).
- **Error handling:** All route handlers return `Result<Json<Value>, AppError>`. Axum converts `AppError` to HTTP responses automatically via `IntoResponse`.
- **Auth extraction:** Route handlers that need the current user accept `AuthUser` as a parameter. Axum's `FromRequestParts` impl extracts and validates the JWT before the handler runs.
- **Admin auth:** Admin routes use `AdminAuth` extractor — checks `X-Admin-Token` header against `ADMIN_TOKEN` env var.
- **Tests:** Integration tests in `backend/tests/` use the shared harness in `tests/common/mod.rs`. Run with `cargo test` (or `cargo nextest run`); measure coverage with `cargo llvm-cov --summary-only`.

## Common Pitfalls

- **`docker compose` vs `docker-compose`:** The project uses `docker-compose.yml` format (v2). On macOS via Homebrew, install with `brew install docker-compose` and use `docker-compose up -d` (hyphenated).
- **`.env` files are gitignored by pattern.** Copy `.env.example` and fill in values. The tools may block reading `.env*` files — create them manually with the shell.
- **Google OAuth errors:** The most common issues are: missing `http://localhost:5173` in Authorized JavaScript Origins (GCP → Credentials → OAuth Client), or the user not being a Test User on the consent screen.
- **Migrations:** If you change the schema, add a new numbered SQL file (e.g., `008_xxx.sql`). SQLx's `migrate!()` runs all unapplied migrations on startup.
- **Frontend env vars:** Vite only reads `.env.local` on startup. Restart the dev server after changing it.
- **`cargo build` vs `cargo run`:** `sqlx::migrate!()` requires a live database at compile time (it checks the schema). Make sure Postgres is running before building.
