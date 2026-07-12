use crate::app::AppState;

pub fn denial(state: &AppState, surface: &str, reason: &str) {
    let Some(_) = state.database_url() else {
        return;
    };
    let Ok(mut client) = state.database_connection() else {
        return;
    };
    let event = lkjmc_store::audit::NewAuditEvent {
        id: uuid::Uuid::new_v4(),
        actor_kind: "security",
        actor_name: surface,
        action: "security.auth.denied",
        target_kind: "credential",
        target_id: "redacted",
        result: reason,
    };
    let _ = lkjmc_store::audit::insert(&mut *client, event);
}
