use postgres::{Client, Row};
use serde_json::Value;
use uuid::Uuid;

use crate::error::StoreError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetRecord {
    pub id: Uuid,
    pub asset_kind: String,
    pub platform: String,
    pub project: String,
    pub channel: String,
    pub name: String,
    pub file_name: String,
    pub path: String,
    pub sha256: String,
    pub size_bytes: i64,
    pub source: String,
    pub metadata: Value,
}

pub struct NewAsset<'a> {
    pub id: Uuid,
    pub asset_kind: &'a str,
    pub platform: &'a str,
    pub project: &'a str,
    pub channel: &'a str,
    pub name: &'a str,
    pub file_name: &'a str,
    pub path: &'a str,
    pub sha256: &'a str,
    pub size_bytes: i64,
    pub source: &'a str,
    pub metadata: Value,
}

pub struct NewAssetDownload<'a> {
    pub id: Uuid,
    pub asset_id: Option<Uuid>,
    pub asset_kind: &'a str,
    pub project: &'a str,
    pub channel: &'a str,
    pub url: &'a str,
    pub result: &'a str,
    pub sha256: Option<&'a str>,
    pub size_bytes: Option<i64>,
    pub error: Option<&'a str>,
}

pub fn insert(client: &mut Client, asset: NewAsset<'_>) -> Result<(), StoreError> {
    client.execute(
        "insert into assets
         (id, asset_kind, platform, project, channel, name, file_name, path,
          sha256, size_bytes, source, metadata)
         values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
         on conflict (path) do nothing",
        &[
            &asset.id,
            &asset.asset_kind,
            &asset.platform,
            &asset.project,
            &asset.channel,
            &asset.name,
            &asset.file_name,
            &asset.path,
            &asset.sha256,
            &asset.size_bytes,
            &asset.source,
            &asset.metadata,
        ],
    )?;
    Ok(())
}

pub fn list(client: &mut Client) -> Result<Vec<AssetRecord>, StoreError> {
    let rows = client.query(
        "select id, asset_kind, platform, project, channel, name, file_name,
         path, sha256, size_bytes, source, metadata
         from assets order by created_at desc, name",
        &[],
    )?;
    Ok(rows.into_iter().map(record_from_row).collect())
}

pub fn get(client: &mut Client, id: Uuid) -> Result<Option<AssetRecord>, StoreError> {
    let row = client.query_opt(&select_sql("where id = $1"), &[&id])?;
    Ok(row.map(record_from_row))
}

pub fn get_by_path(client: &mut Client, path: &str) -> Result<Option<AssetRecord>, StoreError> {
    let row = client.query_opt(&select_sql("where path = $1"), &[&path])?;
    Ok(row.map(record_from_row))
}

pub fn latest_matching(
    client: &mut Client,
    asset_kind: &str,
    project: &str,
    channel: &str,
) -> Result<Option<AssetRecord>, StoreError> {
    let row = client.query_opt(
        &format!(
            "{} where asset_kind = $1 and project = $2 and channel = $3
             order by created_at desc, name limit 1",
            base_select()
        ),
        &[&asset_kind, &project, &channel],
    )?;
    Ok(row.map(record_from_row))
}

pub fn latest_for_project(
    client: &mut Client,
    asset_kind: &str,
    project: &str,
) -> Result<Option<AssetRecord>, StoreError> {
    let row = client.query_opt(
        &format!(
            "{} where asset_kind = $1 and project = $2
             order by created_at desc, name limit 1",
            base_select()
        ),
        &[&asset_kind, &project],
    )?;
    Ok(row.map(record_from_row))
}

pub fn insert_download(
    client: &mut Client,
    download: NewAssetDownload<'_>,
) -> Result<(), StoreError> {
    client.execute(
        "insert into asset_downloads
         (id, asset_id, asset_kind, project, channel, url, result, sha256,
          size_bytes, error)
         values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
        &[
            &download.id,
            &download.asset_id,
            &download.asset_kind,
            &download.project,
            &download.channel,
            &download.url,
            &download.result,
            &download.sha256,
            &download.size_bytes,
            &download.error,
        ],
    )?;
    Ok(())
}

fn select_sql(suffix: &str) -> String {
    format!("{} {}", base_select(), suffix)
}

fn base_select() -> &'static str {
    "select id, asset_kind, platform, project, channel, name, file_name,
     path, sha256, size_bytes, source, metadata from assets"
}

fn record_from_row(row: Row) -> AssetRecord {
    AssetRecord {
        id: row.get(0),
        asset_kind: row.get(1),
        platform: row.get(2),
        project: row.get(3),
        channel: row.get(4),
        name: row.get(5),
        file_name: row.get(6),
        path: row.get(7),
        sha256: row.get(8),
        size_bytes: row.get(9),
        source: row.get(10),
        metadata: row.get(11),
    }
}
