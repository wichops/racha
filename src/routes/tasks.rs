use askama::Template;
use axum::{
    Router,
    extract::{State, Path},
    response::{IntoResponse, Response},
    routing::{get, post},
    Form,
    http::StatusCode,
};
use chrono::NaiveDate;
use serde::Deserialize;
use sqlx::PgPool;

use crate::AppState;
use crate::auth::{AuthUser, LocalDate};
use crate::models::task::{Task, TaskWithStreak};
use crate::models::completion;
use crate::templates::dashboard::ProgressOobPartial;
use crate::templates::tasks::{TaskCardPartial, TaskFormPartial, TaskEditPartial};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/tasks", post(create_task))
        .route("/tasks/form", get(task_form))
        .route("/tasks/{id}/toggle", post(toggle_task))
        .route("/tasks/{id}/edit", get(edit_form).post(update_task))
        .route("/tasks/{id}/card", get(task_card))
        .route("/tasks/{id}/archive", post(archive_task))
}

fn render_progress_oob(tasks: &[TaskWithStreak]) -> String {
    let total_count = tasks.len() as i64;
    let completed_count = tasks.iter().filter(|t| t.completed_today).count() as i64;
    let active_streak_count = tasks.iter().filter(|t| t.current_streak > 0).count() as i64;
    let longest_streak = tasks.iter().map(|t| t.current_streak).max().unwrap_or(0);

    ProgressOobPartial { completed_count, total_count, active_streak_count, longest_streak }
        .render()
        .unwrap_or_default()
}

async fn fetch_progress_oob(db: &PgPool, user_id: i64, today: NaiveDate) -> String {
    let tasks = TaskWithStreak::for_user(db, user_id, today).await.unwrap_or_default();
    render_progress_oob(&tasks)
}

async fn task_form(_user: AuthUser) -> TaskFormPartial {
    TaskFormPartial
}

#[derive(Deserialize)]
struct CreateTaskForm {
    name: String,
    description: Option<String>,
}

async fn create_task(
    State(state): State<AppState>,
    user: AuthUser,
    LocalDate(today): LocalDate,
    Form(form): Form<CreateTaskForm>,
) -> Response {
    let desc = form.description.as_deref().filter(|s| !s.is_empty());
    match Task::create(&state.db, user.id, &form.name, desc).await {
        Ok(task_id) => {
            match TaskWithStreak::find_by_id(&state.db, task_id, today).await {
                Ok(Some(task)) => {
                    let card = TaskCardPartial { task }.render().unwrap_or_default();
                    let progress = fetch_progress_oob(&state.db, user.id, today).await;
                    axum::response::Html(format!("{card}{progress}")).into_response()
                }
                _ => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            }
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn toggle_task(
    State(state): State<AppState>,
    user: AuthUser,
    LocalDate(today): LocalDate,
    Path(id): Path<i64>,
) -> Response {
    let current = match TaskWithStreak::find_by_id(&state.db, id, today).await {
        Ok(Some(task)) if task.user_id == user.id => task,
        Ok(Some(_)) => return StatusCode::FORBIDDEN.into_response(),
        _ => return StatusCode::NOT_FOUND.into_response(),
    };

    if current.completed_today {
        let _ = completion::uncomplete_today(&state.db, id, today).await;
    } else {
        let _ = completion::complete_today(&state.db, id, today).await;
    }

    match TaskWithStreak::find_by_id(&state.db, id, today).await {
        Ok(Some(task)) => {
            let card = TaskCardPartial { task }.render().unwrap_or_default();
            let progress = fetch_progress_oob(&state.db, user.id, today).await;
            axum::response::Html(format!("{card}{progress}")).into_response()
        }
        _ => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn edit_form(
    State(state): State<AppState>,
    user: AuthUser,
    LocalDate(today): LocalDate,
    Path(id): Path<i64>,
) -> Response {
    match TaskWithStreak::find_by_id(&state.db, id, today).await {
        Ok(Some(task)) if task.user_id == user.id => {
            TaskEditPartial { task }.into_response()
        }
        _ => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn task_card(
    State(state): State<AppState>,
    user: AuthUser,
    LocalDate(today): LocalDate,
    Path(id): Path<i64>,
) -> Response {
    match TaskWithStreak::find_by_id(&state.db, id, today).await {
        Ok(Some(task)) if task.user_id == user.id => {
            TaskCardPartial { task }.into_response()
        }
        _ => StatusCode::NOT_FOUND.into_response(),
    }
}

#[derive(Deserialize)]
struct UpdateTaskForm {
    name: String,
    description: Option<String>,
}

async fn update_task(
    State(state): State<AppState>,
    user: AuthUser,
    LocalDate(today): LocalDate,
    Path(id): Path<i64>,
    Form(form): Form<UpdateTaskForm>,
) -> Response {
    let desc = form.description.as_deref().filter(|s| !s.is_empty());
    let _ = Task::update(&state.db, id, user.id, &form.name, desc).await;

    match TaskWithStreak::find_by_id(&state.db, id, today).await {
        Ok(Some(task)) if task.user_id == user.id => {
            TaskCardPartial { task }.into_response()
        }
        _ => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn archive_task(
    State(state): State<AppState>,
    user: AuthUser,
    LocalDate(today): LocalDate,
    Path(id): Path<i64>,
) -> Response {
    let _ = Task::archive(&state.db, id, user.id).await;
    let progress = fetch_progress_oob(&state.db, user.id, today).await;
    axum::response::Html(progress).into_response()
}
