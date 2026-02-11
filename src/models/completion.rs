use sqlx::PgPool;

pub async fn complete_today(pool: &PgPool, task_id: i64) -> sqlx::Result<()> {
    sqlx::query("INSERT INTO completions (task_id, completed_date) VALUES ($1, CURRENT_DATE) ON CONFLICT DO NOTHING")
        .bind(task_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn uncomplete_today(pool: &PgPool, task_id: i64) -> sqlx::Result<()> {
    sqlx::query("DELETE FROM completions WHERE task_id = $1 AND completed_date = CURRENT_DATE")
        .bind(task_id)
        .execute(pool)
        .await?;
    Ok(())
}
