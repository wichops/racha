mod auth;
mod dashboard;
mod tasks;
mod groups;
mod profile;

use axum::Router;
use crate::AppState;

pub fn build_router() -> Router<AppState> {
    Router::new()
        .merge(auth::router())
        .merge(dashboard::router())
        .merge(tasks::router())
        .merge(groups::router())
        .merge(profile::router())
}
