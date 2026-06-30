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

#[cfg(test)]
mod tests {
    use super::*;

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
}
