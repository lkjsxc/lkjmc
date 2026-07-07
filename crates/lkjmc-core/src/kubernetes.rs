mod observe;

pub use observe::{observe_pods_json, KubernetesPodObservation};

use serde_json::{json, Value};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KubernetesPlanInput {
    pub namespace: String,
    pub instance_id: String,
    pub implementation: String,
    pub image: String,
    pub service_type: String,
    pub storage_class: String,
    pub storage_size: String,
    pub server_port: u16,
    pub cpu_request: String,
    pub memory_request: String,
    pub command: Option<String>,
    pub args: Vec<String>,
    pub env: BTreeMap<String, String>,
    pub working_dir: Option<String>,
    pub labels: BTreeMap<String, String>,
    pub annotations: BTreeMap<String, String>,
    pub readiness_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct KubernetesManifest {
    pub kind: &'static str,
    pub name: String,
    pub object: Value,
}

pub fn plan(input: &KubernetesPlanInput) -> Vec<KubernetesManifest> {
    let labels = labels(input);
    vec![
        manifest("PersistentVolumeClaim", name(input), pvc(input, &labels)),
        manifest("Deployment", name(input), deployment(input, &labels)),
        manifest("Service", name(input), service(input, &labels)),
    ]
}

pub fn object_list(input: &KubernetesPlanInput) -> Value {
    let items = plan(input)
        .into_iter()
        .map(|m| m.object)
        .collect::<Vec<_>>();
    json!({"apiVersion":"v1","kind":"List","items":items})
}

pub fn selector(instance_id: &str) -> String {
    format!("app.kubernetes.io/managed-by=lkjmc,lkjmc.io/instance={instance_id}")
}

fn labels(input: &KubernetesPlanInput) -> BTreeMap<String, String> {
    let mut labels = BTreeMap::from([
        (
            "app.kubernetes.io/managed-by".to_string(),
            "lkjmc".to_string(),
        ),
        (
            "app.kubernetes.io/name".to_string(),
            "minecraft".to_string(),
        ),
        ("lkjmc.io/instance".to_string(), input.instance_id.clone()),
        (
            "lkjmc.io/implementation".to_string(),
            input.implementation.clone(),
        ),
    ]);
    labels.extend(input.labels.clone());
    labels
}

fn metadata(input: &KubernetesPlanInput, labels: &BTreeMap<String, String>) -> Value {
    let mut metadata = json!({"name":name(input),"namespace":input.namespace,"labels":labels});
    if !input.annotations.is_empty() {
        metadata["annotations"] = json!(input.annotations);
    }
    metadata
}

fn pvc(input: &KubernetesPlanInput, labels: &BTreeMap<String, String>) -> Value {
    json!({"apiVersion":"v1","kind":"PersistentVolumeClaim","metadata":metadata(input, labels),"spec":{"accessModes":["ReadWriteOnce"],"storageClassName":input.storage_class,"resources":{"requests":{"storage":input.storage_size}}}})
}

fn deployment(input: &KubernetesPlanInput, labels: &BTreeMap<String, String>) -> Value {
    json!({"apiVersion":"apps/v1","kind":"Deployment","metadata":metadata(input, labels),"spec":{"replicas":1,"selector":{"matchLabels":labels},"template":{"metadata":{"labels":labels,"annotations":input.annotations},"spec":{"containers":[container(input)],"volumes":[{"name":"data","persistentVolumeClaim":{"claimName":name(input)}}]}}}})
}

fn container(input: &KubernetesPlanInput) -> Value {
    let mut container = json!({"name":"minecraft","image":input.image,"ports":[{"containerPort":input.server_port}],"resources":{"requests":{"cpu":input.cpu_request,"memory":input.memory_request}},"volumeMounts":[{"name":"data","mountPath":"/data"}],"readinessProbe":readiness(input)});
    if let Some(command) = input.command.as_ref().filter(|value| !value.is_empty()) {
        container["command"] = json!([command]);
    }
    if !input.args.is_empty() {
        container["args"] = json!(input.args);
    }
    if !input.env.is_empty() {
        container["env"] = json!(input
            .env
            .iter()
            .map(|(name, value)| json!({"name":name,"value":value}))
            .collect::<Vec<_>>());
    }
    if let Some(working_dir) = input.working_dir.as_ref().filter(|value| !value.is_empty()) {
        container["workingDir"] = json!(working_dir);
    }
    container
}

fn readiness(input: &KubernetesPlanInput) -> Value {
    match input
        .readiness_path
        .as_ref()
        .filter(|value| !value.is_empty())
    {
        Some(path) => json!({"httpGet":{"path":path,"port":input.server_port}}),
        None => json!({"tcpSocket":{"port":input.server_port}}),
    }
}

fn service(input: &KubernetesPlanInput, labels: &BTreeMap<String, String>) -> Value {
    json!({"apiVersion":"v1","kind":"Service","metadata":metadata(input, labels),"spec":{"type":input.service_type,"selector":labels,"ports":[{"name":"minecraft","port":input.server_port,"targetPort":input.server_port}]}})
}

fn manifest(kind: &'static str, name: String, object: Value) -> KubernetesManifest {
    KubernetesManifest { kind, name, object }
}

fn name(input: &KubernetesPlanInput) -> String {
    format!("lkjmc-{}", input.instance_id)
}
