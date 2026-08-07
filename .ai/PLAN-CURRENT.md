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

## 2. Internationalization (i18n) ✅

Support English (en) and Brazilian Portuguese (pt-BR). App branding is bilingual but copy reads as pt-BR to start (the existing date format on upcoming cards is `'pt-BR'`, the seed team names are Brasileirão clubs, and the user's domain is Brazilian football). The work is in three phases.

### Phase A — infrastructure ✅

Stand up the i18n plumbing. Nothing user-visible changes yet.

- [x] Pick library — `react-i18next` (mature, small, integrates cleanly with React 19 + Vite, supports lazy locale chunks, ICU MessageFormat out of the box, `localStorage` detector out of the box). Alternatives considered: `react-intl` (heavier, better for true ICU but we don't need that yet), `lingui` (excellent but adds a build step).
- [x] Wire `i18n.ts` — calls `i18next.init({ ... })` once with the `react-i18next` and `i18next-browser-languagedetector` plugins, loaded via `import './i18n'` at the top of `App.tsx`.
- [x] Locale files — `frontend/src/locales/en/common.json` and `pt-BR/common.json`. Top-level namespace `common` for now; split into feature namespaces (`eventPicker`, `betForm`, etc.) only when the `common` file exceeds ~200 keys.
- [x] Locale detection — `i18next-browser-languagedetector` reading `localStorage` first, then `navigator.language`, then falling back to `pt-BR`. A `?lng=` query param overrides everything (used by the smoke test + humans previewing a locale). Default for new users is `pt-BR` (Brazilian product).
- [x] `localStorage` persistence — language choice persisted under `i18nextLng`. Survives reloads, logouts, and SSR-less React 19 refresh.
- [x] Smoke test — Vitest + @testing-library/react covers locale detection (default = pt-BR), clicking each flag button changes `i18n.resolvedLanguage`, `aria-pressed` flips correctly, and the chosen locale is persisted to `localStorage`. Run with `npm test`. A `?lng=` query-param override exists in `i18n.ts` for manual browser testing.
- [x] Language switcher UI — `frontend/src/components/LanguageSwitcher.tsx`, flag-emoji button group in `.header-right` (🇧🇷 / 🇺🇸), one active at a time, click switches the locale immediately. Replaces the previous `backend-status` indicator (`App.tsx`), which moved to a footer row in the same PR. The header right area keeps the user-info block (avatar + name + logout) and gains the language switcher.

### Phase B — extract & translate UI strings ✅

Inventory of every hard-coded user-facing string in `frontend/src/components/`:

| Component | String | Notes |
|---|---|---|
| `App.tsx` | `🎲 Sobrou Nada Pro Bet` (h1) | Keep the emoji; translate the title? Decision needed — title is a brand name and likely stays as-is. |
| `App.tsx` | login error message from `loginError` state | Server-supplied, see Phase D for backend localisation. |
| `BetForm.tsx` | `Place a Bet in {groupName}` | Interpolation. |
| `BetForm.tsx` | `Odds: {odds}x` | Interpolation. |
| `BetForm.tsx` | `Amount (pts)` placeholder | |
| `BetForm.tsx` | `{amount} pts` | Interpolation, also needs `Intl.NumberFormat` for thousands. |
| `BetForm.tsx` | `Pick:` label inside prediction bar | |
| `BetForm.tsx` | `Draw` (prediction button) | |
| `BetList.tsx` | `All Bets ({count})` | Interpolation. |
| `BetList.tsx` | `User` / `Event` / `Pick` / `Amount` / `Odds` / `Status` / `Betted at` | Column headers (7). |
| `BetList.tsx` | `pts` suffix in amount cells | Unit, not a translation. |
| `BetList.tsx` | `Payout: {x} pts` (title attribute) | Interpolation. |
| `EventPicker.tsx` | `Recent results ({count})` | Interpolation. |
| `EventPicker.tsx` | `Upcoming matches` | |
| `EventPicker.tsx` | `Search team...` placeholder | |
| `EventPicker.tsx` | `Loading matches...` | |
| `EventPicker.tsx` | `No upcoming matches right now. Check back later.` | |
| `EventPicker.tsx` | `No matches found for "{query}".` | Interpolation. |
| `EventPicker.tsx` | `Betting is closed — match is in progress.` | |
| `EventPicker.tsx` | `Betting is closed — match has finished.` | |
| `EventPicker.tsx` | `Betting is closed — awaiting result.` | |
| `EventPicker.tsx` | `Match was cancelled.` | |
| `EventPicker.tsx` | `vs` (in event cards) | Decorative connector. |
| `EventPicker.tsx` | `Status: ...` aria-label | aria-labels should be localised too. |
| `StatusBadge` (in `EventPicker.tsx`) | `Scheduled` / `Live` / `Finished` / `Cancelled` / `Awaiting result` | |
| `GroupSwitcher.tsx` | `Create` / `Join` / `Copy` buttons | |
| `GroupSwitcher.tsx` | `Group name` / `Invite code` placeholders | |
| `GroupSwitcher.tsx` | `Close` aria-label | |
| `Leaderboard.tsx` | `Leaderboard` heading | |
| `Leaderboard.tsx` | `#` / `Player` / `Balance` / `At risk` column headers | |
| `Leaderboard.tsx` | `{balance} pts` | Interpolation. |
| `Toast.tsx` | messages are server-supplied | See Phase D. |

Also:
- [x] Replace every literal in the table above with `t('namespace.key')` calls. (~80 keys under namespaced sections in `common.json`: `app`, `header`, `footer`, `betForm`, `betList`, `eventPicker`, `groupSwitcher`, `leaderboard`, `units`, `errors`.)
- [x] Extract a small `<Points>` helper component that renders `{Intl.NumberFormat(locale).format(amount)} pts` so the unit + number formatting live in one place. (`frontend/src/components/Points.tsx` — exports `Points`, `formatPoints`, and `useActiveLocale`.)
- [x] Extract a small `<StatusBadge>` lookup helper (already a component — just change its labels to `t(...)` calls).
- [x] Add a Vitest test that snapshots the rendered app per locale and asserts no missing-key warnings. (Per-locale `it.each` walks the union of all shipped keys and asserts each resolves in each locale — catches both "key missing in pt-BR" and "key missing in en".)
- [x] Plural-rule test for pt-BR's `>1` form at `count: 2`. (`betList.heading_one` / `_other` exist in pt-BR; test asserts resource presence + that `count: 2` resolves via the plural-rule path.)
- [x] Wrap fallback error strings in `api/client.ts` and `AuthContext` through `t('errors.*')` so the rare case where the server doesn't include a message flows through the active locale. Phase D will replace the dynamic server-supplied strings with code-based mapping.
- [x] Replace hard-coded `'pt-BR'` `Intl.DateTimeFormat` calls in `EventPicker.tsx` (recent-results time, upcoming-card date/time columns) and `BetList.tsx` (betted-at date/time) with the active locale via `useActiveLocale()`. Phase C will own the kickoff-countdown strings themselves.

### Phase C — kickoff countdown localisation ✅

`frontend/src/kickoff.ts` mixed English literals (`"starting now"`, `"in"`, `"tomorrow"`, `"<1m"`) with `Intl.DateTimeFormat(undefined, ...)` calls. `undefined` here resolves to the browser locale, which is fine but inconsistent with the rest of the app once we have a forced locale.

- [x] Make `kickoffLabel` take an explicit `locale: string` and a `t: (key, opts) => string` parameter. `EventPicker` passes `useActiveLocale()` and `t` from `useTranslation()`.
- [x] Replace the six hard-coded English phrases with translation keys: `kickoff.startingNow`, `kickoff.inOneMinute`, `kickoff.inMinutes`, `kickoff.inHours`, `kickoff.inHoursMinutes`, `kickoff.tomorrowAt`. pt-BR equivalents use `"em"` / `"amanhã"` naturally.
- [x] Switch the three `Intl.DateTimeFormat(undefined, ...)` calls to use the active locale explicitly.
- [x] Force `hour12: false` on `formatHm` so 24-hour time is consistent across both locales — a kickoff app that flips to "06:00 PM" mid-countdown feels off.
- [x] Keep the `{ text, relative }` shape — the `relative` flag drives the `.relative` CSS class on `.event-kickoff` and shouldn't be lost.
- [x] Tests: `src/test/kickoff.test.ts` covers all six countdown phrases in both locales, the two absolute phrases (`tomorrow HH:MM` and `<weekday> HH:MM`), and explicit assertions that the active locale drives `Intl.DateTimeFormat` (e.g. en gives "07/01 18:00", pt-BR gives "01/07 18:00"). 17 tests total.

### Phase D — backend error messages ✅

Errors from the API (`/api/auth/google` failures, validation errors, admin actions) used to render their server-supplied message directly into the UI. With i18n these are translation keys, not English/Portuguese sentences.

- [x] Backend emits the wire shape `{ code, params, message }` from the four in-scope routes (`google_login`, `create_bet`, `join_group`) and `AppError::Internal`. The 11 codes are listed in `.ai/PHASE-D-CONTRACT.md`. Wire codes are snake_case; locale keys are camelCase via the `codeToLocaleKey` mapper exported from `frontend/src/api/client.ts`.
- [x] Frontend `ApiError` class catches the structured payload, `AuthContext.login` and `DevLoginButton` look up `t('errors.<key>', params)` with an English `err.message` fallback when a key is missing.
- [x] Field-level validation errors (e.g. `insufficient_balance` carries `{ have, bet }` as numeric params) follow the same pattern. New params flow through i18next interpolation; no code change required at the render site.
- [x] Out-of-scope routes (`/api/auth/me` token-decode failures, `/api/bets` list, `/api/groups` create/get/leaderboard, `/api/admin/*`, `/api/events/*`, `/api/auth/dev_login`) keep their legacy `{ "error": "<string>" }` shape via parallel `Legacy*` `AppError` variants. No churn at those call sites.
- [x] Out of scope: `Accept-Language`-driven server message templating. The code-based approach sidesteps it — the backend never produces user-facing text in this round.
- [x] Tests: 11 backend integration tests in `backend/tests/error_shape.rs` covering the in-scope shape, the canonical internal shape, and legacy-shape preservation. 14 frontend tests in `frontend/src/test/apiError.test.tsx` covering locale coverage, `insufficientBalance` interpolation, end-to-end translation through `AuthContext.login`, missing-key fallback, `ApiError` class shape, and the `codeToLocaleKey` mapper.

### Open questions ✅ (resolved)

All five open questions were resolved during planning. Decisions live in the phase sections above; this block is kept as an audit trail so the reasoning isn't lost.

- [x] **Brand title** — keep "Sobrou Nada Pro Bet" verbatim in every locale. No entry needed in locale files; Phase B skips it.
- [x] **Language switcher placement** — flag-emoji group (🇧🇷 / 🇺🇸) lives in `.header-right`, replacing the existing `.backend-status` indicator which moves to a footer. Phase A owns the wiring (last bullet under "Phase A — infrastructure").
- [x] **RTL** — defer indefinitely. No RTL string extraction, no `dir` attribute work, no RTL-specific CSS. Re-evaluate only if we ever add Arabic/Hebrew.
- [x] **Plural rules** — include one Vitest test asserting pt-BR's `>1` plural form fires at `count: 2` for any interpolated `{count} bets`-style string. Phase B test list owns it.
- [x] **Locale-aware event-time format** — Phase C owns both `kickoff.ts` and the upcoming-card date columns. Single `Intl.DateTimeFormat` call site per surface, fed from the active locale.

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
