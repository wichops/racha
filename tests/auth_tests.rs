mod common;

use sqlx::PgPool;

#[sqlx::test]
async fn login_page_returns_200(pool: PgPool) {
    let server = common::build_test_server(pool).await;
    let response = server.get("/login").await;
    response.assert_status_ok();
    response.assert_text_contains("Log in");
}

#[sqlx::test]
async fn register_page_returns_200(pool: PgPool) {
    let server = common::build_test_server(pool).await;
    let response = server.get("/register").await;
    response.assert_status_ok();
    response.assert_text_contains("Sign Up");
}

#[sqlx::test]
async fn register_success_redirects_to_dashboard(pool: PgPool) {
    let server = common::build_test_server(pool).await;
    common::register_user(&server, "alice", "alice@test.com", "password123").await;

    let response = server.get("/").await;
    response.assert_status_ok();
    response.assert_text_contains("alice");
}

#[sqlx::test]
async fn register_short_username_shows_error(pool: PgPool) {
    let server = common::build_test_server(pool).await;
    let response = server
        .post("/register")
        .form(&common::RegisterForm {
            username: "ab".to_string(),
            email: "ab@test.com".to_string(),
            password: "password123".to_string(),
        })
        .await;
    response.assert_status_ok();
    response.assert_text_contains("at least 3 characters");
}

#[sqlx::test]
async fn register_short_password_shows_error(pool: PgPool) {
    let server = common::build_test_server(pool).await;
    let response = server
        .post("/register")
        .form(&common::RegisterForm {
            username: "alice".to_string(),
            email: "alice@test.com".to_string(),
            password: "short".to_string(),
        })
        .await;
    response.assert_status_ok();
    response.assert_text_contains("at least 8 characters");
}

#[sqlx::test]
async fn register_duplicate_username_shows_error(pool: PgPool) {
    let server = common::build_test_server(pool).await;
    common::register_user(&server, "alice", "alice@test.com", "password123").await;

    server.post("/logout").await;
    let response = server
        .post("/register")
        .form(&common::RegisterForm {
            username: "alice".to_string(),
            email: "alice2@test.com".to_string(),
            password: "password123".to_string(),
        })
        .await;
    response.assert_status_ok();
    response.assert_text_contains("already taken");
}

#[sqlx::test]
async fn login_success_redirects_to_dashboard(pool: PgPool) {
    let server = common::build_test_server(pool).await;
    common::register_user(&server, "alice", "alice@test.com", "password123").await;
    server.post("/logout").await;

    let response = server
        .post("/login")
        .form(&common::LoginForm {
            username: "alice".to_string(),
            password: "password123".to_string(),
        })
        .await;
    response.assert_status_see_other();

    let dashboard = server.get("/").await;
    dashboard.assert_status_ok();
    dashboard.assert_text_contains("alice");
}

#[sqlx::test]
async fn login_wrong_password_shows_error(pool: PgPool) {
    let server = common::build_test_server(pool).await;
    common::register_user(&server, "alice", "alice@test.com", "password123").await;
    server.post("/logout").await;

    let response = server
        .post("/login")
        .form(&common::LoginForm {
            username: "alice".to_string(),
            password: "wrongpassword".to_string(),
        })
        .await;
    response.assert_status_ok();
    response.assert_text_contains("Invalid username or password");
}

#[sqlx::test]
async fn login_nonexistent_user_shows_error(pool: PgPool) {
    let server = common::build_test_server(pool).await;
    let response = server
        .post("/login")
        .form(&common::LoginForm {
            username: "nobody".to_string(),
            password: "password123".to_string(),
        })
        .await;
    response.assert_status_ok();
    response.assert_text_contains("Invalid username or password");
}

#[sqlx::test]
async fn logout_clears_session(pool: PgPool) {
    let server = common::build_test_server(pool).await;
    common::register_user(&server, "alice", "alice@test.com", "password123").await;

    let response = server.post("/logout").await;
    response.assert_status_see_other();

    let dashboard = server.get("/").await;
    dashboard.assert_status_see_other();
}
