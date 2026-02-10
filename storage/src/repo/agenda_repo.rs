use async_trait::async_trait;
use domain::*;
use jiff::Timestamp;
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

#[derive(FromRow)]
struct DbAgenda {
    id: Uuid,
    title: String,
    agenda_status: String,
    initiate_at: i64,
    terminate_at: i64,
}

pub struct SqliteAgendaRepo {
    pub pool: SqlitePool,
}

#[async_trait]
impl AgendaRepo for SqliteAgendaRepo {
    type Error = sqlx::Error;

    async fn create_agenda(&self, agenda: &AgendaCreate) -> Result<Uuid, Self::Error> {
        let uuid = Uuid::now_v7();
        let timestamp = Timestamp::now().as_millisecond();
        sqlx::query(
            "INSERT INTO agenda (id, title, agenda_status, initiate_at, terminate_at) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(uuid.to_string())
        .bind(&agenda.title)
        .bind(&agenda.agenda_status.to_string())
        .bind(&timestamp)
        .bind(&agenda.terminate_at.as_millisecond())
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

    #[tokio::test]
    async fn create_agenda_inserts_row() {
        let pool = setup_pool().await;
        let repo = SqliteAgendaRepo { pool: pool.clone() };

        let start_ms = Timestamp::now().as_millisecond();
        let terminate_at = Timestamp::now();
        let agenda = AgendaCreate {
            title: "First agenda".to_string(),
            agenda_status: AgendaStatus::Ongoing,
            terminate_at,
        };

        let created_id = repo.create_agenda(&agenda).await.expect("create agenda");
        let end_ms = Timestamp::now().as_millisecond();

        let row = sqlx::query(
            "SELECT id, title, agenda_status, initiate_at, terminate_at FROM agenda WHERE id = ?",
        )
        .bind(created_id.to_string())
        .fetch_one(&pool)
        .await
        .expect("fetch agenda row");

        let id: String = row.get("id");
        let title: String = row.get("title");
        let agenda_status: String = row.get("agenda_status");
        let initiate_at: i64 = row.get("initiate_at");
        let terminate_at_ms: i64 = row.get("terminate_at");

        assert_eq!(id, created_id.to_string());
        assert_eq!(title, "First agenda");
        assert_eq!(agenda_status, "ongoing");
        assert!(initiate_at >= start_ms && initiate_at <= end_ms);
        assert_eq!(terminate_at_ms, agenda.terminate_at.as_millisecond());
    }
}
