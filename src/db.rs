use sqlx::{SqlitePool, migrate::MigrateDatabase};

const DB_URL: &str = "sqlite:quizlet.db";

pub async fn init_db() -> Result<SqlitePool, sqlx::Error> {
    // Create the database if it doesn't exist
    if !sqlx::Sqlite::database_exists(DB_URL).await? {
        sqlx::Sqlite::create_database(DB_URL).await?;
    }

    // Connect to the database
    let pool = SqlitePool::connect(DB_URL).await?;

    // Create the sets table
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS sets (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL UNIQUE
        )"
    )
    .execute(&pool)
    .await?;

    // Create the terms table
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS terms (
            id INTEGER PRIMARY KEY,
            set_id INTEGER NOT NULL,
            term TEXT NOT NULL,
            definition TEXT NOT NULL,
            FOREIGN KEY (set_id) REFERENCES sets(id)
        )"
    )
    .execute(&pool)
    .await?;

    // Create the sessions table
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS sessions (
            set_id INTEGER NOT NULL,
            term_id INTEGER NOT NULL,
            mastered BOOLEAN DEFAULT 0,
            PRIMARY KEY (set_id, term_id),
            FOREIGN KEY (set_id) REFERENCES sets(id),
            FOREIGN KEY (term_id) REFERENCES terms(id)
        )"
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT
        )"
    )
    .execute(&pool)
    .await?;

    Ok(pool)
}

pub async fn get_learning_direction(pool: &sqlx::SqlitePool) -> Result<String, sqlx::Error> {
    let direction: Option<String> = sqlx::query_scalar("SELECT value FROM settings WHERE key = 'learning_direction'")
        .fetch_optional(pool)
        .await?;
    Ok(direction.unwrap_or("term_to_definition".to_string())) // Default to term-to-definition
}

pub async fn set_learning_direction(pool: &sqlx::SqlitePool, direction: &str) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT OR REPLACE INTO settings (key, value) VALUES ('learning_direction', ?)")
        .bind(direction)
        .execute(pool)
        .await?;
    Ok(())
}
