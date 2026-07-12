#!/usr/bin/env bash
set -euo pipefail

output=${1:?output path required}
root=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
crd="$root/controltrial-crd.yaml"
cr_name="e-hc-control-probe"
crd_name="controltrials.research.lkjmc.io"

record() { printf '%s\n' "$*" | tee -a "$output"; }
blocked() { record "KUBERNETES BLOCKED: $*"; exit 0; }

if ! command -v kubectl >/dev/null 2>&1; then
    blocked 'kubectl is not installed; set LKJMC_KUBERNETES_SMOKE=1 with kubectl and authorization'
fi
if ! kubectl version --client --output=json >"$output.client.json" 2>&1; then
    blocked 'kubectl client version command failed'
fi
record 'KUBERNETES attempted: kubectl server version with a five-second timeout'
if ! kubectl version --request-timeout=5s --output=json >"$output.server.json" 2>&1; then
    blocked 'server access failed; authorized cluster credentials are unavailable'
fi
if [ "${LKJMC_KUBERNETES_SMOKE:-}" != "1" ]; then
    blocked 'LKJMC_KUBERNETES_SMOKE=1 is required after server access succeeds'
fi
if [ "${LKJMC_HC_KUBE_DISPOSABLE:-}" != "1" ]; then
    blocked 'LKJMC_HC_KUBE_DISPOSABLE=1 is required for a destructive research lifecycle'
fi
namespace=${LKJMC_HC_KUBE_NAMESPACE:-}
[ -n "$namespace" ] || blocked 'LKJMC_HC_KUBE_NAMESPACE must name an authorized throwaway namespace'
if [ "${LKJMC_HC_KUBE_CLUSTER_SCOPED:-}" != "1" ]; then
    blocked 'LKJMC_HC_KUBE_CLUSTER_SCOPED=1 is required because CRDs are cluster scoped'
fi
if [ "$(kubectl auth can-i create customresourcedefinitions.apiextensions.k8s.io)" != yes ]; then
    blocked 'identity cannot create a temporary CustomResourceDefinition'
fi
if [ "$(kubectl auth can-i create controltrials.research.lkjmc.io -n "$namespace")" != yes ]; then
    blocked 'identity cannot create the namespaced custom resource'
fi
kubectl get namespace "$namespace" >>"$output" 2>&1 || blocked 'the requested namespace is unavailable'
if kubectl get crd "$crd_name" >/dev/null 2>&1; then
    blocked 'the temporary CRD name already exists and is not owned by this run'
fi

created_crd=0
created_cr=0
cleanup() {
    if [ "$created_cr" = 1 ]; then
        kubectl delete controltrial "$cr_name" -n "$namespace" --ignore-not-found >>"$output" 2>&1 || true
    fi
    if [ "$created_crd" = 1 ]; then
        kubectl delete -f "$crd" --ignore-not-found --wait=true >>"$output" 2>&1 || true
    fi
}
trap cleanup EXIT

kubectl apply -f "$crd" >>"$output" 2>&1
created_crd=1
kubectl wait --for=condition=Established "crd/$crd_name" --timeout=30s >>"$output" 2>&1
cat >"$output.resource.yaml" <<EOF
apiVersion: research.lkjmc.io/v1alpha1
kind: ControlTrial
metadata:
  name: $cr_name
  namespace: $namespace
spec:
  desired: observe-only
EOF
kubectl apply -f "$output.resource.yaml" >>"$output" 2>&1
created_cr=1
kubectl get controltrial "$cr_name" -n "$namespace" >>"$output" 2>&1
kubectl delete controltrial "$cr_name" -n "$namespace" --wait=true >>"$output" 2>&1
created_cr=0
kubectl delete -f "$crd" --wait=true >>"$output" 2>&1
created_crd=0
record 'KUBERNETES PASS: custom resource create/read/delete only; no controller ran'
