/// GUI-specific queries that return nested data structures
///
/// These queries are optimized for GUI display and return complete nested data
/// (books with their contents and people, contents with their people and books)
use crate::models::{Format, Role, Type};
use crate::i18n_trait::I18nDisplayable;
use sqlx::SqlitePool;
use std::collections::HashMap;

/// Helper function to translate format key to localized name
pub async fn translate_format_key(pool: &SqlitePool, key: &str) -> String {
    match Format::get_by_key(pool, key).await {
        Ok(Some(format)) => format.translate(),
        _ => key.to_string(),
    }
}

/// Helper function to translate type key to localized name
pub async fn translate_type_key(pool: &SqlitePool, key: &str) -> String {
    match Type::get_by_key(pool, key).await {
        Ok(Some(type_)) => type_.translate(),
        _ => key.to_string(),
    }
}

/// Helper function to translate role key to localized name
pub async fn translate_role_key(pool: &SqlitePool, key: &str) -> String {
    match Role::get_by_key(pool, key).await {
        Ok(Some(role)) => role.translate(),
        _ => key.to_string(),
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct BookRow {
    pub book_id: i64,
    pub book_name: String,
    pub book_original_title: Option<String>,
    pub book_publisher: Option<String>,
    pub book_format_key: Option<String>,
    pub book_series: Option<String>,
    pub book_publication_date: Option<i64>,
    pub book_isbn: Option<String>,
    pub book_file_link: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ContentRow {
    pub content_id: i64,
    pub content_name: String,
    pub content_original_title: Option<String>,
    pub content_type_key: Option<String>,
    pub content_publication_date: Option<i64>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PersonRow {
    pub person_id: i64,
    pub person_name: String,
    pub role_key: String,
}

#[derive(Debug, Clone)]
pub struct BookWithDetails {
    pub book_id: i64,
    pub book_name: String,
    pub book_original_title: Option<String>,
    pub book_publisher: Option<String>,
    pub book_format_key: Option<String>,
    pub book_series: Option<String>,
    pub book_publication_date: Option<i64>,
    pub book_isbn: Option<String>,
    pub book_file_link: Option<String>,
    pub contents: Vec<ContentWithPeople>,
}

#[derive(Debug, Clone)]
pub struct ContentWithPeople {
    pub content_id: i64,
    pub content_name: String,
    pub content_original_title: Option<String>,
    pub content_type_key: Option<String>,
    pub content_publication_date: Option<i64>,
    pub people: Vec<PersonWithRole>,
}

#[derive(Debug, Clone)]
pub struct PersonWithRole {
    pub person_id: i64,
    pub person_name: String,
    pub role_key: String,
}

#[derive(Debug, Clone)]
pub struct ContentWithDetails {
    pub content_id: i64,
    pub content_name: String,
    pub content_original_title: Option<String>,
    pub content_type_key: Option<String>,
    pub content_publication_date: Option<i64>,
    pub people: Vec<PersonWithRole>,
    pub books: Vec<BookBasicInfo>,
}

#[derive(Debug, Clone)]
pub struct BookBasicInfo {
    pub book_id: i64,
    pub book_name: String,
    pub book_publisher: Option<String>,
    pub book_format_key: Option<String>,
    pub book_publication_date: Option<i64>,
}

/// Get books with their contents and all associated people
pub async fn get_books_with_nested_data(
    pool: &SqlitePool,
    search: Option<&str>,
) -> Result<Vec<BookWithDetails>, sqlx::Error> {
    // Query 1: Get books
    let books = if let Some(search_term) = search {
        sqlx::query_as::<_, BookRow>(
            r#"
            SELECT DISTINCT
                b.id as book_id,
                b.name as book_name,
                b.original_title as book_original_title,
                pub.name as book_publisher,
                f.key as book_format_key,
                s.name as book_series,
                b.publication_date as book_publication_date,
                b.isbn as book_isbn,
                b.file_link as book_file_link
            FROM books b
            LEFT JOIN publishers pub ON b.publisher_id = pub.id
            LEFT JOIN formats f ON b.format_id = f.id
            LEFT JOIN series s ON b.series_id = s.id
            WHERE b.name LIKE '%' || ? || '%'
                OR b.original_title LIKE '%' || ? || '%'
                OR pub.name LIKE '%' || ? || '%'
            ORDER BY b.created_at DESC
            LIMIT 100
            "#,
        )
        .bind(search_term)
        .bind(search_term)
        .bind(search_term)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as::<_, BookRow>(
            r#"
            SELECT
                b.id as book_id,
                b.name as book_name,
                b.original_title as book_original_title,
                pub.name as book_publisher,
                f.key as book_format_key,
                s.name as book_series,
                b.publication_date as book_publication_date,
                b.isbn as book_isbn,
                b.file_link as book_file_link
            FROM books b
            LEFT JOIN publishers pub ON b.publisher_id = pub.id
            LEFT JOIN formats f ON b.format_id = f.id
            LEFT JOIN series s ON b.series_id = s.id
            ORDER BY b.created_at DESC
            LIMIT 100
            "#,
        )
        .fetch_all(pool)
        .await?
    };

    if books.is_empty() {
        return Ok(vec![]);
    }

    let book_ids: Vec<i64> = books.iter().map(|b| b.book_id).collect();
    let placeholders = book_ids
        .iter()
        .map(|_| "?")
        .collect::<Vec<_>>()
        .join(",");

    // Query 2: Get all contents for these books
    #[derive(sqlx::FromRow)]
    struct ContentBookRow {
        book_id: i64,
        content_id: i64,
        content_name: String,
        content_original_title: Option<String>,
        content_type_key: Option<String>,
        content_publication_date: Option<i64>,
    }

    let contents_query = format!(
        r#"
        SELECT
            bc.book_id,
            c.id as content_id,
            c.name as content_name,
            c.original_title as content_original_title,
            t.key as content_type_key,
            c.publication_date as content_publication_date
        FROM x_books_contents bc
        JOIN contents c ON bc.content_id = c.id
        LEFT JOIN types t ON c.type_id = t.id
        WHERE bc.book_id IN ({})
        "#,
        placeholders
    );

    let mut query = sqlx::query_as::<_, ContentBookRow>(&contents_query);
    for book_id in &book_ids {
        query = query.bind(book_id);
    }

    let content_rows: Vec<ContentBookRow> = query.fetch_all(pool).await?;

    // Collect unique content IDs
    let content_ids: Vec<i64> = content_rows.iter().map(|c| c.content_id).collect();

    // Query 3: Get all people for these contents
    #[derive(sqlx::FromRow)]
    struct PersonContentRow {
        content_id: i64,
        person_id: i64,
        person_name: String,
        role_key: String,
    }

    let people_map = if !content_ids.is_empty() {
        let placeholders = content_ids
            .iter()
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(",");

        let people_query = format!(
            r#"
            SELECT
                cpr.content_id,
                p.id as person_id,
                p.name as person_name,
                r.key as role_key
            FROM x_contents_people_roles cpr
            JOIN people p ON cpr.person_id = p.id
            JOIN roles r ON cpr.role_id = r.id
            WHERE cpr.content_id IN ({})
            "#,
            placeholders
        );

        let mut query = sqlx::query_as::<_, PersonContentRow>(&people_query);
        for content_id in &content_ids {
            query = query.bind(content_id);
        }

        let person_rows: Vec<PersonContentRow> = query.fetch_all(pool).await?;

        // Build map: content_id -> Vec<PersonWithRole>
        let mut map: HashMap<i64, Vec<PersonWithRole>> = HashMap::new();
        for row in person_rows {
            map.entry(row.content_id).or_insert_with(Vec::new).push(PersonWithRole {
                person_id: row.person_id,
                person_name: row.person_name,
                role_key: row.role_key,
            });
        }
        map
    } else {
        HashMap::new()
    };

    // Build nested structure
    let mut books_map: HashMap<i64, BookWithDetails> = HashMap::new();

    // Initialize books
    for book in books {
        books_map.insert(
            book.book_id,
            BookWithDetails {
                book_id: book.book_id,
                book_name: book.book_name,
                book_original_title: book.book_original_title,
                book_publisher: book.book_publisher,
                book_format_key: book.book_format_key,
                book_series: book.book_series,
                book_publication_date: book.book_publication_date,
                book_isbn: book.book_isbn,
                book_file_link: book.book_file_link,
                contents: Vec::new(),
            },
        );
    }

    // Add contents to books
    for content_row in content_rows {
        if let Some(book_detail) = books_map.get_mut(&content_row.book_id) {
            let people = people_map.get(&content_row.content_id).cloned().unwrap_or_default();

            book_detail.contents.push(ContentWithPeople {
                content_id: content_row.content_id,
                content_name: content_row.content_name,
                content_original_title: content_row.content_original_title,
                content_type_key: content_row.content_type_key,
                content_publication_date: content_row.content_publication_date,
                people,
            });
        }
    }

    Ok(books_map.into_values().collect())
}

/// Get contents with their people and associated books
pub async fn get_contents_with_nested_data(
    pool: &SqlitePool,
    search: Option<&str>,
) -> Result<Vec<ContentWithDetails>, sqlx::Error> {
    // Query 1: Get contents
    let contents = if let Some(search_term) = search {
        sqlx::query_as::<_, ContentRow>(
            r#"
            SELECT DISTINCT
                c.id as content_id,
                c.name as content_name,
                c.original_title as content_original_title,
                t.key as content_type_key,
                c.publication_date as content_publication_date
            FROM contents c
            LEFT JOIN types t ON c.type_id = t.id
            WHERE c.name LIKE '%' || ? || '%'
                OR c.original_title LIKE '%' || ? || '%'
            ORDER BY c.created_at DESC
            LIMIT 100
            "#,
        )
        .bind(search_term)
        .bind(search_term)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as::<_, ContentRow>(
            r#"
            SELECT
                c.id as content_id,
                c.name as content_name,
                c.original_title as content_original_title,
                t.key as content_type_key,
                c.publication_date as content_publication_date
            FROM contents c
            LEFT JOIN types t ON c.type_id = t.id
            ORDER BY c.created_at DESC
            LIMIT 100
            "#,
        )
        .fetch_all(pool)
        .await?
    };

    if contents.is_empty() {
        return Ok(vec![]);
    }

    let content_ids: Vec<i64> = contents.iter().map(|c| c.content_id).collect();
    let placeholders = content_ids
        .iter()
        .map(|_| "?")
        .collect::<Vec<_>>()
        .join(",");

    // Query 2: Get all people for these contents
    #[derive(sqlx::FromRow)]
    struct PersonContentRow2 {
        content_id: i64,
        person_id: i64,
        person_name: String,
        role_key: String,
    }

    let people_query = format!(
        r#"
        SELECT
            cpr.content_id,
            p.id as person_id,
            p.name as person_name,
            r.key as role_key
        FROM x_contents_people_roles cpr
        JOIN people p ON cpr.person_id = p.id
        JOIN roles r ON cpr.role_id = r.id
        WHERE cpr.content_id IN ({})
        "#,
        placeholders
    );

    let mut query = sqlx::query_as::<_, PersonContentRow2>(&people_query);
    for content_id in &content_ids {
        query = query.bind(content_id);
    }

    let person_rows: Vec<PersonContentRow2> = query.fetch_all(pool).await?;

    // Build map: content_id -> Vec<PersonWithRole>
    let mut people_map: HashMap<i64, Vec<PersonWithRole>> = HashMap::new();
    for row in person_rows {
        people_map
            .entry(row.content_id)
            .or_insert_with(Vec::new)
            .push(PersonWithRole {
                person_id: row.person_id,
                person_name: row.person_name,
                role_key: row.role_key,
            });
    }

    // Query 3: Get all books for these contents
    #[derive(sqlx::FromRow)]
    struct BookContentRow {
        content_id: i64,
        book_id: i64,
        book_name: String,
        book_publisher: Option<String>,
        book_format_key: Option<String>,
        book_publication_date: Option<i64>,
    }

    let books_query = format!(
        r#"
        SELECT
            bc.content_id,
            b.id as book_id,
            b.name as book_name,
            pub.name as book_publisher,
            f.key as book_format_key,
            b.publication_date as book_publication_date
        FROM x_books_contents bc
        JOIN books b ON bc.book_id = b.id
        LEFT JOIN publishers pub ON b.publisher_id = pub.id
        LEFT JOIN formats f ON b.format_id = f.id
        WHERE bc.content_id IN ({})
        "#,
        placeholders
    );

    let mut query = sqlx::query_as::<_, BookContentRow>(&books_query);
    for content_id in &content_ids {
        query = query.bind(content_id);
    }

    let book_rows: Vec<BookContentRow> = query.fetch_all(pool).await?;

    // Build map: content_id -> Vec<BookBasicInfo>
    let mut books_map: HashMap<i64, Vec<BookBasicInfo>> = HashMap::new();
    for row in book_rows {
        books_map
            .entry(row.content_id)
            .or_insert_with(Vec::new)
            .push(BookBasicInfo {
                book_id: row.book_id,
                book_name: row.book_name,
                book_publisher: row.book_publisher,
                book_format_key: row.book_format_key,
                book_publication_date: row.book_publication_date,
            });
    }

    // Build final structure
    let result = contents
        .into_iter()
        .map(|content| {
            let people = people_map.get(&content.content_id).cloned().unwrap_or_default();
            let books = books_map.get(&content.content_id).cloned().unwrap_or_default();

            ContentWithDetails {
                content_id: content.content_id,
                content_name: content.content_name,
                content_original_title: content.content_original_title,
                content_type_key: content.content_type_key,
                content_publication_date: content.content_publication_date,
                people,
                books,
            }
        })
        .collect();

    Ok(result)
}
