use axum::{
    Router,
    extract::State,
    routing::get,
};

use crate::AppState;
use crate::auth::AuthUser;
use crate::models::task::TaskWithStreak;
use crate::models::user::User;
use crate::models::group::Group;
use crate::templates::dashboard::DashboardTemplate;

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(dashboard))
}

async fn dashboard(
    State(state): State<AppState>,
    user: AuthUser,
) -> DashboardTemplate {
    let db_user = User::find_by_id(&state.db, user.id).await.ok().flatten();
    let username = db_user.map(|u| u.username).unwrap_or_default();
    let tasks = TaskWithStreak::for_user(&state.db, user.id).await.unwrap_or_default();
    let groups = Group::user_groups(&state.db, user.id).await.unwrap_or_default();

    DashboardTemplate {
        username,
        tasks,
        groups,
        flash_message: None,
        flash_is_error: false,
    }
}
