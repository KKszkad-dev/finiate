use async_trait::async_trait;
use domain::*;
use jiff::Timestamp;
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

#[derive(FromRow)]
struct DbLog {
    id: String,
    create_at: i64,
    content: String,
    log_type: String,
    agenda_id: String,
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

    async fn delete_log(&self, id: Uuid) -> Result<(), Self::Error> {
        sqlx::query("DELETE FROM log WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn get_logs_by_agenda_id(&self, agenda_id: Uuid) -> Result<Vec<Log>, Self::Error> {
        let rows = sqlx::query_as::<_, DbLog>(
            "SELECT id, create_at, content, log_type, agenda_id FROM log WHERE agenda_id = ?",
        )
        .bind(agenda_id.to_string())
        .fetch_all(&self.pool)
        .await?;

        let logs = rows
            .into_iter()
            .map(|row| Log {
                id: Uuid::parse_str(&row.id).expect("valid UUID in DB"),
                agenda_id: Uuid::parse_str(&row.agenda_id).expect("valid UUID in DB"),
                content: row.content,
                create_at: Timestamp::from_millisecond(row.create_at)
                    .expect("invalid timestamp in database"),
                log_type: match row.log_type.as_str() {
                    "activate" => LogType::Activate,
                    "put_off" => LogType::PutOff,
                    "terminate" => LogType::Terminate,
                    "common_log" => LogType::CommonLog,
                    _ => panic!("invalid log type in DB"),
                },
            })
            .collect();

        Ok(logs)
    }
    async fn get_logs_by_time_range(
        &self,
        start: Timestamp,
        end: Timestamp,
    ) -> Result<Vec<Log>, Self::Error> {
        let rows = sqlx::query_as::<_, DbLog>(
            "SELECT id, create_at, content, log_type, agenda_id FROM log WHERE create_at >= ? AND create_at <= ?",
        )
        .bind(start.as_millisecond())
        .bind(end.as_millisecond())
        .fetch_all(&self.pool)
        .await?;

        let logs = rows
            .into_iter()
            .map(|row| Log {
                id: Uuid::parse_str(&row.id).expect("valid UUID in DB"),
                agenda_id: Uuid::parse_str(&row.agenda_id).expect("valid UUID in DB"),
                content: row.content,
                create_at: Timestamp::from_millisecond(row.create_at)
                    .expect("invalid timestamp in database"),
                log_type: match row.log_type.as_str() {
                    "activate" => LogType::Activate,
                    "put_off" => LogType::PutOff,
                    "terminate" => LogType::Terminate,
                    "common_log" => LogType::CommonLog,
                    _ => panic!("invalid log type in DB"),
                },
            })
            .collect();

        Ok(logs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::ToSpan;
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
            log_type: LogType::Activate,
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
        assert_eq!(log_type, "activate");
        assert_eq!(agenda_id_str, agenda_id.to_string());
    }

    #[tokio::test]
    async fn delete_log_removes_row() {
        let pool = setup_pool().await;
        let repo = SqliteLogRepo { pool: pool.clone() };

        let agenda_id = Uuid::now_v7();
        insert_agenda(&pool, agenda_id).await;

        let log = LogCreate {
            agenda_id,
            content: "to delete".to_string(),
            log_type: LogType::CommonLog,
        };
        let log_id = repo.create_log(&log).await.expect("create log");

        let count_before: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM log WHERE id = ?")
            .bind(log_id.to_string())
            .fetch_one(&pool)
            .await
            .expect("count before delete");
        assert_eq!(count_before, 1);

        repo.delete_log(log_id).await.expect("delete log");

        let count_after: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM log WHERE id = ?")
            .bind(log_id.to_string())
            .fetch_one(&pool)
            .await
            .expect("count after delete");
        assert_eq!(count_after, 0);
    }

    #[tokio::test]
    async fn delete_log_idempotent() {
        let pool = setup_pool().await;
        let repo = SqliteLogRepo { pool: pool.clone() };

        let non_existent_id = Uuid::now_v7();
        let result = repo.delete_log(non_existent_id).await;
        assert!(result.is_ok(), "delete non-existent log should not error");
    }

    #[tokio::test]
    async fn get_logs_by_agenda_id_filters() {
        let pool = setup_pool().await;
        let repo = SqliteLogRepo { pool: pool.clone() };

        let agenda_a = Uuid::now_v7();
        let agenda_b = Uuid::now_v7();
        insert_agenda(&pool, agenda_a).await;
        insert_agenda(&pool, agenda_b).await;

        let log_a1 = LogCreate {
            agenda_id: agenda_a,
            content: "a1".to_string(),
            log_type: LogType::Activate,
        };
        let id_a1 = repo.create_log(&log_a1).await.expect("create log a1");

        let log_a2 = LogCreate {
            agenda_id: agenda_a,
            content: "a2".to_string(),
            log_type: LogType::PutOff,
        };
        let id_a2 = repo.create_log(&log_a2).await.expect("create log a2");

        let log_b1 = LogCreate {
            agenda_id: agenda_b,
            content: "b1".to_string(),
            log_type: LogType::CommonLog,
        };
        let _id_b1 = repo.create_log(&log_b1).await.expect("create log b1");

        let result = repo
            .get_logs_by_agenda_id(agenda_a)
            .await
            .expect("query logs");

        assert_eq!(result.len(), 2);
        let ids: Vec<_> = result.iter().map(|l| l.id).collect();
        assert!(ids.contains(&id_a1));
        assert!(ids.contains(&id_a2));
        assert!(result.iter().all(|l| l.agenda_id == agenda_a));
    }

    #[tokio::test]
    async fn get_logs_by_agenda_id_empty() {
        let pool = setup_pool().await;
        let repo = SqliteLogRepo { pool: pool.clone() };

        let agenda_id = Uuid::now_v7();
        insert_agenda(&pool, agenda_id).await;

        let result = repo
            .get_logs_by_agenda_id(agenda_id)
            .await
            .expect("query logs");

        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn get_logs_by_time_range_filters() {
        let pool = setup_pool().await;
        let repo = SqliteLogRepo { pool: pool.clone() };

        let agenda_id = Uuid::now_v7();
        insert_agenda(&pool, agenda_id).await;

        let base = Timestamp::now();
        let t0 = base;
        let t1 = base + 1.seconds();
        let t2 = base + 20.seconds();
        let t3 = base + 30.seconds();

        sqlx::query(
            "INSERT INTO log (id, create_at, content, log_type, agenda_id) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(Uuid::now_v7().to_string())
        .bind(t0.as_millisecond())
        .bind("before")
        .bind("activate")
        .bind(agenda_id.to_string())
        .execute(&pool)
        .await
        .expect("insert before");

        let in_range_id = Uuid::now_v7();
        sqlx::query(
            "INSERT INTO log (id, create_at, content, log_type, agenda_id) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(in_range_id.to_string())
        .bind(t1.as_millisecond())
        .bind("in range")
        .bind("put_off")
        .bind(agenda_id.to_string())
        .execute(&pool)
        .await
        .expect("insert in range");

        sqlx::query(
            "INSERT INTO log (id, create_at, content, log_type, agenda_id) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(Uuid::now_v7().to_string())
        .bind(t3.as_millisecond())
        .bind("after")
        .bind("common_log")
        .bind(agenda_id.to_string())
        .execute(&pool)
        .await
        .expect("insert after");

        let result = repo
            .get_logs_by_time_range(t1, t2)
            .await
            .expect("query range");

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, in_range_id);
    }

    #[tokio::test]
    async fn get_logs_by_time_range_inclusive_bounds() {
        let pool = setup_pool().await;
        let repo = SqliteLogRepo { pool: pool.clone() };

        let agenda_id = Uuid::now_v7();
        insert_agenda(&pool, agenda_id).await;

        let base = Timestamp::now();
        let start = base + 5.seconds();
        let end = base + 15.seconds();

        let start_id = Uuid::now_v7();
        sqlx::query(
            "INSERT INTO log (id, create_at, content, log_type, agenda_id) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(start_id.to_string())
        .bind(start.as_millisecond())
        .bind("at start")
        .bind("activate")
        .bind(agenda_id.to_string())
        .execute(&pool)
        .await
        .expect("insert start");

        let end_id = Uuid::now_v7();
        sqlx::query(
            "INSERT INTO log (id, create_at, content, log_type, agenda_id) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(end_id.to_string())
        .bind(end.as_millisecond())
        .bind("at end")
        .bind("terminate")
        .bind(agenda_id.to_string())
        .execute(&pool)
        .await
        .expect("insert end");

        let result = repo
            .get_logs_by_time_range(start, end)
            .await
            .expect("query range");

        let ids: Vec<_> = result.iter().map(|l| l.id).collect();
        assert_eq!(result.len(), 2);
        assert!(ids.contains(&start_id));
        assert!(ids.contains(&end_id));
    }

    #[tokio::test]
    async fn get_logs_by_time_range_empty() {
        let pool = setup_pool().await;
        let repo = SqliteLogRepo { pool: pool.clone() };

        let agenda_id = Uuid::now_v7();
        insert_agenda(&pool, agenda_id).await;

        let base = Timestamp::now();
        let start = base + 100.seconds();
        let end = base + 200.seconds();

        let result = repo
            .get_logs_by_time_range(start, end)
            .await
            .expect("query range");

        assert!(result.is_empty());
    }
}
