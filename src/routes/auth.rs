use axum::{
    Router,
    extract::State,
    response::Redirect,
    routing::{get, post},
    Form,
};
use serde::Deserialize;
use tower_sessions::Session;

use crate::AppState;
use crate::auth::{hash_password, verify_password, login_session, logout_session};
use crate::models::user::User;
use crate::templates::auth::{LoginTemplate, RegisterTemplate};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/login", get(login_page).post(login_submit))
        .route("/register", get(register_page).post(register_submit))
        .route("/logout", post(logout))
}

async fn login_page() -> LoginTemplate {
    LoginTemplate {
        error: None,
        flash_message: None,
        flash_is_error: false,
    }
}

#[derive(Deserialize)]
struct LoginForm {
    username: String,
    password: String,
}

async fn login_submit(
    State(state): State<AppState>,
    session: Session,
    Form(form): Form<LoginForm>,
) -> Result<Redirect, LoginTemplate> {
    let user = User::find_by_username(&state.db, &form.username)
        .await
        .map_err(|_| LoginTemplate::with_error("Internal error"))?
        .ok_or_else(|| LoginTemplate::with_error("Invalid username or password"))?;

    let valid = verify_password(&form.password, &user.password_hash)
        .map_err(|_| LoginTemplate::with_error("Internal error"))?;

    if !valid {
        return Err(LoginTemplate::with_error("Invalid username or password"));
    }

    login_session(&session, user.id)
        .await
        .map_err(|_| LoginTemplate::with_error("Session error"))?;

    Ok(Redirect::to("/"))
}

async fn register_page() -> RegisterTemplate {
    RegisterTemplate {
        error: None,
        flash_message: None,
        flash_is_error: false,
    }
}

#[derive(Deserialize)]
struct RegisterForm {
    username: String,
    email: String,
    password: String,
}

async fn register_submit(
    State(state): State<AppState>,
    session: Session,
    Form(form): Form<RegisterForm>,
) -> Result<Redirect, RegisterTemplate> {
    if form.username.len() < 3 {
        return Err(RegisterTemplate::with_error("Username must be at least 3 characters"));
    }
    if form.password.len() < 8 {
        return Err(RegisterTemplate::with_error("Password must be at least 8 characters"));
    }

    let password_hash = hash_password(&form.password)
        .map_err(|_| RegisterTemplate::with_error("Internal error"))?;

    let user_id = User::create(&state.db, &form.username, &form.email, &password_hash)
        .await
        .map_err(|_| RegisterTemplate::with_error("Username or email already taken"))?;

    login_session(&session, user_id)
        .await
        .map_err(|_| RegisterTemplate::with_error("Session error"))?;

    Ok(Redirect::to("/"))
}

async fn logout(session: Session) -> Redirect {
    let _ = logout_session(&session).await;
    Redirect::to("/login")
}
