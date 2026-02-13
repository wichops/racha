You are an experienced, pragmatic software engineering AI agent. Do not over-engineer a solution when a simple one is possible. Keep edits minimal. If you want an exception to ANY rule, you MUST stop and get permission first.

# Racha

Daily habit tracker with social accountability groups. Users create recurring tasks, mark them complete each day, and track streaks. Groups let friends see each other's progress.

**Stack**: Rust 2024 edition, Axum 0.8, Askama 0.15, PostgreSQL (sqlx 0.8), htmx, Tailwind CSS v4.

## Essential Commands

```bash
cargo check                    # Type-check (primary validation — no linter/formatter configured)
cargo build                    # Full build
cargo run                      # Run server (needs DATABASE_URL in .env)
cargo test                     # Run integration tests (needs running PostgreSQL)
```

### Tailwind CSS

Regenerate after any template HTML change:
```bash
/tmp/tailwindcss -i static/css/input.css -o static/css/output.css --minify
```

### Database

PostgreSQL required. Set `DATABASE_URL` in `.env` (e.g. `postgres://user:pass@localhost/racha`). Migrations run automatically on startup via `sqlx::migrate!()`. Migration files live in `migrations/`.

## Code Structure

```
src/
  main.rs          # Entry point, builds router via routes::build_router()
  auth.rs          # Password hashing (argon2), session, AuthUser extractor
  config.rs        # Configuration loading
  db.rs            # Database pool setup
  lib.rs           # Library root
  models/          # Data access — structs derive sqlx::FromRow, async methods take &PgPool
    user.rs, task.rs, completion.rs, group.rs
  routes/          # Axum handlers — each module exposes router() fn
    auth.rs, dashboard.rs, tasks.rs, groups.rs, profile.rs
  templates/       # Askama template structs (derive Template + WebTemplate)
templates/         # HTML templates; partials prefixed with _ are htmx fragments
  base.html        # Layout base; full pages extend this
static/            # CSS, JS assets
tests/             # Integration tests using axum_test + real PgPool
  common/          # Shared test helpers (server setup)
migrations/        # SQL migrations (auto-applied on startup)
```

## Patterns

- **Authentication**: Include `AuthUser` in a handler's parameters to enforce login. It reads `user_id` from the session; unauthenticated requests redirect to `/login`.
- **SQL style**: Use PostgreSQL `$1, $2` parameter syntax. Use `RETURNING id` for inserts.
- **htmx partials**: Task/group forms and cards are returned as HTML fragments for htmx swaps. Template files prefixed with `_` are fragments.
- **Streak calculation**: Recursive CTE in PostgreSQL walking backwards through consecutive completion dates. Lives in `TaskWithStreak::for_user`, `TaskWithStreak::find_by_id`, and `Group::member_streaks`.
- **Dead code on model fields**: Fields used only in Askama templates need `#[allow(dead_code)]` since the compiler can't see template usage.
- **Router composition**: `routes::build_router()` merges sub-routers (auth, dashboard, tasks, groups, profile). Add new feature routers there.
- **Session storage**: `tower-sessions` 0.14 with `tower-sessions-sqlx-store` 0.15, sessions stored in PostgreSQL.

## Anti-Patterns

- **Do not upgrade `tower-sessions` to 0.15** — `tower-sessions-sqlx-store 0.15` depends on `tower-sessions-core 0.14`.
- **Do not use `rand 0.9` OsRng for argon2** — use `argon2::password_hash::rand_core::OsRng` (rand_core 0.6).
- **Do not use askama/askama_web version mismatches** — both must be 0.15.

## Commit and PR Guidelines

1. Run `cargo check` before committing. Fix all warnings and errors.
2. If templates changed, regenerate Tailwind CSS and commit the updated `static/css/output.css`.
3. Run `cargo test` if tests exist for the area you changed.
4. Commit messages: `type: description` (e.g. `fix: correct streak calculation`, `feat: add profile page`).
5. Keep commits focused — one logical change per commit.
