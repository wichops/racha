use axum::{
    Router,
    extract::{State, Path},
    response::{IntoResponse, Response},
    routing::{get, post},
    Form,
    http::StatusCode,
};
use serde::Deserialize;

use crate::AppState;
use crate::auth::AuthUser;
use crate::models::task::{Task, TaskWithStreak};
use crate::models::completion;
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
    Form(form): Form<CreateTaskForm>,
) -> Response {
    let desc = form.description.as_deref().filter(|s| !s.is_empty());
    match Task::create(&state.db, user.id, &form.name, desc).await {
        Ok(task_id) => {
            match TaskWithStreak::find_by_id(&state.db, task_id).await {
                Ok(Some(task)) => TaskCardPartial { task }.into_response(),
                _ => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            }
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn toggle_task(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> Response {
    let current = match TaskWithStreak::find_by_id(&state.db, id).await {
        Ok(Some(task)) if task.user_id == user.id => task,
        Ok(Some(_)) => return StatusCode::FORBIDDEN.into_response(),
        _ => return StatusCode::NOT_FOUND.into_response(),
    };

    if current.completed_today {
        let _ = completion::uncomplete_today(&state.db, id).await;
    } else {
        let _ = completion::complete_today(&state.db, id).await;
    }

    match TaskWithStreak::find_by_id(&state.db, id).await {
        Ok(Some(task)) => TaskCardPartial { task }.into_response(),
        _ => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn edit_form(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> Response {
    match TaskWithStreak::find_by_id(&state.db, id).await {
        Ok(Some(task)) if task.user_id == user.id => {
            TaskEditPartial { task }.into_response()
        }
        _ => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn task_card(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> Response {
    match TaskWithStreak::find_by_id(&state.db, id).await {
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
    Path(id): Path<i64>,
    Form(form): Form<UpdateTaskForm>,
) -> Response {
    let desc = form.description.as_deref().filter(|s| !s.is_empty());
    let _ = Task::update(&state.db, id, user.id, &form.name, desc).await;

    match TaskWithStreak::find_by_id(&state.db, id).await {
        Ok(Some(task)) if task.user_id == user.id => {
            TaskCardPartial { task }.into_response()
        }
        _ => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn archive_task(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> StatusCode {
    let _ = Task::archive(&state.db, id, user.id).await;
    StatusCode::OK
}
