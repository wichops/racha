use sqlx::SqlitePool;

#[derive(sqlx::FromRow)]
#[allow(dead_code)]
pub struct Task {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
    pub archived: bool,
}

#[derive(sqlx::FromRow)]
#[allow(dead_code)]
pub struct TaskWithStreak {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
    pub archived: bool,
    pub current_streak: i64,
    pub completed_today: bool,
}

impl Task {
    pub async fn create(
        pool: &SqlitePool,
        user_id: i64,
        name: &str,
        description: Option<&str>,
    ) -> sqlx::Result<i64> {
        let result = sqlx::query(
            "INSERT INTO tasks (user_id, name, description) VALUES (?, ?, ?)",
        )
        .bind(user_id)
        .bind(name)
        .bind(description)
        .execute(pool)
        .await?;
        Ok(result.last_insert_rowid())
    }

    pub async fn update(
        pool: &SqlitePool,
        id: i64,
        user_id: i64,
        name: &str,
        description: Option<&str>,
    ) -> sqlx::Result<()> {
        sqlx::query("UPDATE tasks SET name = ?, description = ? WHERE id = ? AND user_id = ?")
            .bind(name)
            .bind(description)
            .bind(id)
            .bind(user_id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn archive(pool: &SqlitePool, id: i64, user_id: i64) -> sqlx::Result<()> {
        sqlx::query("UPDATE tasks SET archived = 1 WHERE id = ? AND user_id = ?")
            .bind(id)
            .bind(user_id)
            .execute(pool)
            .await?;
        Ok(())
    }
}

impl TaskWithStreak {
    pub async fn for_user(pool: &SqlitePool, user_id: i64) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as(
            r#"
            WITH RECURSIVE streak_cte(task_id, streak_date, streak_count) AS (
                -- Base case: check if completed today
                SELECT c.task_id, c.completed_date, 1
                FROM completions c
                JOIN tasks t ON t.id = c.task_id
                WHERE t.user_id = ?1
                  AND c.completed_date = date('now')
                UNION ALL
                -- If not completed today, check yesterday as base
                SELECT c.task_id, c.completed_date, 1
                FROM completions c
                JOIN tasks t ON t.id = c.task_id
                WHERE t.user_id = ?1
                  AND c.completed_date = date('now', '-1 day')
                  AND NOT EXISTS (
                      SELECT 1 FROM completions c2
                      WHERE c2.task_id = c.task_id
                        AND c2.completed_date = date('now')
                  )
                UNION ALL
                -- Recursive: walk backwards day by day
                SELECT s.task_id, c.completed_date, s.streak_count + 1
                FROM streak_cte s
                JOIN completions c ON c.task_id = s.task_id
                  AND c.completed_date = date(s.streak_date, '-1 day')
            )
            SELECT
                t.id,
                t.user_id,
                t.name,
                t.description,
                t.created_at,
                t.archived,
                COALESCE(MAX(s.streak_count), 0) AS current_streak,
                EXISTS (
                    SELECT 1 FROM completions c
                    WHERE c.task_id = t.id AND c.completed_date = date('now')
                ) AS completed_today
            FROM tasks t
            LEFT JOIN streak_cte s ON s.task_id = t.id
            WHERE t.user_id = ?1 AND t.archived = 0
            GROUP BY t.id
            ORDER BY t.created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_id(pool: &SqlitePool, task_id: i64) -> sqlx::Result<Option<Self>> {
        sqlx::query_as(
            r#"
            WITH RECURSIVE streak_cte(task_id, streak_date, streak_count) AS (
                SELECT c.task_id, c.completed_date, 1
                FROM completions c
                WHERE c.task_id = ?1
                  AND c.completed_date = date('now')
                UNION ALL
                SELECT c.task_id, c.completed_date, 1
                FROM completions c
                WHERE c.task_id = ?1
                  AND c.completed_date = date('now', '-1 day')
                  AND NOT EXISTS (
                      SELECT 1 FROM completions c2
                      WHERE c2.task_id = c.task_id
                        AND c2.completed_date = date('now')
                  )
                UNION ALL
                SELECT s.task_id, c.completed_date, s.streak_count + 1
                FROM streak_cte s
                JOIN completions c ON c.task_id = s.task_id
                  AND c.completed_date = date(s.streak_date, '-1 day')
            )
            SELECT
                t.id,
                t.user_id,
                t.name,
                t.description,
                t.created_at,
                t.archived,
                COALESCE(MAX(s.streak_count), 0) AS current_streak,
                EXISTS (
                    SELECT 1 FROM completions c
                    WHERE c.task_id = t.id AND c.completed_date = date('now')
                ) AS completed_today
            FROM tasks t
            LEFT JOIN streak_cte s ON s.task_id = t.id
            WHERE t.id = ?1
            GROUP BY t.id
            "#,
        )
        .bind(task_id)
        .fetch_optional(pool)
        .await
    }
}
