use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KubernetesPlanInput<'a> {
    pub namespace: &'a str,
    pub instance_id: &'a str,
    pub implementation: &'a str,
    pub image: &'a str,
    pub service_type: &'a str,
    pub storage_class: &'a str,
    pub storage_size: &'a str,
    pub server_port: u16,
    pub cpu_request: &'a str,
    pub memory_request: &'a str,
}

#[derive(Debug, Clone, PartialEq)]
pub struct KubernetesManifest {
    pub kind: &'static str,
    pub name: String,
    pub object: Value,
}

pub fn plan(input: &KubernetesPlanInput<'_>) -> Vec<KubernetesManifest> {
    let labels = labels(input);
    vec![
        manifest("PersistentVolumeClaim", name(input), pvc(input, &labels)),
        manifest("Deployment", name(input), deployment(input, &labels)),
        manifest("Service", name(input), service(input, &labels)),
    ]
}

pub fn object_list(input: &KubernetesPlanInput<'_>) -> Value {
    json!({"apiVersion":"v1","kind":"List","items": plan(input).into_iter().map(|m| m.object).collect::<Vec<_>>()})
}

pub fn selector(instance_id: &str) -> String {
    format!("app.kubernetes.io/managed-by=lkjmc,lkjmc.io/instance={instance_id}")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KubernetesPodObservation {
    pub ready: bool,
    pub phase: Option<String>,
    pub restart_count: i64,
    pub last_error: Option<String>,
}

pub fn observe_pods_json(input: &str) -> Result<Option<KubernetesPodObservation>, String> {
    let list: PodList = serde_json::from_str(input).map_err(|error| error.to_string())?;
    if list.items.is_empty() {
        return Ok(None);
    }
    let ready = list.items.iter().any(pod_ready);
    let restart_count = list
        .items
        .iter()
        .flat_map(|pod| pod.status.container_statuses.iter().flatten())
        .map(|status| status.restart_count)
        .sum();
    let last_error = list.items.iter().find_map(pod_error);
    let phase = list.items.iter().find_map(|pod| pod.status.phase.clone());
    Ok(Some(KubernetesPodObservation {
        ready,
        phase,
        restart_count,
        last_error,
    }))
}

fn pod_ready(pod: &Pod) -> bool {
    let ready_condition = pod.status.conditions.iter().flatten().any(|condition| {
        condition.kind == "Ready" && condition.status.eq_ignore_ascii_case("true")
    });
    let containers_ready = pod
        .status
        .container_statuses
        .as_ref()
        .is_some_and(|items| !items.is_empty() && items.iter().all(|item| item.ready));
    ready_condition && containers_ready
}

fn pod_error(pod: &Pod) -> Option<String> {
    pod.status
        .container_statuses
        .iter()
        .flatten()
        .filter_map(|status| status.state.as_ref())
        .find_map(ContainerState::reason)
}

#[derive(Debug, Deserialize)]
struct PodList {
    items: Vec<Pod>,
}

#[derive(Debug, Deserialize)]
struct Pod {
    #[serde(default)]
    status: PodStatus,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PodStatus {
    phase: Option<String>,
    #[serde(default)]
    conditions: Option<Vec<PodCondition>>,
    container_statuses: Option<Vec<ContainerStatus>>,
}

#[derive(Debug, Deserialize)]
struct PodCondition {
    #[serde(rename = "type")]
    kind: String,
    status: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ContainerStatus {
    ready: bool,
    restart_count: i64,
    state: Option<ContainerState>,
}

#[derive(Debug, Deserialize)]
struct ContainerState {
    waiting: Option<StateReason>,
    terminated: Option<StateReason>,
}

impl ContainerState {
    fn reason(&self) -> Option<String> {
        self.waiting
            .as_ref()
            .or(self.terminated.as_ref())
            .and_then(|state| state.reason.clone())
    }
}

#[derive(Debug, Deserialize)]
struct StateReason {
    reason: Option<String>,
}

fn labels(input: &KubernetesPlanInput<'_>) -> BTreeMap<String, String> {
    BTreeMap::from([
        (
            "app.kubernetes.io/managed-by".to_string(),
            "lkjmc".to_string(),
        ),
        (
            "app.kubernetes.io/name".to_string(),
            "minecraft".to_string(),
        ),
        (
            "lkjmc.io/instance".to_string(),
            input.instance_id.to_string(),
        ),
        (
            "lkjmc.io/implementation".to_string(),
            input.implementation.to_string(),
        ),
    ])
}

fn pvc(input: &KubernetesPlanInput<'_>, labels: &BTreeMap<String, String>) -> Value {
    json!({"apiVersion":"v1","metadata":{"name":name(input),"namespace":input.namespace,"labels":labels},"spec":{"accessModes":["ReadWriteOnce"],"storageClassName":input.storage_class,"resources":{"requests":{"storage":input.storage_size}}}})
}

fn deployment(input: &KubernetesPlanInput<'_>, labels: &BTreeMap<String, String>) -> Value {
    json!({"apiVersion":"apps/v1","metadata":{"name":name(input),"namespace":input.namespace,"labels":labels},"spec":{"replicas":1,"selector":{"matchLabels":labels},"template":{"metadata":{"labels":labels},"spec":{"containers":[{"name":"minecraft","image":input.image,"ports":[{"containerPort":input.server_port}],"resources":{"requests":{"cpu":input.cpu_request,"memory":input.memory_request}},"volumeMounts":[{"name":"data","mountPath":"/data"}],"readinessProbe":{"tcpSocket":{"port":input.server_port}}}],"volumes":[{"name":"data","persistentVolumeClaim":{"claimName":name(input)}}]}}}})
}

fn service(input: &KubernetesPlanInput<'_>, labels: &BTreeMap<String, String>) -> Value {
    json!({"apiVersion":"v1","metadata":{"name":name(input),"namespace":input.namespace,"labels":labels},"spec":{"type":input.service_type,"selector":labels,"ports":[{"name":"minecraft","port":input.server_port,"targetPort":input.server_port}]}})
}

fn manifest(kind: &'static str, name: String, object: Value) -> KubernetesManifest {
    KubernetesManifest { kind, name, object }
}

fn name(input: &KubernetesPlanInput<'_>) -> String {
    format!("lkjmc-{}", input.instance_id)
}
