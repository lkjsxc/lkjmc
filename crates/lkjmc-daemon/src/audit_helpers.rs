use lkjmc_core::command::{ActorKind, CommandEnvelope};
use lkjmc_store::audit::NewAuditEvent;
use postgres::Client;
use uuid::Uuid;

pub fn audit(
    client: &mut Client,
    request: &CommandEnvelope,
    action: &str,
    target_kind: &str,
    target_id: &str,
    result: &str,
) -> Result<(), String> {
    lkjmc_store::audit::insert(
        client,
        NewAuditEvent {
            id: Uuid::new_v4(),
            actor_kind: actor_kind(request.actor.kind),
            actor_name: &request.actor.name,
            action,
            target_kind,
            target_id,
            result,
        },
    )
    .map_err(|error| error.to_string())
}

fn actor_kind(kind: ActorKind) -> &'static str {
    match kind {
        ActorKind::Cli => "cli",
        ActorKind::VelocityPlugin => "velocity-plugin",
        ActorKind::PaperPlugin => "paper-plugin",
        ActorKind::Daemon => "daemon",
        ActorKind::Installer => "installer",
        ActorKind::WebOperator => "web-operator",
    }
}
