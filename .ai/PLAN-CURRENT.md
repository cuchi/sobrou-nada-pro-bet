# PLAN-CURRENT.md — Sobrou Nada Pro Bet (post-1.0)

Remaining roadmap after the 1.0 release. Grouped by theme, in priority order.

> ✅ Everything that shipped is documented in [PLAN-1.0.md](PLAN-1.0.md).
> 🅟️ Monetization-gated work is parked in [PLAN-FUTURE.md](PLAN-FUTURE.md).

---

## 1. Match card event status label & styling ✅

The event card now surfaces the match state at a glance: status badge per card, kickoff countdown for scheduled, pulse for live, dedicated collapsible for recent results (cancelled / awaiting / finished). All work landed in `frontend/src/components/EventPicker.tsx`, `frontend/src/App.css`, `frontend/src/kickoff.ts` (new), `frontend/src/types/index.ts`, `backend/src/routes/events.rs`, and `backend/src/bin/seed.rs` (new).

**Context**
- Backend derives `status` per event via `backend/src/routes/events.rs::display_status`: `scheduled` / `live` / `finished` / `cancelled`. Stored values are `scheduled`, `finished`, `cancelled`; `live` is purely derived from `start_time` (started and within the 2h `MATCH_DURATION` window).
- The picker now requests `?status=scheduled,live,finished,cancelled` so all four buckets reach the frontend. The frontend then partitions the response into `upcoming` (scheduled + live) for the picker grid and `previous` (cancelled + awaiting + finished) for the collapsible.
- Cancellation is wired through `/admin/bets/resolve` (sets stored `status` to `cancelled`); the frontend reflects that automatically.

**Status surface (shipped)**
- [x] `.event-status-badge` per card — four variants: `scheduled` (muted), `live` (green, animated pulse), `finished` (muted), `cancelled` (red). Plus a derived `awaiting-result` overlay (see below).
- [x] For `scheduled`: kickoff countdown from `frontend/src/kickoff.ts` (`kickoffLabel`) replacing the date/time column. Ladder: "starting now" → "in Nm" → "in Hh Mm" (later today) → "tomorrow HH:MM" → weekday + HH:MM (<7 days) → locale date + HH:MM.
- [x] For `live`: pulse animation via `live-pulse` keyframes on `--live-glow`. `prefers-reduced-motion: reduce` disables the pulse. The elapsed-minute suffix ("LIVE · 67'") was deferred — backend doesn't expose it yet.
- [x] For `finished`: muted badge, read-only. For `cancelled`: red badge, prediction bar hidden.
- [x] Badge always sits in the card's `.event-info` column (right side of the grid). On very narrow screens (`<480px`) the odds row hides and the info column takes its slot — see "Responsive" below.

**Pickable vs. non-pickable cards (shipped)**
- [x] `isClosed(status)` disables the card (`cursor: not-allowed`, no click) for everything except `scheduled`. Same opacity/betted treatment composes with it.
- [x] Prediction bar renders only when a `scheduled` card is selected. For `live` / `finished` / `cancelled` / `awaiting_result` selections, a single-line "Betting is closed — …" notice replaces the bar with status-specific copy.
- [x] The 1h server-enforced cutoff still applies — frontend defers to the backend, which already drops too-close-to-kickoff matches via its own filter.

**Awaiting result (added during implementation)**
- [x] When the match window has elapsed but `/admin/bets/resolve` hasn't run yet, the backend derives `status: "finished"` and adds `awaiting_result: true` to the JSON. Frontend renders an "Awaiting result" badge with a dashed border (variant of the `finished` styling) instead of "Finished", and the closed-notice says "Betting is closed — awaiting result." Pending bets on these matches stay `pending` in the user's BetList until resolve fires.
- [x] `frontend/src/types/index.ts` `Event` gains `awaiting_result: boolean`. Backend sets it inside `list_events` via `is_awaiting_result(event)`. The frontend partition routes awaiting rows into the `previous` list so they appear in the Recent results collapsible, chronologically interleaved with cancelled and finished rows.

**Recent results collapsible (shipped, evolved from plan)**
- [x] One collapsed-by-default `<details class="recent-results">` at the top of the picker, summary "Recent results (N)" with the disclosure arrow on the right next to the text.
- [x] Body shows cancelled, awaiting and finished rows as a **single chronological list** (newest first by `start_time`). Rows are compact: `[teams] [when] [badge] [score?]`. The score column is **only rendered** when `home_score` and `away_score` are both non-null — cancelled and awaiting rows don't draw a blank `—`.
- [x] All three buckets share a 7-day recency window (`SEVEN_DAYS_MS`). `recent.length` is capped at `RECENT_RESULTS_LIMIT = 10`.
- [x] Empty state: if `previous` is empty the whole collapsible hides (`hasPrevious` flag).
- [x] Originally the plan called for a separate red "Cancelled match" banner above the search input. During implementation the user preferred a single unified collapsible (no separate banner, no subheadings — the badge inside each row is enough). If user feedback changes, the partition already produces three separate lists and we can re-introduce subheadings in one place.

**Responsive (shipped)**
- [x] `@media (max-width: 640px)` — recent-result cards stack into two lines: line 1 = team names (full width, no ellipsis), line 2 = meta cluster **left-aligned** in `[when, badge, score?]` order. Recent-results collapsible hides its bottom divider when collapsed (`recent-results:not([open]) .recent-results-list { border-bottom: none; padding-bottom: 0 }`) so the gap above "Upcoming matches" stays tight.
- [x] `@media (max-width: 480px)` — upcoming match cards hide the odds row entirely (`.event-odds { display: none }`) and the card grid template drops to 4 columns with `.event-info` moving into the freed slot.
- [x] `.event-picker-heading` uses `margin: 0.25rem 0 0.75rem` so the gap above "Upcoming matches" matches the rest of the picker rhythm.
- [x] Pulse animation respects `prefers-reduced-motion: reduce`.

**Styling & palette (shipped)**
- [x] All new colors reuse existing tokens: `--green`, `--green-bg`, `--red`, `--red-bg`, `--text-muted`, `--text-secondary`, `--text-dim`, `--text-body`, `--bg-input`, `--bg-panel`, `--bg-card`, `--border-card`. One new token: `--live-glow: rgba(52,211,153,0.55)` (alpha channel for the live pulse `box-shadow` ring).
- [x] `.event-status-badge` chosen over `.status-badge` to avoid clashing with `BetList`'s bet-status-specific selector.
- [x] `live-pulse` keyframes at ~1.6s loop, `prefers-reduced-motion` honoured.

**Where it lives in the code (post-impl line refs)**
- `frontend/src/components/EventPicker.tsx` — `EventCardRow` was extracted and then inlined back; partition lives at L82–117, derived `previous` list at L139–143, render at L184–201.
- `frontend/src/kickoff.ts` — new file, kickoff countdown ladder.
- `frontend/src/App.css` — `.event-status-badge` rules and `live-pulse` keyframes near L820–870; `.recent-result*` rules and divider collapse logic at L632–715; mobile overrides at L1245 (640px) and L1392 (480px).
- `frontend/src/types/index.ts` — `Event` interface gained `awaiting_result: boolean` at L80.
- `backend/src/routes/events.rs` — `display_status` unchanged; new `is_awaiting_result` helper at L57, payload gains `awaiting_result` at L92.
- `backend/src/bin/seed.rs` — new binary, see "Dev seed" below.

**Edge cases (shipped behaviour)**
- [x] `live` → `finished` between polls: badge transitions cleanly, card slides from the upcoming grid into the recent-results collapsible on the next 60s poll. No flash of empty state because the `previous` list is built from the same `events` array as `upcoming`.
- [x] Match goes `live` → `finished` while idle: handled by 60s polling + the recent-results collapsible opening on next visit (still collapsed by default — user has to click to see).
- [x] User has a pending bet on a now-finished match: `BetList` row flips `pending` → `won`/`lost` independently. The recent-results card surfacing is purely additive and informational.
- [x] `awaiting_result` rows: when resolve eventually fires, the row's badge flips from "Awaiting result" to "Finished" and the score appears (it was previously rendered as nothing). No flash, just a quiet state transition.

**Out of scope (deferred)**
- Per-match "your bet on this match" pill on the card itself — would benefit from i18n first.
- Live score ticker (would need backend changes to expose scores before resolve).
- Push notifications for live/finished (lives under §4 Emails).
- "LIVE · 67'" elapsed-minute suffix — needs a backend field; the seed already has `seed-live-edge` at 1h50m which would be a natural "67'" demo if/when this lands.

**Dev seed (shipped)**
- `backend/src/bin/seed.rs` (`cargo run --bin seed`) inserts **14** fake events covering every status branch the picker renders:
  - 6 `scheduled` (offsets 5m / 47m / 3h14m / tomorrow / weekday-5d / 14d-future) — exercises the `kickoffLabel` ladder.
  - 2 `live` (30 min in / 1h50m in) — both inside the 2h match window.
  - 3 `finished` (home win, draw, away win) — all within last 7 days.
  - 1 `finished` (10 days ago) — proves the UI's 7-day filter excludes it.
  - 1 `cancelled` (2 days ago).
  - 1 `scheduled` but window elapsed (`seed-sched-unresolved`, −3h) — exercises the derived `awaiting_result: true` path.
- **Migrations run first** — the binary calls `sobrou_nada_pro_bet::db::init(...)` which executes `sqlx::migrate!("./migrations")`. Safe to run on a fresh database. Idempotent on existing DBs.
- **Realistic kickoff times** — `snap_to_half_hour` helper rounds every absolute `start_time` to the nearest `:00` or `:30` so dev data doesn't show kickoffs at e.g. `23:17:42`.
- Idempotent via `ON CONFLICT (external_id) DO UPDATE`. Only touches the `events` table; users/groups/bets are created normally via `/api/dev/login`.

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