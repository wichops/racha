mod common;

use sqlx::PgPool;

#[sqlx::test]
async fn unauthenticated_dashboard_redirects_to_login(pool: PgPool) {
    let server = common::build_test_server(pool).await;
    let response = server.get("/").await;
    response.assert_status_see_other();
}

#[sqlx::test]
async fn authenticated_dashboard_returns_200_with_username(pool: PgPool) {
    let server = common::build_test_server(pool).await;
    common::register_user(&server, "alice", "alice@test.com", "password123").await;

    let response = server.get("/").await;
    response.assert_status_ok();
    response.assert_text_contains("alice");
}

#[sqlx::test]
async fn dashboard_with_no_tasks_or_groups_renders(pool: PgPool) {
    let server = common::build_test_server(pool).await;
    common::register_user(&server, "bob", "bob@test.com", "password123").await;

    let response = server.get("/").await;
    response.assert_status_ok();
    // Should render without error even with empty tasks/groups
    response.assert_text_contains("bob");
}
