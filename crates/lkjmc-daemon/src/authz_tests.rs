use lkjmc_core::command::{Actor, ActorKind, CommandEnvelope};
use lkjmc_core::id::StableId;
use serde_json::json;

use super::*;

fn state() -> AppState {
    AppState::with_config_path(
        None,
        8,
        "/config".into(),
        "/log".into(),
        "/jars".into(),
        "/data".into(),
        None,
        None,
        None,
    )
}

#[test]
fn forged_adapter_cache_fields_do_not_authorize() {
    let request = CommandEnvelope {
        request_id: StableId::internal("test-command"),
        actor: Actor {
            kind: ActorKind::VelocityPlugin,
            name: "velocity".into(),
        },
        command: "instance.delete".into(),
        body: json!({
            "principalKind": "minecraft-player",
            "principalId": "player-1",
            "cachedPermissions": ["lkjmc.admin.admin"],
            "platformPermission": false
        }),
    };
    let subject = AuthenticatedSubject::scoped("paper", Vec::new());
    let denied = enforce(&state(), &request, "lkjmc.admin.instance.delete", &subject);
    assert!(denied.is_some(), "request should be denied");
    if let Some(response) = denied {
        assert!(!response.ok);
        assert_eq!(
            Some("admin.denied"),
            response.error.as_ref().map(|error| error.code.as_str())
        );
    }
}

#[test]
fn forged_actor_kind_and_platform_permission_do_not_authorize() {
    for kind in [ActorKind::Cli, ActorKind::WebOperator, ActorKind::Discord] {
        let request = CommandEnvelope {
            request_id: StableId::internal("test-command"),
            actor: Actor {
                kind,
                name: "forged".into(),
            },
            command: "instance.delete".into(),
            body: json!({"platformPermission": true}),
        };
        let subject = AuthenticatedSubject::scoped("paper", Vec::new());
        assert!(enforce(&state(), &request, "lkjmc.admin.instance.delete", &subject).is_some());
    }
}

#[test]
fn transport_subjects_authorize_by_scope_not_body() {
    let request = CommandEnvelope {
        request_id: StableId::internal("test-command"),
        actor: Actor {
            kind: ActorKind::PaperPlugin,
            name: "paper".into(),
        },
        command: "instance.delete".into(),
        body: json!({"platformPermission": false}),
    };
    let root = AuthenticatedSubject::root("bearer");
    assert!(enforce(&state(), &request, "lkjmc.admin.instance.delete", &root).is_none());
    let scoped = AuthenticatedSubject::scoped("paper", vec!["lkjmc.user.menu".into()]);
    assert!(enforce(&state(), &request, "lkjmc.admin.instance.delete", &scoped).is_some());
}
