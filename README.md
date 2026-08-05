# 🎲 Sobrou Nada Pro Bet

[![CI](https://github.com/cuchi/sobrou-nada-pro-bet/actions/workflows/ci.yml/badge.svg)](https://github.com/cuchi/sobrou-nada-pro-bet/actions/workflows/ci.yml)

A cashless betting app for a closed beta — friends place points-based bets on Brazilian football matches (Brasileirão), compete in private groups, and climb the leaderboard.

> 🤖 Built with [DeepSeek V4 Pro](https://deepseek.com) through agentic engineering.
> Read the full story: [I Vibe-coded a Full-Stack App for \$2.96](https://cuchi.me/posts/vibe-coding/)

![Screenshot](screenshot.png)

## Prerequisites

- [Docker](https://docs.docker.com/get-docker/) (for PostgreSQL)
- [Rust](https://rustup.rs/) (2024 edition)
- [Node.js](https://nodejs.org/) 24+ and npm

## Quick Start

### 1. Database

```sh
docker-compose up -d
```

### 2. Backend

```sh
cd backend
cp .env.example .env
```

Edit `.env` and fill in:

| Variable | How to get it |
|---|---|
| `GOOGLE_CLIENT_ID` | [Google Cloud Console](https://console.cloud.google.com/) → Credentials → OAuth 2.0 Client ID (optional for local dev) |
| `JWT_SECRET` | `openssl rand -base64 32` |
| `ADMIN_TOKEN` | `openssl rand -base64 32` |
| `ODDS_API_KEY` | [the-odds-api.com](https://the-odds-api.com/) (free tier: 500 requests/month) |

```sh
cargo run   # http://localhost:3000
```

### 3. Frontend

```sh
cd frontend
cp .env.example .env.local
```

Edit `.env.local` and set `VITE_GOOGLE_CLIENT_ID` (same as backend). Optional for local testing — the Dev Login button doesn't need it.

```sh
npm install
npm run dev   # http://localhost:5173
```

## Testing

```sh
cd backend

# Option 1: nextest (recommended — faster, cleaner output)
cargo install cargo-nextest   # one-time
cargo nextest run

# Option 2: built-in test runner
cargo test
```

The test DB is created automatically on first run — no setup needed.

## Google OAuth Setup

Required for production. For local testing, a **Dev Login** button appears on the frontend that creates a test user instantly.

If you want the test the Google Auth flow locally:

1. Go to [Google Cloud Console](https://console.cloud.google.com/) → APIs & Services → Credentials
2. Create an **OAuth 2.0 Client ID** (Web application)
3. Add `http://localhost:5173` to **Authorized JavaScript Origins**
4. Under **OAuth consent screen**, add yourself as a Test User
5. Copy the Client ID into both `backend/.env` and `frontend/.env.local`

## Syncing Events

Match data comes from [the-odds-api.com](https://the-odds-api.com/). You need an `ODDS_API_KEY` (free tier works) in your `backend/.env`.

Events are synced via the admin endpoint:

```sh
curl -X POST http://localhost:3000/admin/events/sync \
  -H "X-Admin-Token: your-admin-token-from-env"
```

This fetches all upcoming Brasileirão fixtures and their odds, upserting into the database. Run it whenever you want to refresh the match list.

In production, this should be a cron job or background worker — not exposed to users.

### Resolving Bets

Once matches finish, resolve pending bets via the admin endpoint:

```sh
curl -X POST http://localhost:3000/admin/bets/resolve \
  -H "X-Admin-Token: your-admin-token-from-env"
```

This fetches scores from the-odds-api.com, marks finished events, and resolves all pending bets — crediting payouts to winners.

## Tech Stack

| Layer | Technology |
|---|---|
| Frontend | React 19, TypeScript 5.8, Vite 8 |
| Backend | Rust, Axum 0.8, SQLx 0.8 |
| Database | PostgreSQL 17 |
| Auth | Google OAuth + JWT |
| Match data | the-odds-api.com v4 |

## Trivia

The name **"Sobrou Nada Pro Bet"** is a pun on the Brazilian shitpost meme _"não sobrou nada pro beta"_ ("nothing left for the beta"). The original phrase — popular in shitposting circles — pokes fun at the "beta male" archetype: the naive guy who tries and fails, who puts in effort and ends up empty-handed. It's not meant to be taken seriously; it's a joke.

![Sobrou nada pro beta](shitpost.png)

This app swaps "beta" for "bet" — the English word for a wager. The joke works on two levels: literally, your points are gone after a bad round of bets; metaphorically, the name is a self-aware wink — a betting app that already warns you how this ends. It's a shitpost. Don't overthink it.
