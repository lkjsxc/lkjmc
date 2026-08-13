const HEARTBEAT_TTL_SECONDS: i64 = 30;
const PROXY_REGISTRATION_TTL_SECONDS: i64 = 30;

pub struct Input<'a> {
    pub kind: &'a str,
    pub desired_state: &'a str,
    pub process_healthy: Option<bool>,
    pub connect_port: Option<i64>,
    pub heartbeat_ready: Option<bool>,
    pub heartbeat_age_seconds: Option<i64>,
    pub proxy_registration_desired: bool,
    pub proxy_registered: Option<bool>,
    pub proxy_failure_reason: Option<&'a str>,
    pub proxy_registration_age_seconds: Option<i64>,
}

pub struct Availability {
    pub ready: Option<bool>,
    pub readiness_source: &'static str,
    pub proxy_registered: Option<bool>,
    pub joinable: bool,
    pub join_disabled_reason: String,
}

pub fn evaluate(input: Input<'_>) -> Availability {
    let (ready, readiness_source) = readiness(&input);
    let proxy_registered = current_proxy_registration(&input);
    let (joinable, join_disabled_reason) = joinability(&input);
    Availability {
        ready,
        readiness_source,
        proxy_registered,
        joinable,
        join_disabled_reason,
    }
}

fn readiness(input: &Input<'_>) -> (Option<bool>, &'static str) {
    if input.kind == "velocity" {
        return (None, "unavailable");
    }
    match input.process_healthy {
        Some(false) => (Some(false), "process-observation"),
        None => (None, "unavailable"),
        Some(true) => match (input.heartbeat_ready, input.heartbeat_age_seconds) {
            (Some(ready), Some(age)) => (
                Some(ready && current_age(age, HEARTBEAT_TTL_SECONDS)),
                "process-and-plugin-heartbeat",
            ),
            _ => (None, "unavailable"),
        },
    }
}

fn current_proxy_registration(input: &Input<'_>) -> Option<bool> {
    if !input.proxy_registration_desired {
        return None;
    }
    match (input.proxy_registered, input.proxy_registration_age_seconds) {
        (Some(registered), Some(age)) => {
            Some(registered && current_age(age, PROXY_REGISTRATION_TTL_SECONDS))
        }
        _ => None,
    }
}

fn joinability(input: &Input<'_>) -> (bool, String) {
    if input.kind == "velocity" {
        return denied("not-a-backend");
    }
    if input.desired_state != "running" {
        return denied("desired-state-not-running");
    }
    match input.process_healthy {
        Some(true) => {}
        Some(false) => return denied("server-unhealthy"),
        None => return denied("process-observation-missing"),
    }
    match input.connect_port {
        Some(1..=65535) => {}
        Some(_) => return denied("invalid-connect-port"),
        None => return denied("missing-connect-port"),
    }
    let Some(heartbeat_ready) = input.heartbeat_ready else {
        return denied("heartbeat-missing");
    };
    let Some(heartbeat_age) = input.heartbeat_age_seconds else {
        return denied("heartbeat-age-unknown");
    };
    if heartbeat_age < 0 {
        return denied("heartbeat-age-invalid");
    }
    if heartbeat_age > HEARTBEAT_TTL_SECONDS {
        return denied("heartbeat-stale");
    }
    if !heartbeat_ready {
        return denied("heartbeat-not-ready");
    }
    if !input.proxy_registration_desired {
        return (true, String::new());
    }
    let Some(proxy_registered) = input.proxy_registered else {
        return denied("proxy-registration-unknown");
    };
    let Some(registration_age) = input.proxy_registration_age_seconds else {
        return denied("proxy-registration-age-unknown");
    };
    if registration_age < 0 {
        return denied("proxy-registration-age-invalid");
    }
    if registration_age > PROXY_REGISTRATION_TTL_SECONDS {
        return denied("proxy-registration-stale");
    }
    if !proxy_registered {
        return denied(
            input
                .proxy_failure_reason
                .unwrap_or("proxy-registration-failed"),
        );
    }
    (true, String::new())
}

fn current_age(age: i64, limit: i64) -> bool {
    (0..=limit).contains(&age)
}

fn denied(reason: &str) -> (bool, String) {
    (false, reason.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_evidence_stays_unknown() {
        let value = evaluate(Input {
            kind: "folia",
            desired_state: "running",
            process_healthy: Some(true),
            connect_port: Some(25567),
            heartbeat_ready: None,
            heartbeat_age_seconds: None,
            proxy_registration_desired: true,
            proxy_registered: None,
            proxy_failure_reason: None,
            proxy_registration_age_seconds: None,
        });
        assert_eq!(value.ready, None);
        assert_eq!(value.readiness_source, "unavailable");
        assert_eq!(value.proxy_registered, None);
        assert!(!value.joinable);
        assert_eq!(value.join_disabled_reason, "heartbeat-missing");
    }

    #[test]
    fn future_dated_evidence_never_claims_current_readiness() {
        let heartbeat = evaluate(Input {
            kind: "folia",
            desired_state: "running",
            process_healthy: Some(true),
            connect_port: Some(25567),
            heartbeat_ready: Some(true),
            heartbeat_age_seconds: Some(-1),
            proxy_registration_desired: true,
            proxy_registered: Some(true),
            proxy_failure_reason: None,
            proxy_registration_age_seconds: Some(1),
        });
        assert_eq!(heartbeat.ready, Some(false));
        assert!(!heartbeat.joinable);
        assert_eq!(heartbeat.join_disabled_reason, "heartbeat-age-invalid");

        let registration = evaluate(Input {
            kind: "folia",
            desired_state: "running",
            process_healthy: Some(true),
            connect_port: Some(25567),
            heartbeat_ready: Some(true),
            heartbeat_age_seconds: Some(1),
            proxy_registration_desired: true,
            proxy_registered: Some(true),
            proxy_failure_reason: None,
            proxy_registration_age_seconds: Some(-1),
        });
        assert_eq!(registration.proxy_registered, Some(false));
        assert!(!registration.joinable);
        assert_eq!(
            registration.join_disabled_reason,
            "proxy-registration-age-invalid"
        );
    }

    #[test]
    fn fresh_complete_backend_evidence_is_joinable() {
        let value = evaluate(Input {
            kind: "folia",
            desired_state: "running",
            process_healthy: Some(true),
            connect_port: Some(25567),
            heartbeat_ready: Some(true),
            heartbeat_age_seconds: Some(2),
            proxy_registration_desired: true,
            proxy_registered: Some(true),
            proxy_failure_reason: None,
            proxy_registration_age_seconds: Some(2),
        });
        assert_eq!(value.ready, Some(true));
        assert_eq!(value.proxy_registered, Some(true));
        assert!(value.joinable);
        assert!(value.join_disabled_reason.is_empty());
    }
}
