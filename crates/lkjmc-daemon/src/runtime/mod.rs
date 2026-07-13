pub(crate) mod adapter;
pub(crate) mod coordinator;
pub(crate) mod instance_launch;
pub(crate) mod kubernetes;
pub(crate) mod local;
pub(crate) mod local_adapter;
mod local_identity;
mod local_start;
mod local_stop;
pub(crate) mod logs;
pub(crate) mod process;
pub(crate) mod rcon;
pub(crate) mod reconcile;
mod reconcile_observation;
mod reconcile_plan;

pub(crate) use adapter::{
    ProcessIdentity, RuntimeAdapter, RuntimeCapabilities, RuntimeObservation,
};
pub(crate) use coordinator::LifecycleCoordinator;
pub(crate) use reconcile::RuntimeGoal;

#[cfg(test)]
mod adoption_concurrency_tests;
#[cfg(test)]
mod adoption_tests;
#[cfg(test)]
mod kubernetes_tests;
