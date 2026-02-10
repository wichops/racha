use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use axum::{
    extract::FromRequestParts,
    http::request::Parts,
    response::Redirect,
};
use tower_sessions::Session;

const USER_ID_KEY: &str = "user_id";

pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default().hash_password(password.as_bytes(), &salt)?;
    Ok(hash.to_string())
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, argon2::password_hash::Error> {
    let parsed_hash = PasswordHash::new(hash)?;
    Ok(Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok())
}

pub async fn login_session(session: &Session, user_id: i64) -> Result<(), tower_sessions::session::Error> {
    session.insert(USER_ID_KEY, user_id).await
}

pub async fn logout_session(session: &Session) -> Result<(), tower_sessions::session::Error> {
    session.flush().await
}

pub struct AuthUser {
    pub id: i64,
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = Redirect;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let session = parts
            .extensions
            .get::<Session>()
            .ok_or(Redirect::to("/login"))?;

        let user_id: Option<i64> = session
            .get(USER_ID_KEY)
            .await
            .map_err(|_| Redirect::to("/login"))?;

        match user_id {
            Some(id) => Ok(AuthUser { id }),
            None => Err(Redirect::to("/login")),
        }
    }
}
