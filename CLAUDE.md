# CLAUDE.md

You are an experienced, pragmatic software engineering AI agent. Do not over-engineer a solution when a simple one is possible. Keep edits minimal. If you want an exception to ANY rule, you MUST stop and get permission first.

## Project

Racha is a daily habit tracker with social accountability groups. Users create recurring tasks, mark them complete each day, and track streaks. Groups let friends see each other's progress.

**Stack**: Rust 2024 edition, Axum 0.8, Askama 0.15, PostgreSQL (sqlx 0.8), htmx, Tailwind CSS v4.

## Build & Run Commands

```bash
cargo check                    # Type-check Rust code
npm run check                  # Lint and format check frontend code
cargo build                    # Full build
cargo run                      # Run server (needs DATABASE_URL in .env)
cargo test                     # Run integration tests (needs running PostgreSQL)
```

### Database

PostgreSQL required. Set `DATABASE_URL` in `.env` (e.g. `postgres://user:pass@localhost/racha`). Migrations run automatically on startup via `sqlx::migrate!()`. Migration files live in `migrations/`.

### Tailwind CSS

Standalone CLI at `/tmp/tailwindcss`. Regenerate after any template HTML change:
```bash
/tmp/tailwindcss -i static/css/input.css -o static/css/output.css --minify
```

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

### Layer separation

- **Routes** (`src/routes/`) — Axum handlers. Each module exposes a `router()` fn. Handlers extract `AuthUser` (redirects to `/login` if unauthenticated), call model methods, and return template structs.
- **Models** (`src/models/`) — Data access. Each model struct derives `sqlx::FromRow` and has async methods taking `&PgPool`. All SQL uses PostgreSQL `$1, $2` parameter syntax and `RETURNING id` for inserts.
- **Templates** (`src/templates/`) — Askama template structs deriving both `Template` and `WebTemplate` (from `askama_web`). HTML lives in `templates/` at crate root. Partials prefixed with `_` are htmx fragments.
- **Auth** (`src/auth.rs`) — Password hashing (argon2), session login/logout, and `AuthUser` extractor.

## Patterns

- **Authentication**: Include `AuthUser` in a handler's parameters to enforce login. It reads `user_id` from the session; unauthenticated requests redirect to `/login`.
- **SQL style**: Use PostgreSQL `$1, $2` parameter syntax. Use `RETURNING id` for inserts.
- **htmx partials**: Task/group forms and cards are returned as HTML fragments for htmx swaps. Template files prefixed with `_` are fragments.
- **Streak calculation**: Recursive CTE in PostgreSQL walking backwards through consecutive completion dates. Lives in `TaskWithStreak::for_user`, `TaskWithStreak::find_by_id`, and `Group::member_streaks`.
- **Dead code on model fields**: Fields used only in Askama templates need `#[allow(dead_code)]` since the compiler can't see template usage.
- **Router composition**: `routes::build_router()` merges sub-routers (auth, dashboard, tasks, groups, profile). Add new feature routers there.
- **Session storage**: `tower-sessions` 0.14 with `tower-sessions-sqlx-store` 0.15, sessions stored in PostgreSQL.

## Frontend Best Practices

### CSS Architecture

- **No inline styles** — Always use Tailwind utility classes or custom CSS classes in `static/css/input.css`
- **CSS custom properties** — Use CSS variables defined in `:root` for theming (colors, spacing)
- **Component classes** — Define reusable components in `@layer components` (e.g., `.btn-gradient`, `.neu-raised`)
- **Utility classes** — Use Tailwind utilities for one-off styling; custom utilities go in `@layer utilities`
- **File organization** — Keep CSS in `static/css/input.css` with clear section comments

### JavaScript Best Practices

- **Modular code** — Organize JavaScript into logical modules in `static/js/` directory:
  - `utils.js` — Helper functions (date formatting, cookies)
  - `features/` — Feature-specific modules (task-toggle, toast notifications)
  - `main.js` — Entry point that imports and initializes modules
- **No inline scripts** — Keep all JavaScript in external files, loaded via `<script type="module">`
- **Event delegation** — Use `document.addEventListener` with checks like `e.target.closest('.selector')` for dynamic elements
- **Use `const`/`let`** — Never use `var`; prefer `const`, use `let` only when reassignment needed
- **Modern syntax** — Use arrow functions, template literals, destructuring, optional chaining

### HTML Templates (Askama)

- **Semantic HTML** — Use proper elements (`button` not `div` for clickable, `nav` for navigation)
- **Accessibility** — Include `aria-label` on icon-only buttons, proper heading hierarchy
- **No inline styles** — Use Tailwind classes exclusively; if style varies by condition, use CSS classes
- **Template partials** — Use `_` prefix for htmx fragments (e.g., `_task_card.html`)

### Code Quality Tools

- **Biome** — Linter and formatter for CSS and JavaScript (configured in `biome.json`)
  - Run `npm run lint` to check for issues
  - Run `npm run format` to auto-fix formatting
  - Run `npm run check` to verify both linting and formatting pass
- **Pre-commit** — Always run `npm run check` before committing frontend changes

### Tailwind CSS

- **Utility-first approach** — Prefer Tailwind utilities over custom CSS where available
- **Custom components** — Use `@apply` sparingly; prefer Tailwind's `@layer components` for custom patterns
- **Source scanning** — `@source "../../templates"` enables Tailwind to scan templates for class names
- **Regeneration** — Always regenerate after template changes:
  ```bash
  npm run build:css
  ```

## Dependency Version Constraints

- **`tower-sessions` must be 0.14** (not 0.15) — `tower-sessions-sqlx-store 0.15` depends on `tower-sessions-core 0.14`.
- **Do not use `rand 0.9` OsRng for argon2** — use `argon2::password_hash::rand_core::OsRng` (rand_core 0.6).
- **`askama` and `askama_web` must both be 0.15** — version mismatches break compilation.

## Commit and PR Guidelines

1. Run `cargo check` before committing. Fix all warnings and errors.
2. Run `npm run check` to verify frontend code passes linting and formatting.
3. If templates changed, regenerate Tailwind CSS and commit the updated `static/css/output.css`.
4. Run `cargo test` if tests exist for the area you changed.
5. Commit messages: `type: description` (e.g. `fix: correct streak calculation`, `feat: add profile page`).
6. Keep commits focused — one logical change per commit.
