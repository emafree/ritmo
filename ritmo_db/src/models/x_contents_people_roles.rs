use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, PartialEq, Eq)]
pub struct ContentPersonRole {
    pub content_id: i64,
    pub person_id: i64,
    pub role_id: i64,
}

impl ContentPersonRole {
    pub async fn create(
        pool: &sqlx::SqlitePool,
        new_link: &ContentPersonRole,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO x_contents_people_roles (content_id, person_id, role_id) VALUES (?, ?, ?)",
            new_link.content_id,
            new_link.person_id,
            new_link.role_id
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn delete(
        pool: &sqlx::SqlitePool,
        content_id: i64,
        person_id: i64,
        role_id: i64,
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx::query!(
            "DELETE FROM x_contents_people_roles WHERE content_id = ? AND person_id = ? AND role_id = ?",
            content_id,
            person_id,
            role_id
        )
        .execute(pool)
        .await?;
        Ok(result.rows_affected())
    }

    pub async fn list_by_content(
        pool: &sqlx::SqlitePool,
        content_id: i64,
    ) -> Result<Vec<ContentPersonRole>, sqlx::Error> {
        let links = sqlx::query_as!(
            ContentPersonRole,
            "SELECT * FROM x_contents_people_roles WHERE content_id = ?",
            content_id
        )
        .fetch_all(pool)
        .await?;
        Ok(links)
    }

    pub async fn list_by_person(
        pool: &sqlx::SqlitePool,
        person_id: i64,
    ) -> Result<Vec<ContentPersonRole>, sqlx::Error> {
        let links = sqlx::query_as!(
            ContentPersonRole,
            "SELECT * FROM x_contents_people_roles WHERE person_id = ?",
            person_id
        )
        .fetch_all(pool)
        .await?;
        Ok(links)
    }

    pub async fn list_by_role(
        pool: &sqlx::SqlitePool,
        role_id: i64,
    ) -> Result<Vec<ContentPersonRole>, sqlx::Error> {
        let links = sqlx::query_as!(
            ContentPersonRole,
            "SELECT * FROM x_contents_people_roles WHERE role_id = ?",
            role_id
        )
        .fetch_all(pool)
        .await?;
        Ok(links)
    }
}
