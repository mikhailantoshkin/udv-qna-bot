use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::collections::HashSet;

#[derive(Serialize, Deserialize)]
pub struct Category {
    pub id: i64,
    pub name: String,
}

pub async fn get_category(pool: &SqlitePool, id: i64) -> sqlx::Result<Category> {
    sqlx::query_as!(
        Category,
        r#"
        SELECT * FROM categories WHERE categories.id = ?1
        "#,
        id
    )
    .fetch_one(pool)
    .await
}

pub async fn create_category(pool: &SqlitePool, name: &str) -> sqlx::Result<i64> {
    let mut conn = pool.acquire().await?;

    let id = sqlx::query!(
        r#"
INSERT INTO categories (name) VALUES (?1)
        "#,
        name
    )
    .execute(&mut conn)
    .await?
    .last_insert_rowid();

    Ok(id)
}

pub async fn update_category(pool: &SqlitePool, category: Category) -> sqlx::Result<()> {
    let mut conn = pool.acquire().await?;

    sqlx::query!(
        r#"
        UPDATE categories SET name=?1 WHERE categories.id = ?2
        "#,
        category.name,
        category.id
    )
    .execute(&mut conn)
    .await?;
    Ok(())
}

pub async fn get_categories(pool: &SqlitePool) -> sqlx::Result<Vec<Category>> {
    sqlx::query_as!(
        Category,
        r#"
SELECT id, name
FROM categories
ORDER BY id
        "#
    )
    .fetch_all(pool)
    .await
}

pub async fn delete_category(pool: &SqlitePool, category_id: i64) -> sqlx::Result<()> {
    let mut conn = pool.acquire().await?;

    sqlx::query!(
        r#"
        DELETE FROM categories WHERE categories.id = ?1
        "#,
        category_id,
    )
    .execute(&mut conn)
    .await?;
    Ok(())
}

pub async fn import_categories(pool: &SqlitePool, categories: Vec<Category>) -> sqlx::Result<()> {
    let existing_categories = get_categories(pool).await?;
    let existing_categories_ids: HashSet<i64> = existing_categories.iter().map(|c| c.id).collect();
    let new_categories_ids: HashSet<i64> = categories.iter().map(|c| c.id).collect();
    for category_id in existing_categories_ids.difference(&new_categories_ids) {
        delete_category(pool, *category_id).await?;
    }
    for category in categories {
        if existing_categories_ids.contains(&category.id) {
            update_category(pool, category).await?;
        } else {
            create_category(pool, category.name.as_str()).await?;
        }
    }
    Ok(())
}
