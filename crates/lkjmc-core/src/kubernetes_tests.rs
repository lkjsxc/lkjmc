use crate::kubernetes::{observe_pods_json, plan, selector, KubernetesPlanInput};
use std::collections::BTreeMap;

fn input(port: u16, implementation: &str) -> KubernetesPlanInput {
    KubernetesPlanInput {
        namespace: "lkjmc".to_string(),
        instance_id: "hub".to_string(),
        implementation: implementation.to_string(),
        image: "minecraft:test".to_string(),
        service_type: "ClusterIP".to_string(),
        storage_class: "standard".to_string(),
        storage_size: "1Gi".to_string(),
        server_port: port,
        cpu_request: "500m".to_string(),
        memory_request: "1Gi".to_string(),
        command: None,
        args: Vec::new(),
        env: BTreeMap::new(),
        working_dir: None,
        labels: BTreeMap::new(),
        annotations: BTreeMap::new(),
        readiness_path: None,
    }
}

#[test]
fn planner_sets_owned_labels_and_storage() {
    let input = input(25565, "folia");
    let objects = plan(&input);
    assert_eq!(objects.len(), 3);
    assert_eq!(
        objects[0].object["metadata"]["labels"]["lkjmc.io/instance"],
        "hub"
    );
    assert_eq!(
        selector("hub"),
        "app.kubernetes.io/managed-by=lkjmc,lkjmc.io/instance=hub"
    );
}

#[test]
fn planner_uses_launch_inputs() -> Result<(), String> {
    let args = vec![
        "-jar".to_string(),
        "server.jar".to_string(),
        "nogui".to_string(),
    ];
    let env = BTreeMap::from([(
        "LKJMC_DAEMON_HTTP_TOKEN_FILE".to_string(),
        "/etc/lkjmc/daemon-http.token".to_string(),
    )]);
    let labels = BTreeMap::from([("template".to_string(), "paper-survival".to_string())]);
    let annotations = BTreeMap::from([("checksum/config".to_string(), "abc".to_string())]);
    let input = KubernetesPlanInput {
        command: Some("java".to_string()),
        args,
        env,
        working_dir: Some("/data".to_string()),
        labels,
        annotations,
        readiness_path: Some("/ready".to_string()),
        ..input(25577, "paper")
    };
    let deployment = plan(&input)
        .into_iter()
        .find(|object| object.kind == "Deployment")
        .ok_or_else(|| "deployment".to_string())?;
    let container = &deployment.object["spec"]["template"]["spec"]["containers"][0];
    assert_eq!(container["ports"][0]["containerPort"], 25577);
    assert_eq!(container["command"][0], "java");
    assert_eq!(container["args"][1], "server.jar");
    assert_eq!(container["env"][0]["name"], "LKJMC_DAEMON_HTTP_TOKEN_FILE");
    assert_eq!(container["workingDir"], "/data");
    assert_eq!(container["readinessProbe"]["httpGet"]["path"], "/ready");
    Ok(())
}

#[test]
fn planner_covers_supported_server_kinds() {
    for kind in ["paper", "folia", "velocity", "vanilla-custom"] {
        let objects = plan(&input(25565, kind));
        assert_eq!(
            objects[1].object["metadata"]["labels"]["lkjmc.io/implementation"],
            kind
        );
    }
}

#[test]
fn observes_ready_pod_json() -> Result<(), String> {
    let text = r#"{"items":[{"status":{"phase":"Running","conditions":[{"type":"Ready","status":"True"}],"containerStatuses":[{"ready":true,"restartCount":2,"state":{}}]}}]}"#;
    let observation = observe_pods_json(text)?.ok_or_else(|| "missing pod".to_string())?;
    assert!(observation.ready);
    assert_eq!(observation.phase.as_deref(), Some("Running"));
    assert_eq!(observation.restart_count, 2);
    Ok(())
}

#[test]
fn observes_waiting_reason() -> Result<(), String> {
    let text = r#"{"items":[{"status":{"phase":"Pending","conditions":[],"containerStatuses":[{"ready":false,"restartCount":0,"state":{"waiting":{"reason":"ImagePullBackOff"}}}]}}]}"#;
    let observation = observe_pods_json(text)?.ok_or_else(|| "missing pod".to_string())?;
    assert!(!observation.ready);
    assert_eq!(observation.last_error.as_deref(), Some("ImagePullBackOff"));
    Ok(())
}
