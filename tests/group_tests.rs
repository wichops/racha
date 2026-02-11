mod common;

use sqlx::PgPool;

#[derive(serde::Serialize)]
struct CreateGroupForm {
    name: String,
}

#[derive(serde::Serialize)]
struct JoinGroupForm {
    invite_code: String,
}

#[derive(serde::Serialize)]
struct CreateTaskForm {
    name: String,
    description: Option<String>,
}

#[sqlx::test]
async fn create_group_redirects_to_dashboard(pool: PgPool) {
    let server = common::build_test_server(pool).await;
    common::register_user(&server, "alice", "alice@test.com", "password123").await;

    let response = server
        .post("/groups")
        .form(&CreateGroupForm {
            name: "Fitness Crew".to_string(),
        })
        .await;
    response.assert_status_see_other();

    // Group should appear on dashboard
    let dashboard = server.get("/").await;
    dashboard.assert_status_ok();
    dashboard.assert_text_contains("Fitness Crew");
}

#[sqlx::test]
async fn create_group_form_returns_200(pool: PgPool) {
    let server = common::build_test_server(pool).await;
    common::register_user(&server, "alice", "alice@test.com", "password123").await;

    let response = server.get("/groups/create-form").await;
    response.assert_status_ok();
}

#[sqlx::test]
async fn join_group_form_returns_200(pool: PgPool) {
    let server = common::build_test_server(pool).await;
    common::register_user(&server, "alice", "alice@test.com", "password123").await;

    let response = server.get("/groups/join-form").await;
    response.assert_status_ok();
}

#[sqlx::test]
async fn join_group_by_invite_code(pool: PgPool) {
    let server = common::build_test_server(pool.clone()).await;

    // User A creates a group
    common::register_user(&server, "alice", "alice@test.com", "password123").await;
    server
        .post("/groups")
        .form(&CreateGroupForm {
            name: "Study Group".to_string(),
        })
        .await;

    // Get the invite code from the DB directly
    let invite_code: String =
        sqlx::query_scalar("SELECT invite_code FROM groups WHERE id = 1")
            .fetch_one(&pool)
            .await
            .unwrap();

    // Switch to user B
    server.post("/logout").await;
    common::register_user(&server, "bob", "bob@test.com", "password123").await;

    let response = server
        .post("/groups/join")
        .form(&JoinGroupForm {
            invite_code,
        })
        .await;
    response.assert_status_see_other();

    // Bob's dashboard should show the group
    let dashboard = server.get("/").await;
    dashboard.assert_status_ok();
    dashboard.assert_text_contains("Study Group");
}

#[sqlx::test]
async fn group_feed_shows_member_names(pool: PgPool) {
    let server = common::build_test_server(pool.clone()).await;

    // Create user with task, then create group
    common::register_user(&server, "alice", "alice@test.com", "password123").await;
    server
        .post("/tasks")
        .form(&CreateTaskForm {
            name: "Run".to_string(),
            description: None,
        })
        .await;
    server
        .post("/groups")
        .form(&CreateGroupForm {
            name: "Runners".to_string(),
        })
        .await;

    let response = server.get("/groups/1").await;
    response.assert_status_ok();
    response.assert_text_contains("alice");
    response.assert_text_contains("Runners");
}

#[sqlx::test]
async fn nonexistent_group_returns_404(pool: PgPool) {
    let server = common::build_test_server(pool).await;
    common::register_user(&server, "alice", "alice@test.com", "password123").await;

    let response = server.get("/groups/999").await;
    response.assert_status_not_found();
}

#[sqlx::test]
async fn invalid_invite_code_redirects(pool: PgPool) {
    let server = common::build_test_server(pool).await;
    common::register_user(&server, "alice", "alice@test.com", "password123").await;

    let response = server
        .post("/groups/join")
        .form(&JoinGroupForm {
            invite_code: "BADCODE1".to_string(),
        })
        .await;
    // Current behavior: always redirects to /
    response.assert_status_see_other();
}

#[sqlx::test]
async fn group_routes_require_auth(pool: PgPool) {
    let server = common::build_test_server(pool).await;

    let response = server.get("/groups/create-form").await;
    response.assert_status_see_other();

    let response = server.get("/groups/join-form").await;
    response.assert_status_see_other();

    let response = server
        .post("/groups")
        .form(&CreateGroupForm {
            name: "Test".to_string(),
        })
        .await;
    response.assert_status_see_other();
}
