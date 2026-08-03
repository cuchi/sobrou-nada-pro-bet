# 🎲 Sobrou Nada Pro Bet

*A cashless betting app for sharing highscores with friends.*

Place friendly bets on real-world events using points instead of money. Compete with your friends on a shared leaderboard.

## What it does (today)

- Sign in with Google
- Place bets with custom amounts and odds
- Resolve bets as won or lost
- See all bets in a shared table

## What it will do (roadmap)

- Pull real odds and events from sports/bookmaker APIs
- Points-based economy (no real money)
- Private group leaderboards
- Share results with friends

## Tech Stack

| Layer | Technology |
|---|---|
| Frontend | React 19, TypeScript, Vite 6 |
| Backend | Rust, Axum, SQLx |
| Database | PostgreSQL 16 |
| Auth | Google OAuth + JWT |

## Getting Started

```sh
# 1. Database
docker-compose up -d

# 2. Backend
cd backend
cp .env.example .env    # fill in GOOGLE_CLIENT_ID + JWT_SECRET
cargo run

# 3. Frontend
cd frontend
cp .env.example .env.local   # fill in VITE_GOOGLE_CLIENT_ID
npm install
npm run dev
```

Open http://localhost:5173

See [AGENTS.md](AGENTS.md) for detailed architecture and conventions for AI agents.
