# Feature: Sign-in (SuperTokens with Authentik)

Status: **implemented** — the web app signs people in through the
household's Authentik (OIDC), with SuperTokens sessions. The backend asks the
web app's private verifier about the session token on every household route.
Steps for people: `docs/human-setup.md` §2 and `docs/production.md`
("SuperTokens Core") — keep those in step with this spec.

## Household model
- Everyone signed in shares one grocery list. There are no admin roles.
- Who may sign in is decided in Authentik: the application is bound to the
  household group. The app itself has no list of people.
- Per-person records (dislikes, preferences) key on the SuperTokens user id,
  `AuthUser.id`.

## What needs a session
| Route | How it is checked |
|---|---|
| Every `/api/*` household route | `AuthUser` extractor: `Authorization: Bearer <access token>` |
| `GET /ws` (live counts) | `SocketUser` extractor (`backend/src/auth/socket.rs`); credentials in the URL are ignored |
| `POST /api/intake/alexa` | Alexa bridge shared secret — not a session |
| `POST /api/voice-requests` | webhook shared secret — not a session |
| `/api/store-tab/*` | store-tab shared secret — not a session |
| `GET /api/health` | open |

`backend/tests/auth_required.rs` sends every household route a request with
no token and with a forged one, and expects 401. **A new route goes in its
list.**

## How a session is checked
1. The web app's server (`frontend/src/lib/server/auth-config.ts`) runs the
   SuperTokens Node SDK against SuperTokens Core (`SUPERTOKENS_CONNECTION_URI`,
   `SUPERTOKENS_API_KEY`). Sessions use header tokens, not cookies.
2. Sign-in is the ThirdParty recipe with one OIDC provider, Authentik
   (`AUTH_OIDC_DISCOVERY_URL`, client id and secret). The ID token's issuer
   must match the discovery URL, its `sub` is required, and the callback must
   be exactly `${AUTH_PUBLIC_ORIGIN}/sign-in/callback`.
3. The backend (`backend/src/auth/supertokens.rs`) posts the bearer token to
   `AUTH_VERIFIER_URL` (`/api/auth/internal/verify` on the web app) with
   `x-grocery-auth-secret: AUTH_VERIFIER_SECRET`. 200 `{id, email}` signs the
   request in; 401 is "Invalid or expired session"; anything else is 503,
   never a sign-in.
4. `backend/src/auth/mod.rs` is the only place that names the provider
   (`build_provider`). Swap provider there only.

Tests: `backend/tests/supertokens_sessions.rs` (valid session, verifier
failure never grants access, WebSocket protocol and URL credentials).

## Test mode (sign-in off)
The e2e suite runs with sign-in off on both sides: `DEV_AUTH_BYPASS=true` on
the backend and `VITE_AUTH_MODE=off` on the web app (`lib/auth-mode.ts`). With
sign-in off but the bypass **not** set, the backend refuses everything — the
backend is always the real gate.

## Decisions
- **No authorised-party (`azp`) check.** That was a Clerk concern (#83). Here
  the access token is issued by this app's own SuperTokens Core, sent only as
  a bearer header, and verified by this app's own SDK; Authentik's ID token
  is checked for issuer and callback before a session exists.
- **Going beyond the home network** (#94) needs no "production instance":
  Core and Authentik are self-hosted. Set `AUTH_PUBLIC_ORIGIN` to the public
  HTTPS origin, register its exact callback in Authentik, and add the origin
  to `CORS_ORIGINS` (`docs/human-setup.md` "Optional: web app on your own
  computer", `docs/production.md` "SuperTokens Core").
