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
    pub path: String,
    pub sha256: String,
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

pub fn get(client: &mut Client, id: Uuid) -> Result<Option<JarAssetRecord>, StoreError> {
    let row = client.query_opt(
        "select id, kind, project, channel, path, sha256 from jar_assets where id = $1",
        &[&id],
    )?;
    Ok(row.map(|row| JarAssetRecord {
        id: row.get(0),
        kind: row.get(1),
        project: row.get(2),
        channel: row.get(3),
        path: row.get(4),
        sha256: row.get(5),
    }))
}
