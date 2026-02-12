use chrono::NaiveDate;
use sqlx::PgPool;

pub async fn complete_today(pool: &PgPool, task_id: i64, today: NaiveDate) -> sqlx::Result<()> {
    sqlx::query("INSERT INTO completions (task_id, completed_date) VALUES ($1, $2) ON CONFLICT DO NOTHING")
        .bind(task_id)
        .bind(today)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn uncomplete_today(pool: &PgPool, task_id: i64, today: NaiveDate) -> sqlx::Result<()> {
    sqlx::query("DELETE FROM completions WHERE task_id = $1 AND completed_date = $2")
        .bind(task_id)
        .bind(today)
        .execute(pool)
        .await?;
    Ok(())
}
