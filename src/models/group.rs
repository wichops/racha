use chrono::NaiveDateTime;
use rand::Rng;
use sqlx::PgPool;

#[derive(sqlx::FromRow)]
#[allow(dead_code)]
pub struct Group {
    pub id: i64,
    pub name: String,
    pub invite_code: String,
    pub created_by: i64,
    pub created_at: NaiveDateTime,
}

#[derive(sqlx::FromRow)]
#[allow(dead_code)]
pub struct GroupWithMembership {
    pub id: i64,
    pub name: String,
    pub invite_code: String,
    pub member_count: i64,
}

#[derive(sqlx::FromRow)]
#[allow(dead_code)]
pub struct MemberWithStreaks {
    pub user_id: i64,
    pub username: String,
    pub task_id: i64,
    pub task_name: String,
    pub current_streak: i64,
    pub completed_today: bool,
}

fn generate_invite_code() -> String {
    let mut rng = rand::rng();
    let chars: Vec<char> = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789".chars().collect();
    (0..8).map(|_| chars[rng.random_range(0..chars.len())]).collect()
}

impl Group {
    pub async fn create(pool: &PgPool, name: &str, created_by: i64) -> sqlx::Result<i64> {
        let invite_code = generate_invite_code();
        let group_id: i64 = sqlx::query_scalar(
            "INSERT INTO groups (name, invite_code, created_by) VALUES ($1, $2, $3) RETURNING id",
        )
        .bind(name)
        .bind(&invite_code)
        .bind(created_by)
        .fetch_one(pool)
        .await?;

        // Creator auto-joins the group
        sqlx::query("INSERT INTO group_members (group_id, user_id) VALUES ($1, $2)")
            .bind(group_id)
            .bind(created_by)
            .execute(pool)
            .await?;

        Ok(group_id)
    }

    pub async fn find_by_id(pool: &PgPool, id: i64) -> sqlx::Result<Option<Self>> {
        sqlx::query_as("SELECT * FROM groups WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    pub async fn find_by_invite_code(pool: &PgPool, code: &str) -> sqlx::Result<Option<Self>> {
        sqlx::query_as("SELECT * FROM groups WHERE invite_code = $1")
            .bind(code)
            .fetch_optional(pool)
            .await
    }

    pub async fn join(pool: &PgPool, group_id: i64, user_id: i64) -> sqlx::Result<()> {
        sqlx::query("INSERT INTO group_members (group_id, user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
            .bind(group_id)
            .bind(user_id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn user_groups(pool: &PgPool, user_id: i64) -> sqlx::Result<Vec<GroupWithMembership>> {
        sqlx::query_as(
            r#"
            SELECT g.id, g.name, g.invite_code,
                   (SELECT COUNT(*) FROM group_members gm2 WHERE gm2.group_id = g.id) AS member_count
            FROM groups g
            JOIN group_members gm ON gm.group_id = g.id
            WHERE gm.user_id = $1
            ORDER BY g.name
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
    }

    pub async fn member_streaks(pool: &PgPool, group_id: i64) -> sqlx::Result<Vec<MemberWithStreaks>> {
        sqlx::query_as(
            r#"
            WITH RECURSIVE streak_cte(task_id, streak_date, streak_count) AS (
                -- Base case: completed today
                SELECT c.task_id, c.completed_date, 1
                FROM completions c
                JOIN tasks t ON t.id = c.task_id
                JOIN group_members gm ON gm.user_id = t.user_id AND gm.group_id = $1
                WHERE c.completed_date = (NOW() AT TIME ZONE 'America/Mexico_City')::DATE
                UNION ALL
                -- Base case: completed yesterday but not today
                SELECT c.task_id, c.completed_date, 1
                FROM completions c
                JOIN tasks t ON t.id = c.task_id
                JOIN group_members gm ON gm.user_id = t.user_id AND gm.group_id = $1
                WHERE c.completed_date = (NOW() AT TIME ZONE 'America/Mexico_City')::DATE - INTERVAL '1 day'
                  AND NOT EXISTS (
                      SELECT 1 FROM completions c2
                      WHERE c2.task_id = c.task_id
                        AND c2.completed_date = (NOW() AT TIME ZONE 'America/Mexico_City')::DATE
                  )
                UNION ALL
                -- Recursive: walk backwards day by day
                SELECT s.task_id, c.completed_date, s.streak_count + 1
                FROM streak_cte s
                JOIN completions c ON c.task_id = s.task_id
                  AND c.completed_date = s.streak_date - INTERVAL '1 day'
            )
            SELECT
                u.id AS user_id,
                u.username,
                t.id AS task_id,
                t.name AS task_name,
                COALESCE(MAX(s.streak_count), 0)::BIGINT AS current_streak,
                EXISTS (
                    SELECT 1 FROM completions c
                    WHERE c.task_id = t.id AND c.completed_date = (NOW() AT TIME ZONE 'America/Mexico_City')::DATE
                ) AS completed_today
            FROM group_members gm
            JOIN users u ON u.id = gm.user_id
            JOIN tasks t ON t.user_id = u.id AND t.archived = FALSE
            LEFT JOIN streak_cte s ON s.task_id = t.id
            WHERE gm.group_id = $1
            GROUP BY t.id, u.id, u.username, t.name
            ORDER BY u.username, t.name
            "#,
        )
        .bind(group_id)
        .fetch_all(pool)
        .await
    }
}
