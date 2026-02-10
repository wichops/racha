use sqlx::SqlitePool;

pub async fn complete_today(pool: &SqlitePool, task_id: i64) -> sqlx::Result<()> {
    sqlx::query("INSERT OR IGNORE INTO completions (task_id, completed_date) VALUES (?, date('now'))")
        .bind(task_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn uncomplete_today(pool: &SqlitePool, task_id: i64) -> sqlx::Result<()> {
    sqlx::query("DELETE FROM completions WHERE task_id = ? AND completed_date = date('now')")
        .bind(task_id)
        .execute(pool)
        .await?;
    Ok(())
}
