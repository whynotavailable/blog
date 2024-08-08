use libsql::{de, Connection};
use serde::de::DeserializeOwned;

use crate::errors::{AppError, AppResult};

pub async fn get_one<T: DeserializeOwned>(
    conn: Connection,
    sql: &str,
    params: impl libsql::params::IntoParams,
) -> AppResult<T> {
    let row = conn
        .query(sql, params)
        .await?
        .next()
        .await?
        .ok_or(AppError::not_found("Row not found"))?;

    de::from_row::<T>(&row).map_err(AppError::from)
}

pub async fn get_list<T: DeserializeOwned>(
    conn: Connection,
    sql: &str,
    params: impl libsql::params::IntoParams,
) -> AppResult<Vec<T>> {
    let mut iter = conn.query(sql, params).await?;
    let mut ret: Vec<T> = Vec::new();

    while let Some(page) = iter.next().await? {
        ret.push(de::from_row::<T>(&page)?);
    }

    Ok(ret)
}
