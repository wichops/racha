use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use axum::{
    extract::FromRequestParts,
    http::request::Parts,
    response::Redirect,
};
use chrono::NaiveDate;
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

pub struct LocalDate(pub NaiveDate);

impl<S> FromRequestParts<S> for LocalDate
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let date = parse_from_header(parts)
            .or_else(|| parse_from_cookie(parts))
            .unwrap_or_else(server_today);
        Ok(LocalDate(date))
    }
}

fn parse_date(val: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(val.trim(), "%Y-%m-%d").ok()
}

fn parse_from_header(parts: &Parts) -> Option<NaiveDate> {
    parts.headers.get("X-Local-Date")?.to_str().ok().and_then(parse_date)
}

fn parse_from_cookie(parts: &Parts) -> Option<NaiveDate> {
    let cookies = parts.headers.get("cookie")?.to_str().ok()?;
    for pair in cookies.split(';') {
        let pair = pair.trim();
        if let Some(val) = pair.strip_prefix("local_date=") {
            return parse_date(val);
        }
    }
    None
}

fn server_today() -> NaiveDate {
    use chrono::{FixedOffset, Utc};
    let offset = FixedOffset::west_opt(6 * 3600).unwrap(); // CST (UTC-6)
    Utc::now().with_timezone(&offset).date_naive()
}
