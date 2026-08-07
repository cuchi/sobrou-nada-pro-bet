# Phase D — Backend error code contract (frozen)

This is the **single source of truth** for the wire shape, code list, and
parameter shapes. Both the Rust and TypeScript agents must conform to this
exactly. If anything below needs to change, update this file first and
notify both agents.

## Wire shape

Every 4xx and 5xx response from the API uses the same JSON shape:

```json
{
  "code": "<snake_case_code>",
  "params": { ... } | null,
  "message": "<English fallback string, always populated>"
}
```

- `code`: a stable snake_case identifier. Null is **never** valid; every
  error has exactly one code.
- `params`: an object with typed parameters the frontend interpolates
  into the translated template, OR `null` if the code takes no parameters.
- `message`: an English fallback string, always populated. This is what a
  developer sees when calling the API via curl or reading server logs.
  The frontend does NOT display this directly — it uses `code` to look
  up the active locale's translation.

Status codes follow the existing `AppError` → HTTP mapping:

| Variant             | HTTP status       |
|---------------------|-------------------|
| `Unauthorized`      | 401 Unauthorized  |
| `Forbidden`         | 403 Forbidden     |
| `NotFound`          | 404 Not Found     |
| `BadRequest`        | 400 Bad Request   |
| `Internal`          | 500 Internal Error|

## Codes (Phase D scope = strict, Q4a)

These are the codes the **frontend can render today** (i.e. surfaced via
the four "trust the server" API client callers: `googleLogin`, `devLogin`,
`createBet`, `joinGroup`). Other routes keep their current free-text
behavior — they may emit `{ code: null, params: null, message: "..." }`
or simply continue to send the old shape; both are acceptable in this
phase.

| Code                       | HTTP | Params              | Emitted from                                          |
|----------------------------|------|---------------------|-------------------------------------------------------|
| `auth_google_failed`       | 401  | `null`              | `routes/auth.rs::google_login` (network/parse)        |
| `auth_google_invalid`      | 401  | `null`              | `routes/auth.rs::google_login` (aud/sub/email)        |
| `auth_not_on_allowlist`    | 403  | `null`              | `routes/auth.rs::google_login` (beta gate)            |
| `not_group_member`         | 403  | `null`              | `routes/bets.rs::create_bet` (membership check)       |
| `insufficient_balance`     | 400  | `{have: f64, bet: f64}` | `routes/bets.rs::create_bet`                      |
| `already_bet_on_event`     | 400  | `null`              | `routes/bets.rs::create_bet` (duplicate check)        |
| `event_not_found`          | 400  | `null`              | `routes/bets.rs::create_bet` (event lookup)           |
| `betting_closed`           | 400  | `null`              | `routes/bets.rs::create_bet` (kickoff cutoff)         |
| `invalid_invite_code`      | 404  | `null`              | `routes/groups.rs::join_group` (invite lookup)        |
| `already_in_group`         | 400  | `null`              | `routes/groups.rs::join_group` (duplicate member)     |
| `internal`                 | 500  | `null` (always)     | `AppError::Internal`                                  |

The `internal` code is the only 5xx code; it always has `params: null` and
`message: "Internal server error"`.

### Out of scope (still free-text this phase)

Routes whose errors do **not** reach the four client callers above:
- `routes/auth.rs::me`, `routes/auth.rs::dev_login` (dev_login has no
  4xx surface today)
- `routes/bets.rs::list_bets`
- `routes/events.rs::*`
- `routes/groups.rs::{create_group, get_group, get_invite, regenerate_invite, leaderboard}`
- `routes/admin.rs::*`
- `auth.rs`'s token-decoding failures (`Missing Authorization header`,
  `Expected Bearer token`, `Invalid token: ...`, `Invalid user id`)

These may continue to emit the old `{ "error": "<English string>" }`
shape this phase, or the new shape with `code: null` — both are valid.
Frontend code only has to handle the new shape for the four callers in
scope; everywhere else the existing fallback strings still apply.

## Rust typing

`backend/src/error.rs` defines:

```rust
#[derive(Debug, Clone, thiserror::Error)]
pub enum ErrorCode {
    #[error("Failed to verify Google token")]
    AuthGoogleFailed,
    #[error("Invalid Google token")]
    AuthGoogleInvalid,
    #[error("This app is currently in closed beta. Contact paulo@cuchi.me to request access.")]
    AuthNotOnAllowlist,
    #[error("You're not a member of this group")]
    NotGroupMember,
    #[error("Insufficient balance. You have {have:.0} points, bet is {bet:.0}.")]
    InsufficientBalance { have: f64, bet: f64 },
    #[error("You already have a pending bet on this event")]
    AlreadyBetOnEvent,
    #[error("Event not found")]
    EventNotFound,
    #[error("Bets close 1 hour before kickoff")]
    BettingClosed,
    #[error("Invalid invite code")]
    InvalidInviteCode,
    #[error("You're already in this group")]
    AlreadyInGroup,
    #[error("Internal server error")]
    Internal,
}

impl ErrorCode {
    /// Stable snake_case wire identifier.
    pub fn as_code(&self) -> &'static str { ... }

    /// JSON params object, or `null` if this code takes no parameters.
    pub fn as_params(&self) -> serde_json::Value { ... }
}
```

`AppError` keeps its existing variants but now wraps a code + an
optional English detail string. Concretely:

```rust
pub enum AppError {
    Unauthorized(ErrorCode, Option<String>),
    Forbidden(ErrorCode, Option<String>),
    NotFound(ErrorCode, Option<String>),
    BadRequest(ErrorCode, Option<String>),
    Internal(String),  // unchanged: detail logged, never sent
}
```

The optional second string is for the **English fallback message** the
client displays when no translation exists. Most codes define the message
via `ErrorCode`'s `Display` impl; the optional string lets a call site
override (e.g. for the `insufficient_balance` `{have:.0} / {bet:.0}`
formatting — actually no, that formatting lives in `Display`, so the
override is rarely needed).

`AppError::into_response` emits:

```json
{ "code": "...", "params": {...} | null, "message": "..." }
```

with HTTP status from the variant.

### Code → status mapping (Rust)

- `Unauthorized` → 401
- `Forbidden` → 403
- `NotFound` → 404
- `BadRequest` → 400
- `Internal` → 500 (always `code: "internal"`, `params: null`, `message: "Internal server error"`)

## TypeScript typing

`frontend/src/api/client.ts` defines a custom error class:

```ts
export class ApiError extends Error {
  constructor(
    public code: string,
    public params: Record<string, unknown> | null,
    public message: string,
  ) {
    super(message);
    this.name = 'ApiError';
  }
}
```

The four in-scope callers (`googleLogin`, `devLogin`, `createBet`,
`joinGroup`) read `res.json()`, extract `{ code, params, message }`,
and throw `new ApiError(code, params, message)`.

`AuthContext.login` and `DevLoginButton` (the only two render sites)
catch `ApiError` and call `t(`errors.${codeToLocaleKey(err.code)}`,
err.params ?? {})`. If the locale file is missing a key for that code,
fall back to `err.message` (English) so the UI is never blank.

### Code → key mapping

Wire codes are snake_case; locale keys are camelCase. The mapping is a
straightforward underscore-to-camelCase transform:

```
auth_google_failed       → authGoogleFailed
auth_google_invalid      → authGoogleInvalid
auth_not_on_allowlist    → authNotOnAllowlist
not_group_member         → notGroupMember
insufficient_balance     → insufficientBalance
already_bet_on_event     → alreadyBetOnEvent
event_not_found          → eventNotFound
betting_closed           → bettingClosed
invalid_invite_code      → invalidInviteCode
already_in_group         → alreadyInGroup
internal                 → internal
```

`frontend/src/api/client.ts` exports `codeToLocaleKey(code: string):
string` which performs this transform. Render sites use it rather than
hard-coding the map. Any new code added in a future phase gets the
transform for free — the locale-key naming convention (camelCase under
`errors.*`) is the contract; the map is mechanical.

### Locale keys (both `en` and `pt-BR`)

Under `errors.*`:

- `authGoogleFailed` → "Google login failed" / "Falha no login com Google"
- `authGoogleInvalid` → "Google sign-in failed" / "Falha no login com Google"
- `authNotOnAllowlist` → "This app is currently in closed beta. Contact paulo@cuchi.me to request access." (verbatim, in both locales — Portuguese audience expects the literal email address)
- `notGroupMember` → "You're not a member of this group" / "Você não faz parte deste grupo"
- `insufficientBalance` → "Not enough points — you have {{have}} and tried to bet {{bet}}" / "Pontos insuficientes — você tem {{have}} e tentou apostar {{bet}}"
- `alreadyBetOnEvent` → "You already have a pending bet on this event" / "Você já tem uma aposta pendente neste evento"
- `eventNotFound` → "Event not found" / "Evento não encontrado"
- `bettingClosed` → "Betting is closed for this match" / "Apostas encerradas para esta partida"
- `invalidInviteCode` → "Invalid invite code" / "Código de convite inválido"
- `alreadyInGroup` → "You're already in this group" / "Você já faz parte deste grupo"
- `internal` → "Something went wrong. Please try again." / "Algo deu errado. Tente novamente."

The existing `errors.googleLogin` / `errors.devLogin` etc. fallbacks
stay as-is for the **out-of-scope** callers.

## Backwards compatibility

None. This is a hard cutover. Both halves of the change ship in the
same commit.