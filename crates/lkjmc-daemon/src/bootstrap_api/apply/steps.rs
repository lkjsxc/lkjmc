use lkjmc_core::bootstrap::BootstrapEffect;
use uuid::Uuid;

pub fn record(
    client: &mut postgres::Client,
    run_id: Uuid,
    index: usize,
    effect: &BootstrapEffect,
    result: &Result<(), String>,
) -> Result<(), String> {
    let diagnostic = result.as_ref().err().map(String::as_str);
    lkjmc_store::bootstrap::insert_step(
        client,
        lkjmc_store::bootstrap::NewBootstrapStep {
            id: Uuid::new_v4(),
            run_id,
            step_order: i32::try_from(index + 1).map_err(|error| error.to_string())?,
            effect_kind: effect_kind(effect),
            target: effect_target(effect),
            result: if result.is_ok() {
                "succeeded"
            } else {
                "failed"
            },
            diagnostic,
        },
    )
    .map_err(|error| error.to_string())
}

fn effect_kind(effect: &BootstrapEffect) -> &'static str {
    match effect {
        BootstrapEffect::SyncServerAsset { .. } => "asset.server.sync",
        BootstrapEffect::RegisterLocalPlugin { .. } => "asset.plugin.local",
        BootstrapEffect::SyncPluginAsset { .. } => "asset.plugin.sync",
        BootstrapEffect::ReconcileInstance { .. } => "instance.reconcile",
        BootstrapEffect::RenderInstance { .. } => "template.render",
        BootstrapEffect::InstallPlugin { .. } => "plugin.install",
        BootstrapEffect::StartInstance { .. } => "instance.start",
        BootstrapEffect::RestartInstance { .. } => "instance.restart",
        BootstrapEffect::WaitForReadiness { .. } => "probe.wait",
        BootstrapEffect::GenerateDaemonHttpToken { .. } => "secret.daemon-http",
        BootstrapEffect::GenerateForwardingSecret { .. } => "secret.forwarding",
        _ => "bootstrap.effect",
    }
}

fn effect_target(effect: &BootstrapEffect) -> &str {
    match effect {
        BootstrapEffect::SyncServerAsset { project } => match project {
            lkjmc_core::bootstrap::ServerProject::Paper => "paper",
            lkjmc_core::bootstrap::ServerProject::Folia => "folia",
            lkjmc_core::bootstrap::ServerProject::Purpur => "purpur",
            lkjmc_core::bootstrap::ServerProject::Velocity => "velocity",
        },
        BootstrapEffect::RegisterLocalPlugin { plugin }
        | BootstrapEffect::SyncPluginAsset { plugin } => plugin.as_str(),
        BootstrapEffect::ReconcileInstance { id, .. }
        | BootstrapEffect::RenderInstance { id }
        | BootstrapEffect::InstallPlugin { id, .. }
        | BootstrapEffect::StartInstance { id }
        | BootstrapEffect::RestartInstance { id }
        | BootstrapEffect::WaitForReadiness { id } => id.as_str(),
        BootstrapEffect::GenerateDaemonHttpToken { .. } => "daemon-http-token",
        BootstrapEffect::GenerateForwardingSecret { .. } => "forwarding-secret",
        _ => "bootstrap",
    }
}
