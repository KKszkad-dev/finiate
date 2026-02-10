use async_trait::async_trait;
use domain::*;
use jiff::Timestamp;
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

#[derive(FromRow)]
struct DbLog {
    id: Uuid,
    create_at: i64,
    content: String,
    log_type: String,
    agenda_id: Uuid,
}

pub struct SqliteLogRepo {
    pub pool: SqlitePool,
}

#[async_trait]
impl LogRepo for SqliteLogRepo {
    type Error = sqlx::Error;

    async fn create_log(&self, new_log: &LogCreate) -> Result<Uuid, Self::Error> {
        let uuid = Uuid::now_v7();
        let timestamp = Timestamp::now().as_millisecond();
        sqlx::query(
            "INSERT INTO log (id, create_at, content, log_type, agenda_id) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(uuid.to_string())
        .bind(timestamp)
        .bind(&new_log.content)
        .bind(&new_log.log_type.to_string())
        .bind(new_log.agenda_id.to_string())
        .execute(&self.pool)
        .await?;
        Ok(uuid)
    }
    //TODO trait implementation
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::Row;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn setup_pool() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("create in-memory sqlite pool");

        sqlx::query("PRAGMA foreign_keys = ON;")
            .execute(&pool)
            .await
            .expect("enable foreign keys");

        let crate_dir = env!("CARGO_MANIFEST_DIR");
        let migrations = std::path::Path::new(crate_dir).join("migrations");
        sqlx::migrate::Migrator::new(migrations)
            .await
            .expect("load migrations")
            .run(&pool)
            .await
            .expect("run migrations");

        pool
    }

    async fn insert_agenda(pool: &SqlitePool, agenda_id: Uuid) {
        let now = Timestamp::now().as_millisecond();
        sqlx::query(
            "INSERT INTO agenda (id, title, agenda_status, initiate_at, terminate_at) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(agenda_id.to_string())
        .bind("Test agenda")
        .bind("active")
        .bind(now)
        .bind(now + 1000)
        .execute(pool)
        .await
        .expect("insert agenda");
    }

    #[tokio::test]
    async fn create_log_inserts_row() {
        let pool = setup_pool().await;
        let repo = SqliteLogRepo { pool: pool.clone() };

        let agenda_id = Uuid::now_v7();
        insert_agenda(&pool, agenda_id).await;

        let start_ms = Timestamp::now().as_millisecond();
        let log = LogCreate {
            agenda_id,
            content: "first log".to_string(),
            log_type: LogType::Operation,
        };

        let created_id = repo.create_log(&log).await.expect("create log");
        let end_ms = Timestamp::now().as_millisecond();

        let row =
            sqlx::query("SELECT id, create_at, content, log_type, agenda_id FROM log WHERE id = ?")
                .bind(created_id.to_string())
                .fetch_one(&pool)
                .await
                .expect("fetch log row");

        let id: String = row.get("id");
        let create_at: i64 = row.get("create_at");
        let content: String = row.get("content");
        let log_type: String = row.get("log_type");
        let agenda_id_str: String = row.get("agenda_id");

        assert_eq!(id, created_id.to_string());
        assert!(create_at >= start_ms && create_at <= end_ms);
        assert_eq!(content, "first log");
        assert_eq!(log_type, "operation");
        assert_eq!(agenda_id_str, agenda_id.to_string());
    }
}
