//! Test helpers for creating in-memory test databases
//!
//! This module provides utilities for setting up test databases with sample data
//! for unit testing ML functionality.

use ritmo_errors::RitmoResult;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

/// Creates an in-memory SQLite database with the ritmo schema and test data
pub async fn create_test_db() -> RitmoResult<SqlitePool> {
    // Create in-memory database
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await?;

    // Create tables (simplified schema for tests)
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS "people" (
            "id" INTEGER PRIMARY KEY AUTOINCREMENT,
            "name" TEXT NOT NULL,
            "display_name" TEXT,
            "given_name" TEXT,
            "surname" TEXT,
            "middle_names" TEXT,
            "title" TEXT,
            "suffix" TEXT,
            "nationality" TEXT,
            "birth_date" INTEGER,
            "death_date" INTEGER,
            "biography" TEXT,
            "normalized_key" TEXT,
            "confidence" REAL NOT NULL DEFAULT 1.0 CHECK("confidence" >= 0.0 AND "confidence" <= 1.0),
            "source" TEXT NOT NULL DEFAULT 'biblioteca',
            "verified" INTEGER NOT NULL DEFAULT 0 CHECK("verified" IN (0, 1)),
            "created_at" INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
            "updated_at" INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
        );

        CREATE TABLE IF NOT EXISTS "publishers" (
            "id" INTEGER PRIMARY KEY AUTOINCREMENT,
            "name" TEXT NOT NULL,
            "country" TEXT,
            "website" TEXT,
            "notes" TEXT,
            "created_at" INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
            "updated_at" INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
        );

        CREATE TABLE IF NOT EXISTS "series" (
            "id" INTEGER PRIMARY KEY AUTOINCREMENT,
            "name" TEXT NOT NULL,
            "description" TEXT,
            "total_books" INTEGER,
            "completed" INTEGER NOT NULL DEFAULT 0 CHECK("completed" IN (0, 1)),
            "created_at" INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
            "updated_at" INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
        );

        CREATE TABLE IF NOT EXISTS "tags" (
            "id" INTEGER PRIMARY KEY AUTOINCREMENT,
            "name" TEXT NOT NULL UNIQUE,
            "description" TEXT,
            "created_at" INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
        );

        CREATE TABLE IF NOT EXISTS "roles" (
            "id" INTEGER PRIMARY KEY AUTOINCREMENT,
            "name" TEXT NOT NULL UNIQUE,
            "created_at" INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
        );

        CREATE TABLE IF NOT EXISTS "books" (
            "id" INTEGER PRIMARY KEY AUTOINCREMENT,
            "name" TEXT NOT NULL,
            "publisher_id" INTEGER,
            "series_id" INTEGER,
            "created_at" INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
            FOREIGN KEY("publisher_id") REFERENCES "publishers"("id") ON DELETE SET NULL,
            FOREIGN KEY("series_id") REFERENCES "series"("id") ON DELETE SET NULL
        );

        CREATE TABLE IF NOT EXISTS "contents" (
            "id" INTEGER PRIMARY KEY AUTOINCREMENT,
            "name" TEXT NOT NULL,
            "created_at" INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
        );

        CREATE TABLE IF NOT EXISTS "x_books_people_roles" (
            "book_id" INTEGER NOT NULL,
            "person_id" INTEGER NOT NULL,
            "role_id" INTEGER NOT NULL,
            PRIMARY KEY("book_id", "person_id", "role_id"),
            FOREIGN KEY("book_id") REFERENCES "books"("id") ON DELETE CASCADE,
            FOREIGN KEY("person_id") REFERENCES "people"("id") ON DELETE CASCADE,
            FOREIGN KEY("role_id") REFERENCES "roles"("id") ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS "x_contents_people_roles" (
            "content_id" INTEGER NOT NULL,
            "person_id" INTEGER NOT NULL,
            "role_id" INTEGER NOT NULL,
            PRIMARY KEY("content_id", "person_id", "role_id"),
            FOREIGN KEY("content_id") REFERENCES "contents"("id") ON DELETE CASCADE,
            FOREIGN KEY("person_id") REFERENCES "people"("id") ON DELETE CASCADE,
            FOREIGN KEY("role_id") REFERENCES "roles"("id") ON DELETE CASCADE
        );
        "#,
    )
    .execute(&pool)
    .await?;

    Ok(pool)
}

/// Populate database with test people data (includes duplicates for deduplication testing)
pub async fn populate_test_people(pool: &SqlitePool) -> RitmoResult<()> {
    sqlx::query(
        r#"
        INSERT INTO people (id, name, normalized_key) VALUES
        (1, 'Stephen King', 'stephen king'),
        (2, 'Stephen Edwin King', 'stephen edwin king'),
        (3, 'King, Stephen', 'king stephen'),
        (4, 'S. King', 's king'),
        (5, 'Margaret Atwood', 'margaret atwood'),
        (6, 'M. Atwood', 'm atwood'),
        (7, 'George R.R. Martin', 'george rr martin'),
        (8, 'George R. R. Martin', 'george r r martin'),
        (9, 'G.R.R. Martin', 'grr martin'),
        (10, 'J.K. Rowling', 'jk rowling'),
        (11, 'Joanne K. Rowling', 'joanne k rowling'),
        (12, 'J. K. Rowling', 'j k rowling')
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Populate database with test publishers data (includes duplicates)
pub async fn populate_test_publishers(pool: &SqlitePool) -> RitmoResult<()> {
    sqlx::query(
        r#"
        INSERT INTO publishers (id, name) VALUES
        (1, 'Penguin Random House'),
        (2, 'Penguin Books'),
        (3, 'Random House'),
        (4, 'HarperCollins'),
        (5, 'Harper Collins'),
        (6, 'Simon & Schuster'),
        (7, 'Simon and Schuster'),
        (8, 'Macmillan Publishers'),
        (9, 'Macmillan')
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Populate database with test series data (includes duplicates)
pub async fn populate_test_series(pool: &SqlitePool) -> RitmoResult<()> {
    sqlx::query(
        r#"
        INSERT INTO series (id, name) VALUES
        (1, 'The Dark Tower'),
        (2, 'Dark Tower'),
        (3, 'Harry Potter'),
        (4, 'Harry Potter Series'),
        (5, 'A Song of Ice and Fire'),
        (6, 'Song of Ice and Fire'),
        (7, 'The Handmaids Tale'),
        (8, 'Handmaid''s Tale')
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Populate database with test tags data (includes duplicates)
pub async fn populate_test_tags(pool: &SqlitePool) -> RitmoResult<()> {
    sqlx::query(
        r#"
        INSERT INTO tags (id, name) VALUES
        (1, 'Fantasy'),
        (2, 'fantasy'),
        (3, 'Science Fiction'),
        (4, 'Sci-Fi'),
        (5, 'Horror'),
        (6, 'horror'),
        (7, 'Thriller'),
        (8, 'thriller')
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Populate test database with sample roles including duplicates
pub async fn populate_test_roles(pool: &SqlitePool) -> RitmoResult<()> {
    sqlx::query(
        r#"
        INSERT INTO roles (id, name) VALUES
        (1, 'Autore'),
        (2, 'Author'),
        (3, 'Scrittore'),
        (4, 'Traduttore'),
        (5, 'Translator'),
        (6, 'Illustratore'),
        (7, 'Illustrator'),
        (8, 'Editore')
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Create a fully populated test database with all entity types
pub async fn create_full_test_db() -> RitmoResult<SqlitePool> {
    let pool = create_test_db().await?;
    populate_test_people(&pool).await?;
    populate_test_publishers(&pool).await?;
    populate_test_series(&pool).await?;
    populate_test_tags(&pool).await?;
    populate_test_roles(&pool).await?;
    Ok(pool)
}

/// Create test books linked to people (for merge testing)
pub async fn populate_test_books_with_people(pool: &SqlitePool) -> RitmoResult<()> {
    // Ensure roles exist (role_id 1 will be "Autore")
    populate_test_roles(pool).await?;

    // Create some test books
    sqlx::query(
        r#"
        INSERT INTO books (id, name) VALUES
        (1, 'The Shining'),
        (2, 'It'),
        (3, 'The Stand')
        "#,
    )
    .execute(pool)
    .await?;

    // Link books to people (role_id 1 is "Autore")
    sqlx::query(
        r#"
        INSERT INTO x_books_people_roles (book_id, person_id, role_id) VALUES
        (1, 1, 1),  -- The Shining by Stephen King (id=1)
        (2, 2, 1),  -- It by Stephen Edwin King (id=2) - duplicate
        (3, 3, 1)   -- The Stand by King, Stephen (id=3) - duplicate
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}
