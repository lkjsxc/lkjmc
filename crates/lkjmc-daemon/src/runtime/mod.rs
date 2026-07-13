pub(crate) mod adapter;
pub(crate) mod coordinator;
pub(crate) mod instance_launch;
pub(crate) mod kubernetes;
pub(crate) mod local;
pub(crate) mod local_adapter;
mod local_stop;
pub(crate) mod logs;
pub(crate) mod process;
pub(crate) mod rcon;

pub(crate) use adapter::{
    ProcessIdentity, RuntimeAdapter, RuntimeCapabilities, RuntimeObservation,
};
pub(crate) use coordinator::LifecycleCoordinator;
