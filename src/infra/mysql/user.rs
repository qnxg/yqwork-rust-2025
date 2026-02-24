use sqlx::Row;

use super::get_db_pool;
use crate::{result::AppResult, utils};

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: u32,
    pub last_login: Option<chrono::NaiveDateTime>,
    pub info: UserBasicInfo,
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserBasicInfo {
    pub username: Option<String>,
    pub name: String,
    pub stu_id: String,
    pub email: Option<String>,
    pub xueyuan: u32,
    pub gangwei: Option<String>,
    pub zaiku: bool,
    pub qingonggang: bool,
    pub status: UserStatus,
    pub department_id: u32,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserStatus {
    Unknown,
    Intern,
    Formal,
    Retaired,
}
impl From<u32> for UserStatus {
    fn from(value: u32) -> Self {
        match value {
            0 => UserStatus::Unknown,
            1 => UserStatus::Intern,
            2 => UserStatus::Formal,
            3 => UserStatus::Retaired,
            _ => UserStatus::Unknown,
        }
    }
}
impl From<UserStatus> for u32 {
    fn from(value: UserStatus) -> Self {
        match value {
            UserStatus::Unknown => 0,
            UserStatus::Intern => 1,
            UserStatus::Formal => 2,
            UserStatus::Retaired => 3,
        }
    }
}
impl serde::Serialize for UserStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let v = u32::from(*self);
        serializer.serialize_u32(v)
    }
}

pub async fn get_user_list(
    page: u32,
    page_size: u32,
    stu_id: Option<&str>,
    name: Option<&str>,
    department_id: Option<u32>,
    status: Option<u32>,
) -> AppResult<(u32, Vec<User>)> {
    let mut main_query = sqlx::QueryBuilder::new(
        r#"
        SELECT id, username, name, stuId, email, xueyuan, gangwei, zaiku, qingonggang, status, lastLogin, departmentId
        FROM yqwork_new.users
        WHERE deletedAt IS NULL
        "#,
    );
    let mut count_query = sqlx::QueryBuilder::new(
        r#"
        SELECT COUNT(*) as count
        FROM yqwork_new.users
        WHERE deletedAt IS NULL
        "#,
    );
    if let Some(stu_id) = stu_id {
        main_query
            .push(" AND stuId LIKE ")
            .push_bind(format!("%{}%", stu_id));
        count_query
            .push(" AND stuId LIKE ")
            .push_bind(format!("%{}%", stu_id));
    }
    if let Some(name) = name {
        main_query
            .push(" AND name LIKE ")
            .push_bind(format!("%{}%", name));
        count_query
            .push(" AND name LIKE ")
            .push_bind(format!("%{}%", name));
    }
    if let Some(department_id) = department_id {
        main_query
            .push(" AND departmentId = ")
            .push_bind(department_id);
        count_query
            .push(" AND departmentId = ")
            .push_bind(department_id);
    }
    if let Some(status) = status {
        main_query.push(" AND status = ").push_bind(status);
        count_query.push(" AND status = ").push_bind(status);
    }
    main_query.push(" ORDER BY id DESC");
    main_query.push(" LIMIT ").push_bind(page_size);
    main_query
        .push(" OFFSET ")
        .push_bind((page - 1) * page_size);
    let res = main_query
        .build()
        .fetch_all(get_db_pool().await)
        .await?
        .into_iter()
        .map(|r| User {
            id: r.get("id"),
            info: UserBasicInfo {
                username: r.get("username"),
                name: r.get("name"),
                stu_id: r.get("stuId"),
                email: r.get("email"),
                xueyuan: r.get("xueyuan"),
                gangwei: r.get("gangwei"),
                zaiku: r.get::<u8, _>("zaiku") != 0,
                qingonggang: r.get::<u8, _>("qingonggang") != 0,
                status: UserStatus::from(r.get::<u32, _>("status")),
                department_id: r.get("departmentId"),
            },
            last_login: r.get("lastLogin"),
        })
        .collect::<Vec<_>>();
    let count: i64 = count_query
        .build_query_scalar()
        .fetch_one(get_db_pool().await)
        .await?;
    Ok((count as u32, res))
}

pub async fn get_user(user_id: u32) -> AppResult<Option<User>> {
    let res = sqlx::query!(
        r#"
        SELECT id, username, name, stuId, email, xueyuan, gangwei, zaiku, qingonggang, status, lastLogin, departmentId
        FROM yqwork_new.users
        WHERE id = ? AND deletedAt IS NULL
        "#,
        user_id
    )
    .fetch_optional(get_db_pool().await)
    .await?
    .map(|r| User {
        id: r.id,
        info: UserBasicInfo {
            username: r.username,
            name: r.name,
            stu_id: r.stuId,
            email: r.email,
            xueyuan: r.xueyuan,
            gangwei: r.gangwei,
            zaiku: r.zaiku != 0,
            qingonggang: r.qingonggang != 0,
            status: UserStatus::from(r.status),
            department_id: r.departmentId,
        },
        last_login: r.lastLogin,
    });
    Ok(res)
}

pub async fn add_user(info: &UserBasicInfo, password: &str) -> AppResult<u32> {
    let now = utils::now_time();
    let res = sqlx::query!(
        r#"
        INSERT INTO yqwork_new.users (username, name, stuId, email, xueyuan, gangwei, zaiku, qingonggang, status, departmentId, password, createdAt, updatedAt)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        info.username,
        info.name,
        info.stu_id,
        info.email,
        info.xueyuan,
        info.gangwei,
        info.zaiku,
        info.qingonggang,
        u32::from(info.status),
        info.department_id,
        password,
        now,
        now
    ).execute(get_db_pool().await).await?;
    Ok(res.last_insert_id() as u32)
}

pub async fn update_user(user_id: u32, info: &UserBasicInfo) -> AppResult<()> {
    let now = utils::now_time();
    sqlx::query!(
        r#"
        UPDATE yqwork_new.users
        SET name = ?, stuId = ?, email = ?, xueyuan = ?, gangwei = ?, zaiku = ?, qingonggang = ?, status = ?, departmentId = ?, updatedAt = ?, username = ?
        WHERE id = ? AND deletedAt IS NULL
        "#,
        info.name,
        info.stu_id,
        info.email,
        info.xueyuan,
        info.gangwei,
        info.zaiku,
        info.qingonggang,
        u32::from(info.status),
        info.department_id,
        now,
        info.username,
        user_id
    )
    .execute(get_db_pool().await)
    .await?;
    Ok(())
}

pub async fn delete_user(user_id: u32) -> AppResult<()> {
    let now = utils::now_time();
    sqlx::query!(
        r#"
        UPDATE yqwork_new.users
        SET deletedAt = ?
        WHERE id = ? AND deletedAt IS NULL
        "#,
        now,
        user_id
    )
    .execute(get_db_pool().await)
    .await?;
    Ok(())
}

// pub async fn update_user_department(user_id: u32, department_id: u32) -> AppResult<()> {
//     let now = utils::now_time();
//     sqlx::query!(
//         r#"
//         UPDATE yqwork_new.users
//         SET departmentId = ?, updatedAt = ?
//         WHERE id = ?
//         "#,
//         department_id,
//         now,
//         user_id
//     )
//     .execute(get_db_pool().await)
//     .await?;
//     Ok(())
// }

pub async fn get_user_by_stu_id(stu_id: &str) -> AppResult<Option<User>> {
    let res = sqlx::query!(
        r#"
        SELECT id, username, name, stuId, email, xueyuan, gangwei, zaiku, qingonggang, status, lastLogin, departmentId
        FROM yqwork_new.users
        WHERE stuId = ? AND deletedAt IS NULL
        "#,
        stu_id
    )
    .fetch_optional(get_db_pool().await)
    .await?
    .map(|r| User {
        id: r.id,
        info: UserBasicInfo {
            username: r.username,
            name: r.name,
            stu_id: r.stuId,
            email: r.email,
            xueyuan: r.xueyuan,
            gangwei: r.gangwei,
            zaiku: r.zaiku != 0,
            qingonggang: r.qingonggang != 0,
            status: UserStatus::from(r.status),
            department_id: r.departmentId,
        },
        last_login: r.lastLogin,
    });
    Ok(res)
}

pub async fn get_user_password(user_id: u32) -> AppResult<Option<String>> {
    let res = sqlx::query!(
        r#"
        SELECT password
        FROM yqwork_new.users
        WHERE id = ? AND deletedAt IS NULL
        "#,
        user_id
    )
    .fetch_optional(get_db_pool().await)
    .await?
    .map(|r| r.password);
    Ok(res)
}

pub async fn update_user_password(user_id: u32, password: &str) -> AppResult<()> {
    let now = utils::now_time();
    sqlx::query!(
        r#"
        UPDATE yqwork_new.users
        SET password = ?, updatedAt = ?
        WHERE id = ?
        "#,
        password,
        now,
        user_id
    )
    .execute(get_db_pool().await)
    .await?;
    Ok(())
}

pub async fn update_user_last_login(user_id: u32) -> AppResult<()> {
    let now = utils::now_time();
    sqlx::query!(
        r#"
        UPDATE yqwork_new.users
        SET lastLogin = ?
        WHERE id = ?
        "#,
        now,
        user_id
    )
    .execute(get_db_pool().await)
    .await?;
    Ok(())
}
