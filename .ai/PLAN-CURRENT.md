# PLAN-CURRENT.md — Sobrou Nada Pro Bet (post-1.0)

Remaining roadmap after the 1.0 release. Grouped by theme, roughly in priority order.

> ✅ Everything in this repo that shipped is documented in [PLAN-1.0.md](PLAN-1.0.md).

---

## 1. Automation — background worker 🔲

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
  |
  +-- 3. Send emails (SendGrid)
        For each newly-resolved bet:
          Skip if user.email_notifications = false
          Send transactional email with result
```

- **Idempotent** — safe to re-run
- **Error-resilient** — one failure doesn't stop the loop
- **Fully logged** — `tracing::info!` at each step

> ⚠️ **Render free tier limitation:** The web service spins down after 15 minutes of inactivity, which would kill a long-running Tokio task. Options:
> - Upgrade to a paid Render plan (keeps the service alive)
> - Use Render **Cron Jobs** ($0 for simple scheduled HTTP calls — could trigger `/admin/events/sync` + an auto-resolve endpoint periodically)
> - Use an external cron service (e.g. cron-job.org) to ping admin endpoints

### Env vars needed

```env
SENDGRID_API_KEY=...   # For bet resolution emails
```

### Migration needed

```sql
ALTER TABLE users ADD COLUMN email_notifications BOOLEAN NOT NULL DEFAULT true;
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

---

## 2. Auto-resolve hardening 🚧

The resolve flow works and is mock-tested, but hasn't run against real matches.

- [ ] Validate with real data — no matches played yet when 1.0 shipped (season starts Aug 8)
- [ ] Handle cancelled matches — mark event `cancelled`, refund or void pending bets
- [ ] Handle ties — no draw prediction exists; decide policy (void? loss?)
- [ ] Confirm `GET /api/events` "finished but waiting" derivation holds up at scale

---

## 3. Bet history, streaks & activity feed 🔲

- [ ] Per-user win/loss streaks
- [ ] Activity feed: "Alice just won 200 pts on Flamengo vs Palmeiras"
- [ ] Optional: aggregate per-group stats (biggest win, most active, etc.)

---

## 4. SPA polish (remaining) 🔲

- [ ] Empty states — illustrations or messages for empty bet lists, groups, etc.
- [ ] Error boundaries — catch component crashes gracefully
- [ ] Offline indicator — show when backend is unreachable
- [ ] Keyboard shortcuts — Enter to submit, Esc to close modals

---

## 5. Security & hardening 🔲

- [ ] Rate limiting on auth endpoints (`/api/auth/google`, `/api/dev/login`)
- [ ] Consider stricter CORS posture / security headers (HSTS, CSP)

---

## 6. Internationalization (i18n) 🔲

Support English (en) and Brazilian Portuguese (pt-BR).

- [ ] i18n library — `react-i18next` with language detection (browser `Accept-Language` + manual toggle)
- [ ] Translation files — `locales/en.json` and `locales/pt-BR.json` with all UI strings
- [ ] Language switcher — flag or dropdown in the header, persisted to `localStorage`
- [ ] Translate all components — headings, buttons, labels, status badges, errors, empty states
- [ ] Number/date formatting — use `Intl` with locale-aware formatting (already partially done for dates)
- [ ] Team names — keep as-is (proper nouns, not translated)
- [ ] Backend error messages — optionally localize based on `Accept-Language` header (lower priority)

---

## 7. Maintenance 🔲

Current versions as of Aug 2026 (see [PLAN-1.0.md](PLAN-1.0.md) for the stack):

- [ ] Bump Rust dependencies — `cargo update`, check for breaking changes
- [ ] Bump Node dependencies — `npm outdated` + `npm update`, audit for vulnerabilities
- [ ] Update Docker base images — Node LTS, Rust stable, Debian slim
- [ ] Review Render pricing — free tier limits, consider upgrading if usage grows
- [ ] Monitor the-odds-api.com — free tier quota (500 req/month), API version changes

---

## 8. Backend test coverage gap 🚧

Overall coverage is 76%. The one big gap:

- [ ] `routes/auth.rs` at ~40% — the `google_login` handler calls the live Google API; needs HTTP-client mocking or a refactor to cover its branches
