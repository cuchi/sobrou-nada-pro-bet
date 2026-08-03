# AGENTS.md — Sobrou Nada Pro Bet

Instructions for AI coding agents working on this codebase.

---

## Overview

Full-stack web application for a simple betting system. Users sign in with Google, place bets (amount + odds), and resolve them as won/lost.

## Stack

| Layer | Technology |
|---|---|
| Frontend | React 19, TypeScript, Vite 6 |
| Backend | Rust (2021 edition), Axum 0.8, SQLx 0.8 |
| Database | PostgreSQL 16 (via Docker Compose) |
| Auth | Google OAuth 2.0 (implicit flow) + JWT |
| Dev proxy | Vite proxies `/api` and `/health` to backend |

## Project Structure

```
├── docker-compose.yml          # PostgreSQL 16
├── AGENTS.md                   # This file
├── .gitignore
├── backend/
│   ├── Cargo.toml
│   ├── .env                    # Secrets (gitignored)
│   ├── .env.example            # Template for .env
│   ├── migrations/             # SQLx migrations (auto-run at startup)
│   └── src/
│       ├── main.rs             # Entrypoint: logging, CORS, router, panic hook
│       ├── db.rs               # PgPool init + migration runner
│       ├── error.rs            # AppError — 5xx messages never leak to client
│       ├── models.rs           # User, Bet, BetStatus, JWT claims, Google payloads
│       ├── auth.rs             # AuthUser extractor (Bearer → JWT decode → user ID)
│       └── routes/
│           ├── mod.rs          # Public + authed route handlers
│           └── admin.rs        # Admin endpoints (secret token auth)
└── frontend/
    ├── package.json
    ├── tsconfig.json / .app.json / .node.json
    ├── vite.config.ts          # Dev proxy to backend :3000
    ├── .env.example            # Template for .env.local
    ├── index.html
    └── src/
        ├── main.tsx
        ├── App.tsx             # Root: providers, layout, auth gating
        ├── App.css             # Dark theme, responsive
        ├── vite-env.d.ts
        ├── types/
        │   └── index.ts        # Bet, BetStatus, PublicUser, AuthResponse, CreateBetRequest
        ├── api/
        │   └── client.ts       # fetch wrappers with auto JWT injection
        ├── context/
        │   └── AuthContext.tsx  # useAuth(): user, login(credential), logout()
        └── components/
            ├── BetForm.tsx      # Amount + odds form
            ├── BetList.tsx      # Table with win/loss resolve buttons
            └── GoogleLoginButton.tsx
```

## API Endpoints

| Method | Path | Auth | Description |
|---|---|---|---|
| GET | `/health` | No | `{"status":"ok"}` |
| POST | `/api/auth/google` | No | Body: `{credential: "google_id_token"}` → JWT + user |
| GET | `/api/auth/me` | Bearer | Returns current user |
| GET | `/api/bets` | No* | List all bets (ordered by newest) |
| POST | `/api/bets` | Bearer | Body: `{amount, odds}` → creates bet for authed user |
| PATCH | `/api/bets/:id/resolve` | Bearer | Body: `{status: "won"|"lost"}` → resolves bet |
| POST | `/admin/events/sync` | Admin | Sync events from the-odds-api.com |

\* `GET /api/bets` is intentionally public for the MVP leaderboard-style view.

### Admin Auth

Admin endpoints use a secret token passed via the `X-Admin-Token` header (not JWT). The `AdminAuth` extractor in `routes/admin.rs` validates it against the `ADMIN_TOKEN` env var:
- Missing header → 401
- Wrong token → 403

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
cp .env.example .env          # Fill in GOOGLE_CLIENT_ID, JWT_SECRET, ADMIN_TOKEN, ODDS_API_KEY
cargo run                      # http://localhost:3000

# Terminal 3 — Frontend
cd frontend
cp .env.example .env.local    # Fill in VITE_GOOGLE_CLIENT_ID
npm install
npm run dev                    # http://localhost:5173
```

Migrations run automatically on backend startup.

## Environment Variables

### Backend (`backend/.env`)

| Variable | Required | Default | Description |
|---|---|---|---|
| `DATABASE_URL` | Yes | — | Postgres connection string |
| `GOOGLE_CLIENT_ID` | Yes | — | From GCP → Credentials → OAuth 2.0 Client ID |
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
| `VITE_GOOGLE_CLIENT_ID` | Yes | Same Google Client ID (must match backend) |

## Security Rules

- **Never expose internal errors to the client.** `AppError::Internal` logs the full detail via `tracing::error!` but returns only `"Internal server error"` over HTTP.
- **Never hardcode secrets.** All secrets come from environment variables. The app panics at startup if required vars are missing.
- **Tokens are never logged.** `TraceLayer` redacts the `Authorization` header.
- **CORS must be explicit in production.** Set `CORS_ALLOWED_ORIGINS` to your domain(s).
- **Panics are caught.** A custom panic hook logs the details internally; the connection simply drops.

## Code Conventions

- **Backend:** Standard Rust 2021. Modules: `models`, `routes`, `auth`, `error`, `db`. Routes are in `routes/mod.rs` and `routes/admin.rs`.
- **Frontend:** Functional components with hooks. Auth state lives in `AuthContext` via React context. API calls live in `api/client.ts` with JWT auto-injection.
- **SQL:** Migrations in `backend/migrations/` — numbered sequentially. `sqlx::migrate!()` runs them at compile time. All queries use `$1, $2` bind parameters (no string interpolation).
- **Error handling:** All route handlers return `Result<Json<Value>, AppError>`. Axum converts `AppError` to HTTP responses automatically via `IntoResponse`.
- **Auth extraction:** Route handlers that need the current user accept `AuthUser` as a parameter. Axum's `FromRequestParts` impl extracts and validates the JWT before the handler runs.
- **Admin auth:** Admin routes use `AdminAuth` extractor — checks `X-Admin-Token` header against `ADMIN_TOKEN` env var.

## Common Pitfalls

- **`docker compose` vs `docker-compose`:** The project uses `docker-compose.yml` format (v2). On macOS via Homebrew, install with `brew install docker-compose` and use `docker-compose up -d` (hyphenated).
- **`.env` files are gitignored by pattern.** Copy `.env.example` and fill in values. The tools may block reading `.env*` files — create them manually with the shell.
- **Google OAuth errors:** The most common issues are: missing `http://localhost:5173` in Authorized JavaScript Origins (GCP → Credentials → OAuth Client), or the user not being a Test User on the consent screen.
- **Migrations:** If you change the schema, add a new numbered SQL file (e.g., `003_xxx.sql`). SQLx's `migrate!()` runs all unapplied migrations on startup.
- **Frontend env vars:** Vite only reads `.env.local` on startup. Restart the dev server after changing it.
- **`cargo build` vs `cargo run`:** `sqlx::migrate!()` requires a live database at compile time (it checks the schema). Make sure Postgres is running before building.
