use axum::{
    Router,
    extract::{State, Path},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
    Form,
    http::StatusCode,
};
use serde::Deserialize;

use crate::AppState;
use crate::auth::AuthUser;
use crate::models::group::{Group, MemberWithStreaks};
use crate::templates::groups::{GroupFeedTemplate, CreateGroupFormPartial, JoinGroupFormPartial};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/groups", post(create_group))
        .route("/groups/create-form", get(create_form))
        .route("/groups/join", post(join_group))
        .route("/groups/join-form", get(join_form))
        .route("/groups/{id}", get(group_feed))
}

async fn create_form(_user: AuthUser) -> CreateGroupFormPartial {
    CreateGroupFormPartial
}

async fn join_form(_user: AuthUser) -> JoinGroupFormPartial {
    JoinGroupFormPartial
}

#[derive(Deserialize)]
struct CreateGroupForm {
    name: String,
}

async fn create_group(
    State(state): State<AppState>,
    user: AuthUser,
    Form(form): Form<CreateGroupForm>,
) -> Redirect {
    let _ = Group::create(&state.db, &form.name, user.id).await;
    Redirect::to("/")
}

#[derive(Deserialize)]
struct JoinGroupForm {
    invite_code: String,
}

async fn join_group(
    State(state): State<AppState>,
    user: AuthUser,
    Form(form): Form<JoinGroupForm>,
) -> Redirect {
    let code = form.invite_code.trim().to_uppercase();
    if let Ok(Some(group)) = Group::find_by_invite_code(&state.db, &code).await {
        let _ = Group::join(&state.db, group.id, user.id).await;
    }
    Redirect::to("/")
}

async fn group_feed(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<i64>,
) -> Response {
    let group = match Group::find_by_id(&state.db, id).await {
        Ok(Some(g)) => g,
        _ => return StatusCode::NOT_FOUND.into_response(),
    };
    let streaks = Group::member_streaks(&state.db, id).await.unwrap_or_default();
    let members_grouped = group_streaks_by_member(streaks);

    GroupFeedTemplate {
        group,
        members_grouped,
        flash_message: None,
        flash_is_error: false,
    }
    .into_response()
}

fn group_streaks_by_member(streaks: Vec<MemberWithStreaks>) -> Vec<(String, Vec<MemberWithStreaks>)> {
    let mut grouped: Vec<(String, Vec<MemberWithStreaks>)> = Vec::new();
    for streak in streaks {
        if let Some(last) = grouped.last_mut() {
            if last.0 == streak.username {
                last.1.push(streak);
                continue;
            }
        }
        let username = streak.username.clone();
        grouped.push((username, vec![streak]));
    }
    grouped
}
