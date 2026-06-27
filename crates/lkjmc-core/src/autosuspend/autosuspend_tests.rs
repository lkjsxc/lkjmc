use super::*;

fn input() -> AutosuspendInput {
    AutosuspendInput {
        kind: InstanceKind::Folia,
        desired_state: DesiredState::Running,
        observed_running: true,
        heartbeat_age_seconds: Some(1),
        player_count: Some(0),
        active_sessions: 0,
        uptime_seconds: Some(500),
        empty_since_age_seconds: Some(400),
        consecutive_empty_heartbeats: 2,
        policy: AutosuspendPolicy::default(),
    }
}

#[test]
fn skips_proxy_keepwarm_and_unknown_presence() {
    assert!(matches!(
        plan(AutosuspendInput {
            kind: InstanceKind::Velocity,
            ..input()
        }),
        AutosuspendDecision::Skip { .. }
    ));
    assert!(matches!(
        plan(AutosuspendInput {
            policy: AutosuspendPolicy {
                keep_warm: true,
                ..AutosuspendPolicy::default()
            },
            ..input()
        }),
        AutosuspendDecision::Skip { .. }
    ));
    assert!(matches!(
        plan(AutosuspendInput {
            player_count: None,
            ..input()
        }),
        AutosuspendDecision::Skip { .. }
    ));
}

#[test]
fn manages_empty_and_nonempty_state() {
    assert_eq!(
        plan(AutosuspendInput {
            empty_since_age_seconds: None,
            ..input()
        }),
        AutosuspendDecision::SetEmptySince
    );
    assert_eq!(
        plan(AutosuspendInput {
            player_count: Some(2),
            ..input()
        }),
        AutosuspendDecision::ClearEmptySince
    );
}

#[test]
fn stops_after_grace_and_minimum_uptime() {
    assert!(matches!(
        plan(input()),
        AutosuspendDecision::MarkSuspendedAndStop { .. }
    ));
    assert!(matches!(
        plan(AutosuspendInput {
            uptime_seconds: Some(10),
            ..input()
        }),
        AutosuspendDecision::Skip { .. }
    ));
}

#[test]
fn suspended_state_does_not_restart_and_manual_start_wakes() {
    assert_eq!(
        plan(AutosuspendInput {
            desired_state: DesiredState::Suspended,
            ..input()
        }),
        AutosuspendDecision::Noop
    );
    assert_eq!(manual_start_state(), DesiredState::Running);
}
