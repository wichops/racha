use chrono::NaiveDateTime;
use sqlx::PgPool;

#[derive(sqlx::FromRow)]
#[allow(dead_code)]
pub struct Task {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub created_at: NaiveDateTime,
    pub archived: bool,
}

#[derive(sqlx::FromRow)]
#[allow(dead_code)]
pub struct TaskWithStreak {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub created_at: NaiveDateTime,
    pub archived: bool,
    pub current_streak: i64,
    pub completed_today: bool,
}

impl Task {
    pub async fn create(
        pool: &PgPool,
        user_id: i64,
        name: &str,
        description: Option<&str>,
    ) -> sqlx::Result<i64> {
        let id: i64 = sqlx::query_scalar(
            "INSERT INTO tasks (user_id, name, description) VALUES ($1, $2, $3) RETURNING id",
        )
        .bind(user_id)
        .bind(name)
        .bind(description)
        .fetch_one(pool)
        .await?;
        Ok(id)
    }

    pub async fn update(
        pool: &PgPool,
        id: i64,
        user_id: i64,
        name: &str,
        description: Option<&str>,
    ) -> sqlx::Result<()> {
        sqlx::query("UPDATE tasks SET name = $1, description = $2 WHERE id = $3 AND user_id = $4")
            .bind(name)
            .bind(description)
            .bind(id)
            .bind(user_id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn archive(pool: &PgPool, id: i64, user_id: i64) -> sqlx::Result<()> {
        sqlx::query("UPDATE tasks SET archived = TRUE WHERE id = $1 AND user_id = $2")
            .bind(id)
            .bind(user_id)
            .execute(pool)
            .await?;
        Ok(())
    }
}

impl TaskWithStreak {
    pub async fn for_user(pool: &PgPool, user_id: i64) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as(
            r#"
            WITH RECURSIVE streak_cte(task_id, streak_date, streak_count) AS (
                -- Base case: check if completed today
                SELECT c.task_id, c.completed_date, 1
                FROM completions c
                JOIN tasks t ON t.id = c.task_id
                WHERE t.user_id = $1
                  AND c.completed_date = CURRENT_DATE
                UNION ALL
                -- If not completed today, check yesterday as base
                SELECT c.task_id, c.completed_date, 1
                FROM completions c
                JOIN tasks t ON t.id = c.task_id
                WHERE t.user_id = $1
                  AND c.completed_date = CURRENT_DATE - INTERVAL '1 day'
                  AND NOT EXISTS (
                      SELECT 1 FROM completions c2
                      WHERE c2.task_id = c.task_id
                        AND c2.completed_date = CURRENT_DATE
                  )
                UNION ALL
                -- Recursive: walk backwards day by day
                SELECT s.task_id, c.completed_date, s.streak_count + 1
                FROM streak_cte s
                JOIN completions c ON c.task_id = s.task_id
                  AND c.completed_date = s.streak_date - INTERVAL '1 day'
            )
            SELECT
                t.id,
                t.user_id,
                t.name,
                t.description,
                t.created_at,
                t.archived,
                COALESCE(MAX(s.streak_count), 0)::BIGINT AS current_streak,
                EXISTS (
                    SELECT 1 FROM completions c
                    WHERE c.task_id = t.id AND c.completed_date = CURRENT_DATE
                ) AS completed_today
            FROM tasks t
            LEFT JOIN streak_cte s ON s.task_id = t.id
            WHERE t.user_id = $1 AND t.archived = FALSE
            GROUP BY t.id
            ORDER BY t.created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_id(pool: &PgPool, task_id: i64) -> sqlx::Result<Option<Self>> {
        sqlx::query_as(
            r#"
            WITH RECURSIVE streak_cte(task_id, streak_date, streak_count) AS (
                SELECT c.task_id, c.completed_date, 1
                FROM completions c
                WHERE c.task_id = $1
                  AND c.completed_date = CURRENT_DATE
                UNION ALL
                SELECT c.task_id, c.completed_date, 1
                FROM completions c
                WHERE c.task_id = $1
                  AND c.completed_date = CURRENT_DATE - INTERVAL '1 day'
                  AND NOT EXISTS (
                      SELECT 1 FROM completions c2
                      WHERE c2.task_id = c.task_id
                        AND c2.completed_date = CURRENT_DATE
                  )
                UNION ALL
                SELECT s.task_id, c.completed_date, s.streak_count + 1
                FROM streak_cte s
                JOIN completions c ON c.task_id = s.task_id
                  AND c.completed_date = s.streak_date - INTERVAL '1 day'
            )
            SELECT
                t.id,
                t.user_id,
                t.name,
                t.description,
                t.created_at,
                t.archived,
                COALESCE(MAX(s.streak_count), 0)::BIGINT AS current_streak,
                EXISTS (
                    SELECT 1 FROM completions c
                    WHERE c.task_id = t.id AND c.completed_date = CURRENT_DATE
                ) AS completed_today
            FROM tasks t
            LEFT JOIN streak_cte s ON s.task_id = t.id
            WHERE t.id = $1
            GROUP BY t.id
            "#,
        )
        .bind(task_id)
        .fetch_optional(pool)
        .await
    }
}
