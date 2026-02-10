# Racha - Social Streak Tracker

## Summary
A web app for tracking daily habit streaks and sharing progress with friends in social groups.

## Problem
Maintaining daily habits is hard without accountability. Existing habit trackers are solitary experiences that lack social motivation. Users want to see friends' streaks, cheer each other on, and stay consistent through group accountability.

## Proposed Solution
A lightweight web application built with a Rust backend and htmx-powered frontend where users can:
- Define daily tasks and track streaks (consecutive days completed)
- Join or create groups to share streaks with friends
- View a group feed showing members' active streaks and milestones

## Scope

**In scope:**
- User authentication (sign up, log in, log out)
- Create, edit, and delete daily tasks (habits)
- Mark tasks as done each day, auto-calculate streak counts
- Streak history and calendar view per task
- Create and join groups (via invite link/code)
- Group feed: see members' streaks, streak milestones (e.g., 7, 30, 100 days)
- Basic notifications (streak about to break, friend hit a milestone)
- Mobile-responsive UI

**Out of scope:**
- Native mobile apps
- Real-time chat or messaging
- Gamification beyond streaks (no points, badges, leaderboards)
- Third-party integrations (calendars, fitness APIs)
- Payment or subscription features

## Technical Approach

### Stack
- **Backend:** Rust with Axum
- **Frontend:** Server-rendered HTML templates (Tera or Askama) + htmx for interactivity
- **Database:** SQLite via `rusqlite` or `sqlx`
- **Auth:** Session-based auth with cookies, passwords hashed with `argon2`
- **Styling:** Tailwind CSS (standalone CLI, no Node.js required)

### Data Model (core tables)

```
users
  id (INTEGER PK)
  username (TEXT UNIQUE)
  email (TEXT UNIQUE)
  password_hash (TEXT)
  created_at (DATETIME)

tasks
  id (INTEGER PK)
  user_id (INTEGER FK -> users)
  name (TEXT)
  description (TEXT, nullable)
  created_at (DATETIME)
  archived (BOOLEAN)

completions
  id (INTEGER PK)
  task_id (INTEGER FK -> tasks)
  completed_date (DATE)
  UNIQUE(task_id, completed_date)

groups
  id (INTEGER PK)
  name (TEXT)
  invite_code (TEXT UNIQUE)
  created_by (INTEGER FK -> users)
  created_at (DATETIME)

group_members
  group_id (INTEGER FK -> groups)
  user_id (INTEGER FK -> users)
  joined_at (DATETIME)
  PRIMARY KEY (group_id, user_id)
```

### Key Routes

- `GET /` — Dashboard: user's tasks with current streak counts
- `POST /tasks` — Create a new task
- `POST /tasks/{id}/complete` — Mark task done today (htmx partial)
- `GET /groups/{id}` — Group feed with members' streaks
- `POST /groups` — Create a group
- `POST /groups/join` — Join via invite code

### Streak Calculation
- Streak = count of consecutive days ending today (or yesterday if not yet completed today) in `completions` for a given task
- Computed on read via SQL query; no denormalized streak counter needed at this scale

### htmx Patterns
- Task completion toggles via `hx-post` returning updated streak partial
- Group feed loads member streaks via `hx-get` with `hx-trigger="load"`
- Inline task creation/editing with `hx-swap="outerHTML"`

## Acceptance Criteria
- [ ] User can register, log in, and log out
- [ ] User can create, edit, and archive daily tasks
- [ ] User can mark a task as completed for today; streak count updates immediately via htmx
- [ ] Streak resets to 0 if a day is missed; correctly shows current streak length
- [ ] User can create a group and receive an invite code
- [ ] User can join a group via invite code
- [ ] Group page displays all members' tasks and current streak counts
- [ ] UI is responsive and usable on mobile browsers
- [ ] SQLite database initializes schema on first run (migrations)
- [ ] Passwords are hashed; sessions are secure (HttpOnly, SameSite cookies)

## Open Questions
- Should users be able to backfill missed days (e.g., "I forgot to check in yesterday")?
- Should streak milestones trigger visible celebrations in the group feed?

