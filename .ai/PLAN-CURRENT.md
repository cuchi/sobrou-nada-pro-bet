# PLAN-CURRENT.md — Sobrou Nada Pro Bet (post-1.0)

Remaining roadmap after the 1.0 release. Grouped by theme, in priority order.

> ✅ Everything that shipped is documented in [PLAN-1.0.md](PLAN-1.0.md).
> 🅟️ Monetization-gated work is parked in [PLAN-FUTURE.md](PLAN-FUTURE.md).

---

## 1. Match card event status label & styling 🔲

The event card currently shows team crests, time, and prediction buttons, but doesn't surface the match state at a glance. Make the status obvious on the card itself.

- [ ] Status badge per card — `scheduled` / `live` / `finished` / `cancelled`, styled consistently with existing palette (green for live, muted for finished, red for cancelled)
- [ ] Kickoff countdown for `scheduled` cards (e.g. "in 2h 14m")
- [ ] "LIVE" pulse / accent for in-progress matches
- [ ] Hide prediction buttons (or show "closed" state) once the match is past the 1h cutoff / finished
- [ ] Visual treatment for `cancelled` matches (muted / strikethrough, no predictions)

## 2. Internationalization (i18n) 🔲

Support English (en) and Brazilian Portuguese (pt-BR).

- [ ] i18n library — `react-i18next` with language detection (browser `Accept-Language` + manual toggle)
- [ ] Translation files — `locales/en.json` and `locales/pt-BR.json` with all UI strings
- [ ] Language switcher — flag or dropdown in the header, persisted to `localStorage`
- [ ] Translate all components — headings, buttons, labels, status badges, errors, empty states
- [ ] Number/date formatting — use `Intl` with locale-aware formatting (already partially done for dates)
- [ ] Team names — keep as-is (proper nouns, not translated)
- [ ] Backend error messages — optionally localize based on `Accept-Language` header (lower priority)

## 3. SPA polish (remaining) 🔲

- [ ] Empty states — illustrations or messages for empty bet lists, groups, etc.
- [ ] Error boundaries — catch component crashes gracefully
- [ ] Offline indicator — show when backend is unreachable
- [ ] Keyboard shortcuts — Enter to submit, Esc to close modals

## 4. Emails 🔲

Transactional emails, gated on user preference. Two trigger types initially.

- [ ] Pick a provider — SendGrid (env: `SENDGRID_API_KEY`)
- [ ] Migration — `users.email_notifications BOOLEAN NOT NULL DEFAULT true`
- [ ] Per-user preference — surface a toggle in account/settings (default on)
- [ ] **Bet resolved** — on win/loss/void, send a short summary (match, pick, outcome, payout)
- [ ] **New events for upcoming matches** — notify opted-in users when fresh events land in the events table (weekly digest is fine; avoid spam)
- [ ] Respect `email_notifications = false` everywhere
- [ ] All sends fully logged (`tracing::info!` per send, success/failure)
- [ ] Email triggers are idempotent — re-running resolve or sync must not double-send (track `notified_at` or use the resolved timestamp as the dedupe key)

Triggering today is manual (admin hits `/admin/bets/resolve`); emails should fire from the same code path so they "just work" once a trigger exists.

## 5. Hardening 🔲

Tighten what's already shipping before adding more surface area.

- [ ] Rate limiting on `/api/auth/google` (dev login intentionally not rate-limited — local/dev-only, gated by `ENVIRONMENT != "production"`)
- [ ] Stricter CORS posture / security headers — HSTS, CSP
- [ ] Validate auto-resolve against real data (no matches played yet when 1.0 shipped; season starts Aug 8)
- [ ] Handle cancelled matches — mark event `cancelled`, **refund** pending bets to group balance
- [ ] Confirm "win prediction vs actual draw" resolves as a **loss** (draw predictions are first-class — picking `draw` is its own outcome; the only "loss" case is a non-draw pick on a draw result)
- [ ] Confirm `GET /api/events` "finished but waiting" derivation holds up at scale
- [ ] Backend test coverage gap — `routes/auth.rs` at ~40% (`google_login` hits the live Google API; needs HTTP-client mocking or a refactor to cover its branches)