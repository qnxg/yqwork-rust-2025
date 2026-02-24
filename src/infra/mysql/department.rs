use super::get_db_pool;
use crate::{result::AppResult, utils};

#[derive(serde::Serialize, Debug)]
pub struct Department {
    pub id: u32,
    pub name: String,
    pub desc: String,
}

pub async fn get_department_list() -> AppResult<Vec<Department>> {
    let departments = sqlx::query!(
        r#"
        SELECT id, name, `desc`
        FROM yqwork_new.departments
        WHERE deletedAt IS NULL
        "#,
    )
    .fetch_all(get_db_pool().await)
    .await?
    .into_iter()
    .map(|r| Department {
        id: r.id,
        name: r.name,
        desc: r.desc,
    })
    .collect::<Vec<_>>();

    Ok(departments)
}

pub async fn add_department(name: &str, desc: &str) -> AppResult<u32> {
    let now = utils::now_time();
    let res = sqlx::query!(
        r#"
        INSERT INTO yqwork_new.departments (name, `desc`, createdAt, updatedAt)
        VALUES (?, ?, ?, ?)
        "#,
        name,
        desc,
        now,
        now
    )
    .execute(get_db_pool().await)
    .await?;
    Ok(res.last_insert_id() as u32)
}

pub async fn update_department(id: u32, name: &str, desc: &str) -> AppResult<()> {
    let now = utils::now_time();
    sqlx::query!(
        r#"
        UPDATE yqwork_new.departments
        SET name = ?, `desc` = ?, updatedAt = ?
        WHERE id = ? AND deletedAt IS NULL
        "#,
        name,
        desc,
        now,
        id
    )
    .execute(get_db_pool().await)
    .await?;
    Ok(())
}

pub async fn delete_department(id: u32) -> AppResult<()> {
    let now = utils::now_time();
    sqlx::query!(
        r#"
        UPDATE yqwork_new.departments
        SET deletedAt = ?
        WHERE id = ? AND deletedAt IS NULL
        "#,
        now,
        id
    )
    .execute(get_db_pool().await)
    .await?;
    Ok(())
}
