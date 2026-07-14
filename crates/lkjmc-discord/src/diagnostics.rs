use std::collections::BTreeMap;
use std::sync::mpsc::{sync_channel, Receiver, SyncSender, TrySendError};
use std::thread::JoinHandle;
use std::time::Duration;

use lkjmc_core::observability::{Component, EventEnvelope, EventKind, Outcome, Severity, Surface};
use serde_json::Value;

const CAPACITY: usize = 64;

pub struct Diagnostics {
    sender: Option<SyncSender<EventEnvelope>>,
    worker: Option<JoinHandle<()>>,
    finished: Receiver<()>,
}

impl Diagnostics {
    pub fn start() -> Self {
        let (sender, receiver) = sync_channel::<EventEnvelope>(CAPACITY);
        let (finished_sender, finished) = sync_channel(1);
        let worker = std::thread::Builder::new()
            .name("lkjmc-discord-diagnostics".into())
            .spawn(move || {
                for event in receiver {
                    if let Ok(line) = serde_json::to_string(&event) {
                        eprintln!("{line}");
                    }
                }
                let _ = finished_sender.send(());
            })
            .ok();
        Self {
            sender: Some(sender),
            worker,
            finished,
        }
    }

    pub fn emit(&self, outcome: Outcome, reason: &str) -> bool {
        let mut attributes = BTreeMap::new();
        attributes.insert(
            "reason".into(),
            Value::String(reason.chars().take(128).collect()),
        );
        let Ok(event) = EventEnvelope::new(
            Severity::Info,
            Component::Discord,
            EventKind::DiscordDiagnostic,
            None,
            None,
            None,
            "discord",
            "discord-adapter",
            Surface::Discord,
            outcome,
            None,
            attributes,
            "discord-local",
        ) else {
            return false;
        };
        self.sender
            .as_ref()
            .is_some_and(|sender| match sender.try_send(event) {
                Ok(()) => true,
                Err(TrySendError::Full(_)) | Err(TrySendError::Disconnected(_)) => false,
            })
    }

    pub fn close(mut self) {
        self.shutdown();
    }

    fn shutdown(&mut self) {
        self.sender.take();
        let completed = self.finished.recv_timeout(Duration::from_secs(2)).is_ok();
        if let Some(worker) = self.worker.take() {
            if completed {
                let _ = worker.join();
            }
        }
    }
}

impl Drop for Diagnostics {
    fn drop(&mut self) {
        self.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_emitter_retains_sensitive_reason_as_redacted() {
        let diagnostics = Diagnostics::start();
        assert!(diagnostics.emit(Outcome::Succeeded, "config-checked"));
        assert!(diagnostics.emit(Outcome::Failed, "https://secret.example"));
        diagnostics.close();
    }
}
