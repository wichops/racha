# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Run Commands

```bash
cargo build                    # Build the project
cargo run                      # Run the server (requires DATABASE_URL env var or .env file)
cargo check                    # Type-check without building
```

There are no tests yet. No linter or formatter is configured beyond `cargo check`.

### Database

PostgreSQL is required. Set `DATABASE_URL` in `.env` (e.g. `DATABASE_URL=postgres://user:pass@localhost/racha`). Migrations run automatically on startup via `sqlx::migrate!()`.

### Tailwind CSS

Standalone CLI binary at `/tmp/tailwindcss`. Regenerate after template changes:
```bash
/tmp/tailwindcss -i static/css/input.css -o static/css/output.css --minify
```

## Architecture

Racha is a daily habit tracker with social accountability groups. Users create recurring tasks, mark them complete each day, and track streaks. Groups let friends see each other's streaks.

**Stack**: Rust 2024 edition, Axum 0.8, Askama 0.15 templates, PostgreSQL (sqlx 0.8), htmx for interactivity, Tailwind CSS v4.

### Request flow

`src/main.rs` builds the Axum router via `routes::build_router()` which merges four sub-routers (auth, dashboard, tasks, groups). Session middleware (`tower-sessions` 0.14 + `tower-sessions-sqlx-store` 0.15) provides cookie-based sessions stored in PostgreSQL.

### Layer separation

- **Routes** (`src/routes/`) — Axum handlers. Each module exposes a `router()` fn. Handlers extract `AuthUser` (redirects to `/login` if unauthenticated), call model methods, and return template structs.
- **Models** (`src/models/`) — Data access. Each model struct derives `sqlx::FromRow` and has async methods taking `&PgPool`. All SQL uses PostgreSQL `$1, $2` parameter syntax and `RETURNING id` for inserts.
- **Templates** (`src/templates/`) — Askama template structs deriving both `Template` and `WebTemplate` (from `askama_web`). HTML lives in `templates/` at crate root. Partials prefixed with `_` are htmx fragments.
- **Auth** (`src/auth.rs`) — Password hashing (argon2), session login/logout, and `AuthUser` extractor.

### Key patterns

- `AuthUser` is an Axum `FromRequestParts` extractor that reads `user_id` from the session. Including it in a handler signature enforces authentication.
- htmx partials: task and group forms/cards are returned as HTML fragments for htmx swaps. Full pages extend `templates/base.html`.
- Streak calculation uses a recursive CTE in PostgreSQL that walks backwards through consecutive completion dates. This logic exists in `TaskWithStreak::for_user`, `TaskWithStreak::find_by_id`, and `Group::member_streaks`.
- Model struct fields used only in templates need `#[allow(dead_code)]` since the Rust compiler can't see Askama template usage.

## Dependency version constraints

- `tower-sessions` must be **0.14** (not 0.15) because `tower-sessions-sqlx-store 0.15` depends on `tower-sessions-core 0.14`.
- `argon2`'s `SaltString::generate` requires `rand_core 0.6` OsRng (re-exported as `argon2::password_hash::rand_core::OsRng`), not `rand 0.9`'s OsRng.
- `askama_web` version tracks askama (both 0.15).
