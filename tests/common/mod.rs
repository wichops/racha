use axum_test::TestServer;
use sqlx::PgPool;
use tower_sessions::SessionManagerLayer;
use tower_sessions_sqlx_store::PostgresStore;

use racha::{AppState, routes};

pub async fn build_test_server(pool: PgPool) -> TestServer {
    let session_store = PostgresStore::new(pool.clone());
    session_store.migrate().await.expect("Failed to migrate session store");

    let session_layer = SessionManagerLayer::new(session_store);

    let state = AppState { db: pool };

    let app = routes::build_router()
        .layer(session_layer)
        .with_state(state);

    TestServer::builder()
        .save_cookies()
        .build(app)
        .unwrap()
}

pub async fn register_user(server: &TestServer, username: &str, email: &str, password: &str) {
    let response = server
        .post("/register")
        .form(&RegisterForm {
            username: username.to_string(),
            email: email.to_string(),
            password: password.to_string(),
        })
        .await;
    response.assert_status_see_other();
}

#[derive(serde::Serialize)]
pub struct RegisterForm {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(serde::Serialize)]
#[allow(dead_code)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}
