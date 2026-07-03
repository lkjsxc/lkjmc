#[allow(dead_code)]
mod support;

use lkjmc_store::{asset, bootstrap, instance, migrate, plugin, pool};
use serde_json::json;
use std::env;
use uuid::Uuid;

const TEST_SHA: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

#[test]
fn assets_plugins_and_bootstrap_round_trip() -> Result<(), lkjmc_store::error::StoreError> {
    let database_url = match env::var("LKJMC_STORE_TEST_DATABASE_URL") {
        Ok(value) => value,
        Err(_) => return Ok(()),
    };
    let mut client = pool::connect(&database_url)?;
    let _schema = support::prepare_isolated_schema(&mut client)?;
    migrate::apply(&mut client)?;
    let asset_id = Uuid::new_v4();
    asset::insert(&mut client, new_asset(asset_id))?;
    let stored = asset::get(&mut client, asset_id)?
        .ok_or_else(|| lkjmc_store::error::StoreError::invalid_state("asset missing"))?;
    assert_eq!(stored.sha256, TEST_SHA);
    assert!(asset::get_by_path(&mut client, stored.path.as_str())?.is_some());
    assert!(asset::latest_matching(&mut client, "plugin", "lkjmc-paper", "stable")?.is_some());
    asset::insert_download(
        &mut client,
        asset::NewAssetDownload {
            id: Uuid::new_v4(),
            asset_id: Some(asset_id),
            asset_kind: "plugin",
            project: "lkjmc-paper",
            channel: "stable",
            url: "file:///build/lkjmc-paper.jar",
            result: "succeeded",
            sha256: Some(TEST_SHA),
            size_bytes: Some(12),
            error: None,
        },
    )?;
    plugin::upsert_catalog(&mut client, new_catalog())?;
    assert_eq!(plugin::list_catalog(&mut client)?.len(), 1);
    instance::insert(&mut client, "hub", None, "paper", "running", &json!({}))?;
    plugin::upsert_installation(
        &mut client,
        plugin::UpsertPluginInstallation {
            instance_id: "hub",
            plugin_id: "lkjmc-paper",
            asset_id,
            target_path: "/var/lib/lkjmc/instances/hub/plugins/lkjmc-paper.jar",
            installed_sha256: TEST_SHA,
        },
    )?;
    assert_eq!(plugin::list_installations(&mut client, "hub")?.len(), 1);
    let run_id = Uuid::new_v4();
    bootstrap::create_run(
        &mut client,
        bootstrap::NewBootstrapRun {
            id: run_id,
            profile: "playable",
            requested_by: "test",
            result: "running",
            diagnostics: json!([]),
        },
    )?;
    bootstrap::insert_step(
        &mut client,
        bootstrap::NewBootstrapStep {
            id: Uuid::new_v4(),
            run_id,
            step_order: 1,
            effect_kind: "asset.plugin.sync",
            target: "lkjmc-paper",
            result: "succeeded",
            diagnostic: None,
        },
    )?;
    bootstrap::finish_run(&mut client, run_id, "succeeded", json!([]))?;
    let run = bootstrap::get_run(&mut client, run_id)?
        .ok_or_else(|| lkjmc_store::error::StoreError::invalid_state("run missing"))?;
    assert_eq!(run.result, "succeeded");
    assert_eq!(bootstrap::steps_for_run(&mut client, run_id)?.len(), 1);
    Ok(())
}

fn new_asset(id: Uuid) -> asset::NewAsset<'static> {
    asset::NewAsset {
        id,
        asset_kind: "plugin",
        platform: "paper",
        project: "lkjmc-paper",
        channel: "stable",
        name: "lkjmc-paper",
        file_name: "lkjmc-paper.jar",
        path: "/opt/lkjmc/assets/plugin/lkjmc/paper/test-lkjmc-paper.jar",
        sha256: TEST_SHA,
        size_bytes: 12,
        source: "gradle-shadowJar",
        metadata: json!({}),
    }
}

fn new_catalog() -> plugin::UpsertPluginCatalog<'static> {
    plugin::UpsertPluginCatalog {
        plugin_id: "lkjmc-paper",
        display_name: "lkjmc Paper",
        platforms: json!(["paper", "folia"]),
        default_policy: "required",
        source_kind: "local",
        source_project: "lkjmc",
        required_plugin_ids: json!([]),
        metadata: json!({}),
    }
}
