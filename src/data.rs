use libsql::{de, Connection};
use serde::de::DeserializeOwned;

pub async fn get_one<T: DeserializeOwned>(
    conn: Connection,
    sql: &str,
    params: impl libsql::params::IntoParams,
) -> anyhow::Result<T> {
    let row = conn
        .query(sql, params)
        .await?
        .next()
        .await?
        .ok_or(anyhow::anyhow!("No clue"))?;

    de::from_row::<T>(&row).map_err(|e| anyhow::anyhow!(e))
}

pub async fn get_list<T: DeserializeOwned>(
    conn: Connection,
    sql: &str,
    params: impl libsql::params::IntoParams,
) -> anyhow::Result<Vec<T>> {
    let mut iter = conn.query(sql, params).await?;
    let mut ret: Vec<T> = Vec::new();

    while let Some(page) = iter.next().await? {
        ret.push(de::from_row::<T>(&page)?);
    }

    Ok(ret)
}
