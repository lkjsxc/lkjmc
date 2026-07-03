use postgres::Client;
use serde_json::Value;
use uuid::Uuid;

use crate::support::instance_helpers::store;

pub(super) struct JarResolution {
    pub(super) asset_id: Option<Uuid>,
    pub(super) attempted_queries: Vec<String>,
    pub(super) missing_explicit: Option<Uuid>,
}

pub(super) fn jar_asset(
    client: &mut Client,
    body: &Value,
    kind: &str,
    template: &str,
) -> Result<JarResolution, String> {
    if let Some(asset_id) = body.get("jarAssetId").and_then(Value::as_str) {
        let asset_id = Uuid::parse_str(asset_id).map_err(|error| error.to_string())?;
        return Ok(match store(lkjmc_store::jar::get(client, asset_id))? {
            Some(_) => resolution(Some(asset_id), Vec::new(), None),
            None => resolution(None, Vec::new(), Some(asset_id)),
        });
    }
    if body.get("command").and_then(Value::as_str).is_some() {
        return Ok(resolution(None, Vec::new(), None));
    }
    default_asset(client, kind, template)
}

fn default_asset(client: &mut Client, kind: &str, template: &str) -> Result<JarResolution, String> {
    let queries = asset_queries(kind, template);
    for query in &queries {
        if let Some(asset) = store(lkjmc_store::jar::latest_matching(client, query))? {
            return Ok(resolution(Some(asset.id), queries, None));
        }
    }
    Ok(resolution(None, queries, None))
}

fn resolution(
    asset_id: Option<Uuid>,
    attempted_queries: Vec<String>,
    missing_explicit: Option<Uuid>,
) -> JarResolution {
    JarResolution {
        asset_id,
        attempted_queries,
        missing_explicit,
    }
}

fn asset_queries(kind: &str, template: &str) -> Vec<String> {
    let mut queries = vec![kind.to_string(), template.to_string()];
    if let Some(prefix) = template.split('-').next() {
        if !queries.iter().any(|value| value == prefix) {
            queries.push(prefix.to_string());
        }
    }
    queries
}
