use postgres::{Client, Row};
use serde_json::Value;
use uuid::Uuid;

use crate::error::StoreError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginCatalogRecord {
    pub plugin_id: String,
    pub display_name: String,
    pub platforms: Value,
    pub default_policy: String,
    pub source_kind: String,
    pub source_project: String,
    pub required_plugin_ids: Value,
    pub metadata: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginInstallationRecord {
    pub instance_id: String,
    pub plugin_id: String,
    pub asset_id: Uuid,
    pub target_path: String,
    pub installed_sha256: String,
}

pub struct UpsertPluginCatalog<'a> {
    pub plugin_id: &'a str,
    pub display_name: &'a str,
    pub platforms: Value,
    pub default_policy: &'a str,
    pub source_kind: &'a str,
    pub source_project: &'a str,
    pub required_plugin_ids: Value,
    pub metadata: Value,
}

pub struct UpsertPluginInstallation<'a> {
    pub instance_id: &'a str,
    pub plugin_id: &'a str,
    pub asset_id: Uuid,
    pub target_path: &'a str,
    pub installed_sha256: &'a str,
}

pub fn upsert_catalog(
    client: &mut Client,
    entry: UpsertPluginCatalog<'_>,
) -> Result<(), StoreError> {
    client.execute(
        "insert into plugin_catalog_entries
         (plugin_id, display_name, platforms, default_policy, source_kind,
          source_project, required_plugin_ids, metadata)
         values ($1, $2, $3, $4, $5, $6, $7, $8)
         on conflict (plugin_id) do update set
         display_name = excluded.display_name,
         platforms = excluded.platforms,
         default_policy = excluded.default_policy,
         source_kind = excluded.source_kind,
         source_project = excluded.source_project,
         required_plugin_ids = excluded.required_plugin_ids,
         metadata = excluded.metadata,
         updated_at = now()",
        &[
            &entry.plugin_id,
            &entry.display_name,
            &entry.platforms,
            &entry.default_policy,
            &entry.source_kind,
            &entry.source_project,
            &entry.required_plugin_ids,
            &entry.metadata,
        ],
    )?;
    Ok(())
}

pub fn list_catalog(client: &mut Client) -> Result<Vec<PluginCatalogRecord>, StoreError> {
    let rows = client.query(
        "select plugin_id, display_name, platforms, default_policy, source_kind,
         source_project, required_plugin_ids, metadata
         from plugin_catalog_entries order by plugin_id",
        &[],
    )?;
    Ok(rows.into_iter().map(catalog_from_row).collect())
}

pub fn upsert_installation(
    client: &mut Client,
    install: UpsertPluginInstallation<'_>,
) -> Result<(), StoreError> {
    client.execute(
        "insert into instance_plugin_installations
         (instance_id, plugin_id, asset_id, target_path, installed_sha256)
         values ($1, $2, $3, $4, $5)
         on conflict (instance_id, plugin_id) do update set
         asset_id = excluded.asset_id,
         target_path = excluded.target_path,
         installed_sha256 = excluded.installed_sha256,
         installed_at = now()",
        &[
            &install.instance_id,
            &install.plugin_id,
            &install.asset_id,
            &install.target_path,
            &install.installed_sha256,
        ],
    )?;
    Ok(())
}

pub fn list_installations(
    client: &mut Client,
    instance_id: &str,
) -> Result<Vec<PluginInstallationRecord>, StoreError> {
    let rows = client.query(
        "select instance_id, plugin_id, asset_id, target_path, installed_sha256
         from instance_plugin_installations where instance_id = $1 order by plugin_id",
        &[&instance_id],
    )?;
    Ok(rows.into_iter().map(installation_from_row).collect())
}

fn catalog_from_row(row: Row) -> PluginCatalogRecord {
    PluginCatalogRecord {
        plugin_id: row.get(0),
        display_name: row.get(1),
        platforms: row.get(2),
        default_policy: row.get(3),
        source_kind: row.get(4),
        source_project: row.get(5),
        required_plugin_ids: row.get(6),
        metadata: row.get(7),
    }
}

fn installation_from_row(row: Row) -> PluginInstallationRecord {
    PluginInstallationRecord {
        instance_id: row.get(0),
        plugin_id: row.get(1),
        asset_id: row.get(2),
        target_path: row.get(3),
        installed_sha256: row.get(4),
    }
}
