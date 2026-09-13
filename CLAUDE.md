# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Simple Budget is a full-stack "envelope budgeting" web app:

- **Backend**: Rust (edition 2024) using Axum, server-rendered HTML via Handlebars, Hotwire (Turbo + Stimulus) for HTML-over-the-wire interactivity.
- **Frontend**: TypeScript Stimulus controllers + TailwindCSS, bundled by Vite into `static/` (single library entry `assets/src/index.ts` → `static/index.mjs`/`index.css`).
- **Database**: PostgreSQL, accessed via raw SQL through `tokio-postgres`/`rust-database-common` (no ORM, no query builder).
- **Auth**: OpenID Connect (Google Sign-In), session cookies signed with `axum-extra`'s `SignedCookieJar`.

## Commands

### Rust backend

- `cargo build` / `cargo run` — build/run the server (listens on `0.0.0.0:8000`)
- `cargo test` — run the full test suite (needs `TEST_DATABASE_URL`, see below)
- `cargo test <path::to::test_fn>` — run a single test
- `cargo clippy --no-deps -- -D warnings` — lint exactly as CI does (`branch-protection.yml`)
- `cargo fmt` — format
- `bacon` (config in `bacon.toml`) — watch mode; `bacon test`, `bacon clippy-all`, `bacon run-long` etc.

Tests need a real Postgres instance (no mocking of the DB layer):

```bash
docker run -d -p 5432:5432 -e POSTGRES_HOST_AUTH_METHOD=trust \
  -e POSTGRES_USER=simple_budget -e POSTGRES_DB=simple_budget_test \
  -v ./migrations/schema.sql:/docker-entrypoint-initdb.d/schema.sql postgres
TEST_DATABASE_URL="postgresql://simple_budget@localhost:5432/simple_budget_test" cargo test
```

`compose.yaml` starts a dev-mode Postgres (loaded from `migrations/schema.sql`) on port 5432.

Required env vars are documented in `.env` (sourced with `source .env`) / README: `DATABASE_URL`, `SECRET_KEY`, `GOOGLE_CALLBACK_URL`, `GOOGLE_CLIENT_ID`, `GOOGLE_CLIENT_SECRET`; optional `TEST_DATABASE_URL`, `LOG_LEVEL`, `METRICS_ENDPOINT`, `TRACING_ENDPOINT`, `DATABASE_CA_CERT`.

### Frontend (pnpm workspace)

- `pnpm install` then `pnpm build` (or `npm run build`) — runs `vite build`, output goes to `static/`
- `npx tailwindcss -i input.css -o static/app.css [--watch]` for CSS during local dev

### CI

- `.github/workflows/branch-protection.yml` runs on every non-`main` push: `cargo clippy --no-deps -- -D warnings` and `cargo test` against a throwaway Postgres container. Both must pass (`check-success` job gates on them).
- `.github/workflows/continuous-delivery.yaml` builds and pushes the Docker image on push to `main` (no tests run here — rely on branch-protection).

## Architecture

### Request flow

`src/main.rs` builds one `Router` composed of:

- `authentication::authentication_router()` — unauthenticated OIDC login/callback routes
- `authenticated::authenticated_router(state)` — everything behind session auth, gated by two `route_layer`s applied in this order: `authenticated` (session/cookie check, redirects to login) then `validate_csrf` (checks `x-csrf-token` header against the session's CSRF token for POST/PUT/PATCH/DELETE)
- a `/healthcheck` route
- static assets under `/assets` served from `static/`, cache-headers added by `middleware::cache_assets`

Two more `from_fn` layers apply globally: `middleware::inject_context` (stamps a per-request CSP nonce into a `HandlebarsContext` extension) and `middleware::secure_headers` (reads that nonce back out and sets the `Content-Security-Policy` header). Handler-level middleware in each authenticated submodule (e.g. `authenticated::goals::initialize_context`) further enriches the `HandlebarsContext` with page-specific data (`section`, `csrf`, etc.) before it reaches the Handlebars template.

Handlebars templates are compiled at startup by walking `./templates` recursively (`utilities::handlebars::walk_directory`) and registered under a name equal to their path relative to `templates/` with `.hbs` stripped — so `templates/goals/index.hbs` is referenced in Rust as `"goals/index"`.

### Route module layout

Each resource under `src/authenticated/` (accounts, envelopes, goals, preferences) follows the same shape: a `mod.rs`-equivalent file defines a `<resource>_router()` plus a JSON-schema-validated `<Resource>Form` struct and a scoped `initialize_context` middleware, with one file per action (`index.rs`, `new.rs`, `create.rs`, `edit.rs`, `update.rs`, `delete.rs`). Form payloads are validated against a `jsonschema`-built schema (see `goals.rs::schema()`) before being deserialized into the form struct.

### Models (`src/models/`)

Each model (`Account`, `Envelope`, `Goal`, `User`) hand-writes its own SQL via `impl TryInto<T> for tokio_postgres::Row` plus `create`/`update`/`delete`/`get_*` methods taking `&impl GenericClient` (so the same code works against a plain connection or an open transaction). All handler/model errors funnel through `AppError` (`src/errors.rs`), which maps DB/template/etc. errors to HTTP responses; `AppResponse = Result<Response, AppError>` is the standard handler return type.

`Goal` carries the core domain logic: `Recurrence` (Never/Daily/Weekly/Monthly/Quarterly/Yearly) drives `increment()` (roll target_date forward when a goal cycle expires), `accumulate()`/`accumulated_now()` (linear accrual toward `target` between `start_at()` and `target_date`, driven by an injected `Times` clock, not wall-clock time directly), and `accelerate()` (add a lump sum, capped at `target`).

### Background jobs (`src/jobs/`)

`start_background_jobs()` in `main.rs` spawns a tokio task that ticks every 60s and runs `clear_sessions` and `convert_goals` concurrently via `tokio::join!`. `convert_goals` (`src/jobs/convert_goals.rs`) is the most complex job: it (1) converts expired goals into `Envelope`s and rolls their recurrence, (2) accumulates all goals toward their target, then (3) optionally "accelerates" goals using unspent monthly income, gated by the user's `Preferences.accelerate_goals` / `accelerate_non_monthly` flags — acceleration only ever applies extra funds beyond ordinary accrual, and non-monthly goals only accelerate if `accelerate_non_monthly` is explicitly enabled.

### Time abstraction

Anything date/time-dependent takes a `&impl Times` (see `utilities/dates.rs`) instead of calling `Utc::now()` directly, so tests can inject a fixed `MockTimeProvider`. Follow this pattern for new time-sensitive logic.

### Preferences

`User.preferences` is a `postgres_types::Json<Preferences>` column (`src/models/user/preferences.rs`). All fields are `Option<T>` with accessor methods (`timezone()`, `monthly_income()`, `accelerate_goals()`, `accelerate_non_monthly()`) that supply defaults — always read preferences through these accessors rather than the raw `Option` fields.

### Testing conventions

- `src/test_utils.rs` (test-only module) provides `state_for_tests()` (builds a `SharedState` + fake authenticated user/CSRF extension against `TEST_DATABASE_URL`) and `user_for_tests()`.
- Tests that mutate shared tables (goals, accounts) generally wrap work in a transaction and `rollback()` at the end rather than relying on fixture teardown; some suites (e.g. `convert_goals`) intentionally run sub-tests sequentially from one `#[tokio::test]` runner function because parallel DB access across them causes deadlocks — check for a `_runner` test before assuming tests run independently.
