use async_trait::async_trait;
use domain::*;
use jiff::Timestamp;
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

#[derive(FromRow)]
struct DbAgenda {
    id: String,
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

    async fn delete_agenda_by_id(&self, id: Uuid) -> Result<(), Self::Error> {
        sqlx::query("DELETE FROM agenda WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn update_agenda(&self, id: Uuid, update: &AgendaUpdate) -> Result<(), Self::Error> {
        let mut query = "UPDATE agenda SET ".to_string();
        let mut args: Vec<(String, String)> = Vec::new();

        if let Some(title) = &update.title {
            query.push_str("title = ?, ");
            args.push(("title".to_string(), title.clone()));
        }
        if let Some(status) = &update.agenda_status {
            query.push_str("agenda_status = ?, ");
            args.push(("agenda_status".to_string(), status.to_string()));
        }
        if let Some(terminate_at) = &update.terminate_at {
            query.push_str("terminate_at = ?, ");
            args.push((
                "terminate_at".to_string(),
                terminate_at.as_millisecond().to_string(),
            ));
        }

        // If no fields to update, return early without executing query
        if args.is_empty() {
            return Ok(());
        }

        // Remove trailing comma and space
        query.truncate(query.len() - 2);
        query.push_str(" WHERE id = ?");

        let mut sql_query = sqlx::query(&query);
        for (_, value) in &args {
            sql_query = sql_query.bind(value);
        }
        sql_query = sql_query.bind(id.to_string());

        sql_query.execute(&self.pool).await?;
        Ok(())
    }

    async fn get_agenda_by_id(&self, id: Uuid) -> Result<Option<Agenda>, Self::Error> {
        let row = sqlx::query_as::<_, DbAgenda>("SELECT * FROM agenda WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|db_agenda| Agenda {
            id: Uuid::parse_str(&db_agenda.id).expect("invalid uuid in database"),
            title: db_agenda.title,
            agenda_status: match db_agenda.agenda_status.as_str() {
                "stored" => AgendaStatus::Stored,
                "ongoing" => AgendaStatus::Ongoing,
                "terminated" => AgendaStatus::Terminated,
                _ => panic!("invalid agenda_status in database"),
            },
            initiate_at: Timestamp::from_millisecond(db_agenda.initiate_at)
                .expect("invalid initiate_at in database"),
            terminate_at: Timestamp::from_millisecond(db_agenda.terminate_at)
                .expect("invalid terminate_at in database"),
        }))
    }

    async fn get_agendas_by_title(&self, title: &str) -> Result<Vec<Agenda>, Self::Error> {
        let rows = sqlx::query_as::<_, DbAgenda>("SELECT * FROM agenda WHERE title = ?")
            .bind(title)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows
            .into_iter()
            .map(|db_agenda| Agenda {
                id: Uuid::parse_str(&db_agenda.id).expect("invalid uuid in database"),
                title: db_agenda.title,
                agenda_status: match db_agenda.agenda_status.as_str() {
                    "stored" => AgendaStatus::Stored,
                    "ongoing" => AgendaStatus::Ongoing,
                    "terminated" => AgendaStatus::Terminated,
                    _ => panic!("invalid agenda_status in database"),
                },
                initiate_at: Timestamp::from_millisecond(db_agenda.initiate_at)
                    .expect("invalid initiate_at in database"),
                terminate_at: Timestamp::from_millisecond(db_agenda.terminate_at)
                    .expect("invalid terminate_at in database"),
            })
            .collect())
    }

    async fn get_agendas_by_status(
        &self,
        status: Option<&str>,
    ) -> Result<Vec<Agenda>, Self::Error> {
        let rows = if let Some(status) = status {
            sqlx::query_as::<_, DbAgenda>("SELECT * FROM agenda WHERE agenda_status = ?")
                .bind(status)
                .fetch_all(&self.pool)
                .await?
        } else {
            sqlx::query_as::<_, DbAgenda>("SELECT * FROM agenda")
                .fetch_all(&self.pool)
                .await?
        };

        Ok(rows
            .into_iter()
            .map(|db_agenda| Agenda {
                id: Uuid::parse_str(&db_agenda.id).expect("invalid uuid in database"),
                title: db_agenda.title,
                agenda_status: match db_agenda.agenda_status.as_str() {
                    "stored" => AgendaStatus::Stored,
                    "ongoing" => AgendaStatus::Ongoing,
                    "terminated" => AgendaStatus::Terminated,
                    _ => panic!("invalid agenda_status in database"),
                },
                initiate_at: Timestamp::from_millisecond(db_agenda.initiate_at)
                    .expect("invalid initiate_at in database"),
                terminate_at: Timestamp::from_millisecond(db_agenda.terminate_at)
                    .expect("invalid terminate_at in database"),
            })
            .collect())
    }
    async fn count_agendas_by_status(&self, status: Option<&str>) -> Result<u64, Self::Error> {
        let count: i64 = if let Some(status) = status {
            sqlx::query_scalar("SELECT COUNT(*) FROM agenda WHERE agenda_status = ?")
                .bind(status)
                .fetch_one(&self.pool)
                .await?
        } else {
            sqlx::query_scalar("SELECT COUNT(*) FROM agenda")
                .fetch_one(&self.pool)
                .await?
        };
        Ok(count as u64)
    }
    async fn get_agendas_by_terminate_time_range(
        &self,
        start: Timestamp,
        end: Timestamp,
    ) -> Result<Vec<Agenda>, Self::Error> {
        let rows = sqlx::query_as::<_, DbAgenda>(
            "SELECT * FROM agenda WHERE terminate_at >= ? AND terminate_at <= ?",
        )
        .bind(start.as_millisecond())
        .bind(end.as_millisecond())
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|db_agenda| Agenda {
                id: Uuid::parse_str(&db_agenda.id).expect("invalid uuid in database"),
                title: db_agenda.title,
                agenda_status: match db_agenda.agenda_status.as_str() {
                    "stored" => AgendaStatus::Stored,
                    "ongoing" => AgendaStatus::Ongoing,
                    "terminated" => AgendaStatus::Terminated,
                    _ => panic!("invalid agenda_status in database"),
                },
                initiate_at: Timestamp::from_millisecond(db_agenda.initiate_at)
                    .expect("invalid initiate_at in database"),
                terminate_at: Timestamp::from_millisecond(db_agenda.terminate_at)
                    .expect("invalid terminate_at in database"),
            })
            .collect())
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

    #[tokio::test]
    async fn delete_agenda_removes_row() {
        let pool = setup_pool().await;
        let repo = SqliteAgendaRepo { pool: pool.clone() };

        // create an agenda
        let terminate_at = Timestamp::now();
        let agenda = AgendaCreate {
            title: "To be deleted".to_string(),
            agenda_status: AgendaStatus::Stored,
            terminate_at,
        };
        let agenda_id = repo.create_agenda(&agenda).await.expect("create agenda");

        // verify insertion
        let count_before: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM agenda WHERE id = ?")
            .bind(agenda_id.to_string())
            .fetch_one(&pool)
            .await
            .expect("count before delete");
        assert_eq!(count_before, 1);

        // delete the agenda
        repo.delete_agenda_by_id(agenda_id)
            .await
            .expect("delete agenda");

        // verify deletion
        let count_after: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM agenda WHERE id = ?")
            .bind(agenda_id.to_string())
            .fetch_one(&pool)
            .await
            .expect("count after delete");
        assert_eq!(count_after, 0);
    }

    #[tokio::test]
    async fn delete_agenda_idempotent() {
        let pool = setup_pool().await;
        let repo = SqliteAgendaRepo { pool: pool.clone() };

        // Attempt to delete a non-existent ID, should succeed without error (SQLite behavior)
        let non_existent_id = Uuid::now_v7();
        let result = repo.delete_agenda_by_id(non_existent_id).await;
        assert!(
            result.is_ok(),
            "delete non-existent agenda should not error"
        );
    }

    #[tokio::test]
    async fn update_agenda_single_field() {
        let pool = setup_pool().await;
        let repo = SqliteAgendaRepo { pool: pool.clone() };

        // create an agenda
        let terminate_at = Timestamp::now();
        let agenda = AgendaCreate {
            title: "Original title".to_string(),
            agenda_status: AgendaStatus::Ongoing,
            terminate_at,
        };
        let agenda_id = repo.create_agenda(&agenda).await.expect("create agenda");

        // update title
        let update = AgendaUpdate {
            title: Some("Updated title".to_string()),
            agenda_status: None,
            terminate_at: None,
        };
        repo.update_agenda(agenda_id, &update)
            .await
            .expect("update agenda");

        // verify title update, other fields remain unchanged
        let row = sqlx::query("SELECT title, agenda_status, terminate_at FROM agenda WHERE id = ?")
            .bind(agenda_id.to_string())
            .fetch_one(&pool)
            .await
            .expect("fetch agenda row");

        let title: String = row.get("title");
        let status: String = row.get("agenda_status");
        let term_at: i64 = row.get("terminate_at");

        assert_eq!(title, "Updated title");
        assert_eq!(status, "ongoing"); // unchanged
        assert_eq!(term_at, terminate_at.as_millisecond()); // unchanged
    }

    #[tokio::test]
    async fn update_agenda_multiple_fields() {
        let pool = setup_pool().await;
        let repo = SqliteAgendaRepo { pool: pool.clone() };

        // create an agenda
        let terminate_at = Timestamp::now();
        let agenda = AgendaCreate {
            title: "Original".to_string(),
            agenda_status: AgendaStatus::Stored,
            terminate_at,
        };
        let agenda_id = repo.create_agenda(&agenda).await.expect("create agenda");

        // update title and status, not terminate_at
        let new_status = AgendaStatus::Ongoing;
        let update = AgendaUpdate {
            title: Some("New title".to_string()),
            agenda_status: Some(new_status),
            terminate_at: None,
        };
        repo.update_agenda(agenda_id, &update)
            .await
            .expect("update agenda");

        let row = sqlx::query("SELECT title, agenda_status, terminate_at FROM agenda WHERE id = ?")
            .bind(agenda_id.to_string())
            .fetch_one(&pool)
            .await
            .expect("fetch agenda row");

        let title: String = row.get("title");
        let status: String = row.get("agenda_status");
        let term_at: i64 = row.get("terminate_at");

        assert_eq!(title, "New title");
        assert_eq!(status, "ongoing");
        assert_eq!(term_at, terminate_at.as_millisecond()); // 保持不变
    }

    #[tokio::test]
    async fn update_agenda_all_fields() {
        let pool = setup_pool().await;
        let repo = SqliteAgendaRepo { pool: pool.clone() };

        // create an agenda
        let original_terminate = Timestamp::now();
        let agenda = AgendaCreate {
            title: "Original".to_string(),
            agenda_status: AgendaStatus::Stored,
            terminate_at: original_terminate,
        };
        let agenda_id = repo.create_agenda(&agenda).await.expect("create agenda");

        // update all fields
        let new_terminate = Timestamp::now();
        let update = AgendaUpdate {
            title: Some("Fully updated".to_string()),
            agenda_status: Some(AgendaStatus::Terminated),
            terminate_at: Some(new_terminate),
        };
        repo.update_agenda(agenda_id, &update)
            .await
            .expect("update agenda");

        let row = sqlx::query("SELECT title, agenda_status, terminate_at FROM agenda WHERE id = ?")
            .bind(agenda_id.to_string())
            .fetch_one(&pool)
            .await
            .expect("fetch agenda row");

        let title: String = row.get("title");
        let status: String = row.get("agenda_status");
        let term_at: i64 = row.get("terminate_at");

        assert_eq!(title, "Fully updated");
        assert_eq!(status, "terminated");
        assert_eq!(term_at, new_terminate.as_millisecond());
    }

    #[tokio::test]
    async fn update_agenda_nonexistent_id() {
        let pool = setup_pool().await;
        let repo = SqliteAgendaRepo { pool: pool.clone() };

        // Attempt to update a non-existent ID, should succeed without effect (SQLite behavior)
        let non_existent_id = Uuid::now_v7();
        let update = AgendaUpdate {
            title: Some("Won't be saved".to_string()),
            agenda_status: None,
            terminate_at: None,
        };

        let result = repo.update_agenda(non_existent_id, &update).await;
        assert!(result.is_ok(), "update nonexistent agenda should not error");

        // Verify that no rows were updated (i.e., the non-existent ID is still not present)
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM agenda WHERE id = ?")
            .bind(non_existent_id.to_string())
            .fetch_one(&pool)
            .await
            .expect("count rows");
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn update_agenda_empty_update() {
        let pool = setup_pool().await;
        let repo = SqliteAgendaRepo { pool: pool.clone() };

        // create an agenda
        let terminate_at = Timestamp::now();
        let agenda = AgendaCreate {
            title: "Original title".to_string(),
            agenda_status: AgendaStatus::Ongoing,
            terminate_at,
        };
        let agenda_id = repo.create_agenda(&agenda).await.expect("create agenda");

        // empty update (all fields are None)
        let update = AgendaUpdate {
            title: None,
            agenda_status: None,
            terminate_at: None,
        };
        repo.update_agenda(agenda_id, &update)
            .await
            .expect("update agenda");

        // verify all fields remain unchanged
        let row = sqlx::query("SELECT title, agenda_status, terminate_at FROM agenda WHERE id = ?")
            .bind(agenda_id.to_string())
            .fetch_one(&pool)
            .await
            .expect("fetch agenda row");

        let title: String = row.get("title");
        let status: String = row.get("agenda_status");
        let term_at: i64 = row.get("terminate_at");

        assert_eq!(title, "Original title");
        assert_eq!(status, "ongoing");
        assert_eq!(term_at, terminate_at.as_millisecond());
    }

    #[tokio::test]
    async fn get_agenda_by_id_found() {
        let pool = setup_pool().await;
        let repo = SqliteAgendaRepo { pool: pool.clone() };

        // create an agenda
        let terminate_at = Timestamp::now();
        let agenda = AgendaCreate {
            title: "Test agenda".to_string(),
            agenda_status: AgendaStatus::Ongoing,
            terminate_at,
        };
        let created_id = repo.create_agenda(&agenda).await.expect("create agenda");

        // query the newly created agenda
        let result = repo
            .get_agenda_by_id(created_id)
            .await
            .expect("query agenda");

        assert!(result.is_some(), "agenda should be found");
        let fetched_agenda = result.unwrap();

        assert_eq!(fetched_agenda.id, created_id);
        assert_eq!(fetched_agenda.title, "Test agenda");
        match fetched_agenda.agenda_status {
            AgendaStatus::Ongoing => {}
            _ => panic!("expected Ongoing status"),
        }
        assert_eq!(
            fetched_agenda.terminate_at.as_millisecond(),
            terminate_at.as_millisecond()
        );
    }

    #[tokio::test]
    async fn get_agenda_by_id_not_found() {
        let pool = setup_pool().await;
        let repo = SqliteAgendaRepo { pool: pool.clone() };

        // query a non-existent ID
        let non_existent_id = Uuid::now_v7();
        let result = repo
            .get_agenda_by_id(non_existent_id)
            .await
            .expect("query should succeed");

        assert!(result.is_none(), "non-existent agenda should return None");
    }

    #[tokio::test]
    async fn get_agenda_by_id_all_status_types() {
        let pool = setup_pool().await;
        let repo = SqliteAgendaRepo { pool: pool.clone() };

        let terminate_at = Timestamp::now();

        // test all status types
        let statuses = vec![
            AgendaStatus::Stored,
            AgendaStatus::Ongoing,
            AgendaStatus::Terminated,
        ];

        for (idx, status) in statuses.into_iter().enumerate() {
            let agenda = AgendaCreate {
                title: format!("Agenda {}", idx),
                agenda_status: status,
                terminate_at,
            };
            let agenda_id = repo.create_agenda(&agenda).await.expect("create agenda");

            // query and verify status is correctly parsed
            let fetched = repo
                .get_agenda_by_id(agenda_id)
                .await
                .expect("query agenda")
                .expect("agenda should exist");

            match (&fetched.agenda_status, idx) {
                (AgendaStatus::Stored, 0) => {}
                (AgendaStatus::Ongoing, 1) => {}
                (AgendaStatus::Terminated, 2) => {}
                _ => panic!("status mismatch at index {}", idx),
            }
        }
    }

    #[tokio::test]
    async fn get_agenda_by_id_after_update() {
        let pool = setup_pool().await;
        let repo = SqliteAgendaRepo { pool: pool.clone() };

        // create an agenda
        let terminate_at = Timestamp::now();
        let agenda = AgendaCreate {
            title: "Original".to_string(),
            agenda_status: AgendaStatus::Stored,
            terminate_at,
        };
        let agenda_id = repo.create_agenda(&agenda).await.expect("create agenda");

        // update the agenda
        let new_terminate = Timestamp::now();
        let update = AgendaUpdate {
            title: Some("Updated".to_string()),
            agenda_status: Some(AgendaStatus::Ongoing),
            terminate_at: Some(new_terminate),
        };
        repo.update_agenda(agenda_id, &update)
            .await
            .expect("update agenda");

        // query and verify the updated values
        let fetched = repo
            .get_agenda_by_id(agenda_id)
            .await
            .expect("query agenda")
            .expect("agenda should exist");

        assert_eq!(fetched.title, "Updated");
        match fetched.agenda_status {
            AgendaStatus::Ongoing => {}
            _ => panic!("expected Ongoing status"),
        }
        assert_eq!(
            fetched.terminate_at.as_millisecond(),
            new_terminate.as_millisecond()
        );
    }

    #[tokio::test]
    async fn get_agendas_by_status_filter_by_status() {
        let pool = setup_pool().await;
        let repo = SqliteAgendaRepo { pool: pool.clone() };

        let terminate_at = Timestamp::now();

        // create multiple agendas with different statuses
        let stored_agenda = AgendaCreate {
            title: "Stored agenda".to_string(),
            agenda_status: AgendaStatus::Stored,
            terminate_at,
        };
        let stored_id = repo
            .create_agenda(&stored_agenda)
            .await
            .expect("create stored");

        let ongoing_agenda = AgendaCreate {
            title: "Ongoing agenda".to_string(),
            agenda_status: AgendaStatus::Ongoing,
            terminate_at,
        };
        let ongoing_id = repo
            .create_agenda(&ongoing_agenda)
            .await
            .expect("create ongoing");

        let terminated_agenda = AgendaCreate {
            title: "Terminated agenda".to_string(),
            agenda_status: AgendaStatus::Terminated,
            terminate_at,
        };
        let _terminated_id = repo
            .create_agenda(&terminated_agenda)
            .await
            .expect("create terminated");

        // query by "ongoing" status
        let result = repo
            .get_agendas_by_status(Some("ongoing"))
            .await
            .expect("query by status");

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, ongoing_id);
        assert_eq!(result[0].title, "Ongoing agenda");
        match result[0].agenda_status {
            AgendaStatus::Ongoing => {}
            _ => panic!("expected Ongoing status"),
        }

        // query by "stored" status
        let result = repo
            .get_agendas_by_status(Some("stored"))
            .await
            .expect("query by status");

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, stored_id);
        assert_eq!(result[0].title, "Stored agenda");
    }

    #[tokio::test]
    async fn get_agendas_by_status_get_all_when_none() {
        let pool = setup_pool().await;
        let repo = SqliteAgendaRepo { pool: pool.clone() };

        let terminate_at = Timestamp::now();

        // create multiple agendas with different statuses
        let agenda1 = AgendaCreate {
            title: "Agenda 1".to_string(),
            agenda_status: AgendaStatus::Stored,
            terminate_at,
        };
        repo.create_agenda(&agenda1).await.expect("create agenda1");

        let agenda2 = AgendaCreate {
            title: "Agenda 2".to_string(),
            agenda_status: AgendaStatus::Ongoing,
            terminate_at,
        };
        repo.create_agenda(&agenda2).await.expect("create agenda2");

        let agenda3 = AgendaCreate {
            title: "Agenda 3".to_string(),
            agenda_status: AgendaStatus::Terminated,
            terminate_at,
        };
        repo.create_agenda(&agenda3).await.expect("create agenda3");

        // query all agendas when status is None
        let result = repo
            .get_agendas_by_status(None)
            .await
            .expect("query all agendas");

        assert_eq!(result.len(), 3);
        assert_eq!(result.iter().map(|a| &a.title).collect::<Vec<_>>().len(), 3);
    }

    #[tokio::test]
    async fn get_agendas_by_status_empty_result() {
        let pool = setup_pool().await;
        let repo = SqliteAgendaRepo { pool: pool.clone() };

        let terminate_at = Timestamp::now();

        // create agendas with only "ongoing" status
        let agenda = AgendaCreate {
            title: "Only ongoing".to_string(),
            agenda_status: AgendaStatus::Ongoing,
            terminate_at,
        };
        repo.create_agenda(&agenda).await.expect("create agenda");

        // query by non-matching status "stored"
        let result = repo
            .get_agendas_by_status(Some("stored"))
            .await
            .expect("query by status");

        assert_eq!(result.len(), 0);
    }

    #[tokio::test]
    async fn get_agendas_by_status_multiple_with_same_status() {
        let pool = setup_pool().await;
        let repo = SqliteAgendaRepo { pool: pool.clone() };

        let terminate_at = Timestamp::now();

        // create multiple agendas with the same "ongoing" status
        let agenda1 = AgendaCreate {
            title: "Ongoing 1".to_string(),
            agenda_status: AgendaStatus::Ongoing,
            terminate_at,
        };
        let id1 = repo.create_agenda(&agenda1).await.expect("create agenda1");

        let agenda2 = AgendaCreate {
            title: "Ongoing 2".to_string(),
            agenda_status: AgendaStatus::Ongoing,
            terminate_at,
        };
        let id2 = repo.create_agenda(&agenda2).await.expect("create agenda2");

        // query by "ongoing" status
        let result = repo
            .get_agendas_by_status(Some("ongoing"))
            .await
            .expect("query by status");

        assert_eq!(result.len(), 2);
        let ids: Vec<_> = result.iter().map(|a| a.id).collect();
        assert!(ids.contains(&id1));
        assert!(ids.contains(&id2));
    }

    #[tokio::test]
    async fn get_agendas_by_terminate_time_range_filters() {
        let pool = setup_pool().await;
        let repo = SqliteAgendaRepo { pool: pool.clone() };

        let base = Timestamp::now();
        let t0 = base;
        let t1 = base + 10.seconds();
        let t2 = base + 20.seconds();
        let t3 = base + 30.seconds();

        let agenda_before = AgendaCreate {
            title: "Before".to_string(),
            agenda_status: AgendaStatus::Stored,
            terminate_at: t0,
        };
        repo.create_agenda(&agenda_before)
            .await
            .expect("create before");

        let agenda_in_range = AgendaCreate {
            title: "In range".to_string(),
            agenda_status: AgendaStatus::Ongoing,
            terminate_at: t1,
        };
        let in_range_id = repo
            .create_agenda(&agenda_in_range)
            .await
            .expect("create in range");

        let agenda_after = AgendaCreate {
            title: "After".to_string(),
            agenda_status: AgendaStatus::Terminated,
            terminate_at: t3,
        };
        repo.create_agenda(&agenda_after)
            .await
            .expect("create after");

        let result = repo
            .get_agendas_by_terminate_time_range(t1, t2)
            .await
            .expect("query range");

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, in_range_id);
    }

    #[tokio::test]
    async fn get_agendas_by_terminate_time_range_inclusive_bounds() {
        let pool = setup_pool().await;
        let repo = SqliteAgendaRepo { pool: pool.clone() };

        let base = Timestamp::now();
        let start = base + 5.seconds();
        let end = base + 15.seconds();

        let agenda_start = AgendaCreate {
            title: "At start".to_string(),
            agenda_status: AgendaStatus::Stored,
            terminate_at: start,
        };
        let start_id = repo
            .create_agenda(&agenda_start)
            .await
            .expect("create start");

        let agenda_end = AgendaCreate {
            title: "At end".to_string(),
            agenda_status: AgendaStatus::Ongoing,
            terminate_at: end,
        };
        let end_id = repo.create_agenda(&agenda_end).await.expect("create end");

        let result = repo
            .get_agendas_by_terminate_time_range(start, end)
            .await
            .expect("query range");

        let ids: Vec<_> = result.iter().map(|a| a.id).collect();
        assert_eq!(result.len(), 2);
        assert!(ids.contains(&start_id));
        assert!(ids.contains(&end_id));
    }

    #[tokio::test]
    async fn get_agendas_by_terminate_time_range_empty() {
        let pool = setup_pool().await;
        let repo = SqliteAgendaRepo { pool: pool.clone() };

        let base = Timestamp::now();
        let start = base + 100.seconds();
        let end = base + 200.seconds();

        let result = repo
            .get_agendas_by_terminate_time_range(start, end)
            .await
            .expect("query range");

        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn get_agendas_by_title_exact_match() {
        let pool = setup_pool().await;
        let repo = SqliteAgendaRepo { pool: pool.clone() };

        let terminate_at = Timestamp::now();
        let agenda = AgendaCreate {
            title: "Title A".to_string(),
            agenda_status: AgendaStatus::Ongoing,
            terminate_at,
        };
        let agenda_id = repo.create_agenda(&agenda).await.expect("create agenda");

        let result = repo
            .get_agendas_by_title("Title A")
            .await
            .expect("query by title");

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, agenda_id);
        assert_eq!(result[0].title, "Title A");
    }

    #[tokio::test]
    async fn get_agendas_by_title_multiple_results() {
        let pool = setup_pool().await;
        let repo = SqliteAgendaRepo { pool: pool.clone() };

        let terminate_at = Timestamp::now();
        let agenda1 = AgendaCreate {
            title: "Same Title".to_string(),
            agenda_status: AgendaStatus::Stored,
            terminate_at,
        };
        let id1 = repo.create_agenda(&agenda1).await.expect("create agenda1");

        let agenda2 = AgendaCreate {
            title: "Same Title".to_string(),
            agenda_status: AgendaStatus::Ongoing,
            terminate_at,
        };
        let id2 = repo.create_agenda(&agenda2).await.expect("create agenda2");

        let result = repo
            .get_agendas_by_title("Same Title")
            .await
            .expect("query by title");

        assert_eq!(result.len(), 2);
        let ids: Vec<_> = result.iter().map(|a| a.id).collect();
        assert!(ids.contains(&id1));
        assert!(ids.contains(&id2));
    }

    #[tokio::test]
    async fn get_agendas_by_title_empty_result() {
        let pool = setup_pool().await;
        let repo = SqliteAgendaRepo { pool: pool.clone() };

        let terminate_at = Timestamp::now();
        let agenda = AgendaCreate {
            title: "Existing".to_string(),
            agenda_status: AgendaStatus::Stored,
            terminate_at,
        };
        repo.create_agenda(&agenda).await.expect("create agenda");

        let result = repo
            .get_agendas_by_title("Missing")
            .await
            .expect("query by title");

        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn count_agendas_by_status_with_filter() {
        let pool = setup_pool().await;
        let repo = SqliteAgendaRepo { pool: pool.clone() };

        let terminate_at = Timestamp::now();
        let agenda1 = AgendaCreate {
            title: "Stored 1".to_string(),
            agenda_status: AgendaStatus::Stored,
            terminate_at,
        };
        repo.create_agenda(&agenda1).await.expect("create agenda1");

        let agenda2 = AgendaCreate {
            title: "Stored 2".to_string(),
            agenda_status: AgendaStatus::Stored,
            terminate_at,
        };
        repo.create_agenda(&agenda2).await.expect("create agenda2");

        let agenda3 = AgendaCreate {
            title: "Ongoing 1".to_string(),
            agenda_status: AgendaStatus::Ongoing,
            terminate_at,
        };
        repo.create_agenda(&agenda3).await.expect("create agenda3");

        let stored_count = repo
            .count_agendas_by_status(Some("stored"))
            .await
            .expect("count stored");
        let ongoing_count = repo
            .count_agendas_by_status(Some("ongoing"))
            .await
            .expect("count ongoing");

        assert_eq!(stored_count, 2);
        assert_eq!(ongoing_count, 1);
    }

    #[tokio::test]
    async fn count_agendas_by_status_without_filter() {
        let pool = setup_pool().await;
        let repo = SqliteAgendaRepo { pool: pool.clone() };

        let terminate_at = Timestamp::now();
        let agenda1 = AgendaCreate {
            title: "A".to_string(),
            agenda_status: AgendaStatus::Stored,
            terminate_at,
        };
        repo.create_agenda(&agenda1).await.expect("create agenda1");

        let agenda2 = AgendaCreate {
            title: "B".to_string(),
            agenda_status: AgendaStatus::Ongoing,
            terminate_at,
        };
        repo.create_agenda(&agenda2).await.expect("create agenda2");

        let agenda3 = AgendaCreate {
            title: "C".to_string(),
            agenda_status: AgendaStatus::Terminated,
            terminate_at,
        };
        repo.create_agenda(&agenda3).await.expect("create agenda3");

        let total_count = repo.count_agendas_by_status(None).await.expect("count all");

        assert_eq!(total_count, 3);
    }
}
