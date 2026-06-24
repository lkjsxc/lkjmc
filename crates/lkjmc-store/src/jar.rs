use postgres::Client;
use serde_json::Value;
use uuid::Uuid;

use crate::error::StoreError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JarAssetRecord {
    pub id: Uuid,
    pub kind: String,
    pub project: String,
    pub channel: String,
    pub name: String,
    pub path: String,
    pub sha256: String,
    pub size_bytes: i64,
    pub source: String,
}

pub struct NewJarAsset<'a> {
    pub id: Uuid,
    pub kind: &'a str,
    pub project: &'a str,
    pub channel: &'a str,
    pub name: &'a str,
    pub path: &'a str,
    pub sha256: &'a str,
    pub size_bytes: i64,
    pub source: &'a str,
}

pub struct NewJarDownload<'a> {
    pub id: Uuid,
    pub jar_asset_id: Option<Uuid>,
    pub project: &'a str,
    pub channel: &'a str,
    pub url: &'a str,
    pub result: &'a str,
    pub sha256: Option<&'a str>,
    pub size_bytes: Option<i64>,
}

pub fn insert(client: &mut Client, asset: NewJarAsset<'_>) -> Result<(), StoreError> {
    let metadata = Value::Object(Default::default());
    client.execute(
        "insert into jar_assets
         (id, kind, project, channel, name, path, sha256, size_bytes, source, metadata)
         values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
        &[
            &asset.id,
            &asset.kind,
            &asset.project,
            &asset.channel,
            &asset.name,
            &asset.path,
            &asset.sha256,
            &asset.size_bytes,
            &asset.source,
            &metadata,
        ],
    )?;
    Ok(())
}

pub fn list(client: &mut Client) -> Result<Vec<JarAssetRecord>, StoreError> {
    let rows = client.query(
        "select id, kind, project, channel, name, path, sha256, size_bytes, source
         from jar_assets order by created_at desc, name",
        &[],
    )?;
    Ok(rows.into_iter().map(record_from_row).collect())
}

pub fn get(client: &mut Client, id: Uuid) -> Result<Option<JarAssetRecord>, StoreError> {
    let row = client.query_opt(
        "select id, kind, project, channel, name, path, sha256, size_bytes, source
         from jar_assets where id = $1",
        &[&id],
    )?;
    Ok(row.map(record_from_row))
}

pub fn get_by_path(client: &mut Client, path: &str) -> Result<Option<JarAssetRecord>, StoreError> {
    let row = client.query_opt(
        "select id, kind, project, channel, name, path, sha256, size_bytes, source
         from jar_assets where path = $1",
        &[&path],
    )?;
    Ok(row.map(record_from_row))
}

pub fn insert_download(
    client: &mut Client,
    download: NewJarDownload<'_>,
) -> Result<(), StoreError> {
    let metadata = Value::Object(Default::default());
    client.execute(
        "insert into jar_downloads
         (id, jar_asset_id, project, channel, url, result, sha256, size_bytes, metadata)
         values ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        &[
            &download.id,
            &download.jar_asset_id,
            &download.project,
            &download.channel,
            &download.url,
            &download.result,
            &download.sha256,
            &download.size_bytes,
            &metadata,
        ],
    )?;
    Ok(())
}

pub fn prunable(client: &mut Client) -> Result<Vec<JarAssetRecord>, StoreError> {
    let rows = client.query(
        "select id, kind, project, channel, name, path, sha256, size_bytes, source
         from jar_assets a
         where not exists (select 1 from instances i where i.jar_asset_id = a.id)
         order by created_at, name",
        &[],
    )?;
    Ok(rows.into_iter().map(record_from_row).collect())
}

pub fn delete(client: &mut Client, id: Uuid) -> Result<u64, StoreError> {
    Ok(client.execute("delete from jar_assets where id = $1", &[&id])?)
}

pub fn latest_matching(
    client: &mut Client,
    query: &str,
) -> Result<Option<JarAssetRecord>, StoreError> {
    let row = client.query_opt(
        "select id, kind, project, channel, name, path, sha256, size_bytes, source
         from jar_assets where kind = $1 or project = $1
         order by created_at desc, name limit 1",
        &[&query],
    )?;
    Ok(row.map(record_from_row))
}

fn record_from_row(row: postgres::Row) -> JarAssetRecord {
    JarAssetRecord {
        id: row.get(0),
        kind: row.get(1),
        project: row.get(2),
        channel: row.get(3),
        name: row.get(4),
        path: row.get(5),
        sha256: row.get(6),
        size_bytes: row.get(7),
        source: row.get(8),
    }
}
