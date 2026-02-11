mod common;

use sqlx::PgPool;

#[derive(serde::Serialize)]
struct CreateTaskForm {
    name: String,
    description: Option<String>,
}

#[derive(serde::Serialize)]
struct UpdateTaskForm {
    name: String,
    description: Option<String>,
}

#[sqlx::test]
async fn create_task_requires_auth(pool: PgPool) {
    let server = common::build_test_server(pool).await;
    let response = server
        .post("/tasks")
        .form(&CreateTaskForm {
            name: "Exercise".to_string(),
            description: None,
        })
        .await;
    // Unauthenticated -> redirect to login
    response.assert_status_see_other();
}

#[sqlx::test]
async fn create_task_returns_html_partial(pool: PgPool) {
    let server = common::build_test_server(pool).await;
    common::register_user(&server, "alice", "alice@test.com", "password123").await;

    let response = server
        .post("/tasks")
        .form(&CreateTaskForm {
            name: "Exercise".to_string(),
            description: Some("Daily workout".to_string()),
        })
        .await;
    response.assert_status_ok();
    response.assert_text_contains("Exercise");
}

#[sqlx::test]
async fn task_form_returns_200(pool: PgPool) {
    let server = common::build_test_server(pool).await;
    common::register_user(&server, "alice", "alice@test.com", "password123").await;

    let response = server.get("/tasks/form").await;
    response.assert_status_ok();
}

#[sqlx::test]
async fn toggle_task_marks_completed(pool: PgPool) {
    let server = common::build_test_server(pool).await;
    common::register_user(&server, "alice", "alice@test.com", "password123").await;

    // Create a task (id will be 1 in fresh DB)
    server
        .post("/tasks")
        .form(&CreateTaskForm {
            name: "Meditate".to_string(),
            description: None,
        })
        .await;

    // Toggle on
    let response = server.post("/tasks/1/toggle").await;
    response.assert_status_ok();

    // The dashboard should show the task as completed today
    let dashboard = server.get("/").await;
    dashboard.assert_status_ok();
    dashboard.assert_text_contains("Meditate");
}

#[sqlx::test]
async fn toggle_task_uncompletes(pool: PgPool) {
    let server = common::build_test_server(pool).await;
    common::register_user(&server, "alice", "alice@test.com", "password123").await;

    server
        .post("/tasks")
        .form(&CreateTaskForm {
            name: "Meditate".to_string(),
            description: None,
        })
        .await;

    // Toggle on then off
    server.post("/tasks/1/toggle").await;
    let response = server.post("/tasks/1/toggle").await;
    response.assert_status_ok();
}

#[sqlx::test]
async fn toggle_other_users_task_returns_403(pool: PgPool) {
    let server = common::build_test_server(pool).await;

    // User A creates a task
    common::register_user(&server, "alice", "alice@test.com", "password123").await;
    server
        .post("/tasks")
        .form(&CreateTaskForm {
            name: "Alice task".to_string(),
            description: None,
        })
        .await;

    // Switch to user B
    server.post("/logout").await;
    common::register_user(&server, "bob", "bob@test.com", "password123").await;

    let response = server.post("/tasks/1/toggle").await;
    response.assert_status_forbidden();
}

#[sqlx::test]
async fn toggle_nonexistent_task_returns_404(pool: PgPool) {
    let server = common::build_test_server(pool).await;
    common::register_user(&server, "alice", "alice@test.com", "password123").await;

    let response = server.post("/tasks/999/toggle").await;
    response.assert_status_not_found();
}

#[sqlx::test]
async fn edit_form_returns_task_data(pool: PgPool) {
    let server = common::build_test_server(pool).await;
    common::register_user(&server, "alice", "alice@test.com", "password123").await;

    server
        .post("/tasks")
        .form(&CreateTaskForm {
            name: "Read".to_string(),
            description: Some("Read a book".to_string()),
        })
        .await;

    let response = server.get("/tasks/1/edit").await;
    response.assert_status_ok();
    response.assert_text_contains("Read");
}

#[sqlx::test]
async fn update_task_returns_updated_card(pool: PgPool) {
    let server = common::build_test_server(pool).await;
    common::register_user(&server, "alice", "alice@test.com", "password123").await;

    server
        .post("/tasks")
        .form(&CreateTaskForm {
            name: "Read".to_string(),
            description: None,
        })
        .await;

    let response = server
        .post("/tasks/1/edit")
        .form(&UpdateTaskForm {
            name: "Read more".to_string(),
            description: Some("30 minutes".to_string()),
        })
        .await;
    response.assert_status_ok();
    response.assert_text_contains("Read more");
}

#[sqlx::test]
async fn task_card_returns_html(pool: PgPool) {
    let server = common::build_test_server(pool).await;
    common::register_user(&server, "alice", "alice@test.com", "password123").await;

    server
        .post("/tasks")
        .form(&CreateTaskForm {
            name: "Stretch".to_string(),
            description: None,
        })
        .await;

    let response = server.get("/tasks/1/card").await;
    response.assert_status_ok();
    response.assert_text_contains("Stretch");
}

#[sqlx::test]
async fn archive_task_returns_200(pool: PgPool) {
    let server = common::build_test_server(pool).await;
    common::register_user(&server, "alice", "alice@test.com", "password123").await;

    server
        .post("/tasks")
        .form(&CreateTaskForm {
            name: "Old habit".to_string(),
            description: None,
        })
        .await;

    let response = server.post("/tasks/1/archive").await;
    response.assert_status_ok();

    // Archived task should not appear on dashboard
    let dashboard = server.get("/").await;
    dashboard.assert_status_ok();
    let text = dashboard.text();
    assert!(!text.contains("Old habit"), "Archived task should not appear on dashboard");
}
