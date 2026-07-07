use serde::Deserialize;

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
    Ok(Some(KubernetesPodObservation {
        ready,
        phase: list.items.iter().find_map(|pod| pod.status.phase.clone()),
        restart_count,
        last_error: list.items.iter().find_map(pod_error),
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
