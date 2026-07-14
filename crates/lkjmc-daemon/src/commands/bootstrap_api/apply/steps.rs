use uuid::Uuid;

use super::network_plan::NetworkEffect;

pub fn record(
    client: &mut postgres::Client,
    run_id: Uuid,
    index: usize,
    effect: &NetworkEffect,
    result: &Result<(), String>,
) -> Result<(), String> {
    insert(
        client,
        Uuid::new_v4(),
        run_id,
        index,
        effect,
        terminal(result),
        result.as_ref().err().map(String::as_str),
    )
}

pub fn start(
    client: &mut postgres::Client,
    run_id: Uuid,
    index: usize,
    effect: &NetworkEffect,
) -> Result<Uuid, String> {
    let id = Uuid::new_v4();
    insert(client, id, run_id, index, effect, "running", None)?;
    Ok(id)
}

fn insert(
    client: &mut postgres::Client,
    id: Uuid,
    run_id: Uuid,
    index: usize,
    effect: &NetworkEffect,
    result: &str,
    diagnostic: Option<&str>,
) -> Result<(), String> {
    lkjmc_store::bootstrap::insert_step(
        client,
        lkjmc_store::bootstrap::NewBootstrapStep {
            id,
            run_id,
            step_order: i32::try_from(index + 1).map_err(|error| error.to_string())?,
            effect_kind: effect_kind(effect),
            target: effect_target(effect),
            result,
            diagnostic,
        },
    )
    .map_err(|error| error.to_string())
}

pub fn complete(
    client: &mut postgres::Client,
    step_id: Uuid,
    result: &Result<(), String>,
) -> Result<(), String> {
    lkjmc_store::bootstrap::finish_step(
        client,
        step_id,
        terminal(result),
        result.as_ref().err().map(String::as_str),
    )
    .map_err(|error| error.to_string())
}

fn terminal(result: &Result<(), String>) -> &'static str {
    if result.is_ok() {
        "succeeded"
    } else {
        "failed"
    }
}

fn effect_kind(effect: &NetworkEffect) -> &'static str {
    match effect {
        NetworkEffect::EnsureRoots => "root.ensure",
        NetworkEffect::ReconcileInstance { .. } => "instance.reconcile",
        NetworkEffect::RenderInstance { .. } => "template.render",
        NetworkEffect::StartInstance { .. } => "instance.start",
        NetworkEffect::StopInstance { .. } => "instance.stop",
        NetworkEffect::WaitForReadiness { .. } => "probe.wait",
        NetworkEffect::GenerateForwardingSecret { .. } => "secret.forwarding",
    }
}

fn effect_target(effect: &NetworkEffect) -> &str {
    match effect {
        NetworkEffect::EnsureRoots => "roots",
        NetworkEffect::ReconcileInstance { id, .. }
        | NetworkEffect::RenderInstance { id }
        | NetworkEffect::StartInstance { id }
        | NetworkEffect::StopInstance { id }
        | NetworkEffect::WaitForReadiness { id } => id.as_str(),
        NetworkEffect::GenerateForwardingSecret { .. } => "forwarding-secret",
    }
}
