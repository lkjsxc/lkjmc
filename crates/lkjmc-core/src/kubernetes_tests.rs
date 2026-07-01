use crate::kubernetes::{observe_pods_json, plan, selector, KubernetesPlanInput};

#[test]
fn planner_sets_owned_labels_and_storage() {
    let input = KubernetesPlanInput {
        namespace: "lkjmc",
        instance_id: "hub",
        implementation: "folia",
        image: "minecraft:test",
        service_type: "ClusterIP",
        storage_class: "standard",
        storage_size: "1Gi",
        server_port: 25565,
        cpu_request: "500m",
        memory_request: "1Gi",
    };
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
