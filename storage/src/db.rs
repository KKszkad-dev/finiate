use super::repo::{agenda_repo::SqliteAgendaRepo, log_repo::SqliteLogRepo};
use sqlx::{SqlitePool, migrate::MigrateDatabase, sqlite};
const DB_URL: &str = "sqlite://finiate.db";

pub async fn init_db() -> Result<SqlitePool, sqlx::Error> {
    if !sqlite::Sqlite::database_exists(DB_URL)
        .await
        .unwrap_or(false)
    {
        sqlite::Sqlite::create_database(DB_URL).await?;
        println!("Database created.");
    } else {
        println!("Database already exists.")
    }

    let pool = SqlitePool::connect(DB_URL).await?;

    // use env! to get the stable storage crate directory path
    let crate_dir = env!("CARGO_MANIFEST_DIR");
    println!("crate_dir: {}", crate_dir);
    let migrations = std::path::Path::new(&crate_dir).join("./migrations");
    let migration_results = sqlx::migrate::Migrator::new(migrations)
        .await
        .unwrap()
        .run(&pool)
        .await;
    match migration_results {
        Ok(_) => println!("Migration success"),
        Err(error) => {
            panic!("error: {}", error);
        }
    }
    println!("migration: {:?}", migration_results);
    // migration code end
    Ok(pool)
}

pub fn create_repos(pool: &SqlitePool) -> (SqliteAgendaRepo, SqliteLogRepo) {
    let agenda_repo = SqliteAgendaRepo { pool: pool.clone() };
    let log_repo = SqliteLogRepo { pool: pool.clone() };
    (agenda_repo, log_repo)
}
