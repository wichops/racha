use axum::{
    Router,
    extract::State,
    routing::get,
};

use crate::AppState;
use crate::auth::AuthUser;
use crate::models::user::User;
use crate::templates::profile::ProfileTemplate;

pub fn router() -> Router<AppState> {
    Router::new().route("/profile", get(profile))
}

async fn profile(
    State(state): State<AppState>,
    user: AuthUser,
) -> ProfileTemplate {
    let db_user = User::find_by_id(&state.db, user.id).await.ok().flatten();
    let (username, email) = db_user
        .map(|u| (u.username, u.email))
        .unwrap_or_default();

    ProfileTemplate {
        username,
        email,
        flash_message: None,
        flash_is_error: false,
    }
}
