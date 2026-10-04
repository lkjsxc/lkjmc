//! Inspection owns a guest boot, never a Minecraft start. All calls hold operations.lock.
use crate::{config::Binding, worker::Worker};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use uuid::Uuid;

pub fn helper_version() -> String {
    let mut hash = Sha256::new();
    hash.update(include_bytes!("../../../ops/guest/guest.py"));
    hash.update(include_bytes!("../../../ops/guest/managed-paths.json"));
    hex::encode(hash.finalize())
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub server: Uuid,
    pub expires: i64,
    pub booted: bool,
    pub closed: bool,
}
impl Session {
    fn admit(id: Uuid, server: Uuid, expires: i64, status: &str) -> Result<Self> {
        ensure!(
            matches!(status, "Running" | "Stopped"),
            "Guest power transition is still in progress"
        );
        Ok(Self {
            id,
            server,
            expires,
            booted: status == "Stopped",
            closed: false,
        })
    }
    fn live(&self) -> bool {
        !self.closed && self.expires > chrono_now()
    }
}
fn chrono_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}
impl Worker {
    pub(crate) async fn upgrade_helper(&self, b: &Binding) -> Result<()> {
        let instance = self.incus.instance(b).await?;
        self.incus.verify(b, &instance)?;
        ensure!(
            instance["status"] == "Running",
            "Guest upgrade waits for an authorized start"
        );
        let mut files = json!({});
        for (name, body, mode) in [
            (
                "managed-paths.json",
                include_str!("../../../ops/guest/managed-paths.json"),
                0o644,
            ),
            (
                "guest.py",
                include_str!("../../../ops/guest/guest.py"),
                0o755,
            ),
        ] {
            files[name] = json!({"text":body,"sha256":hex::encode(Sha256::digest(body.as_bytes())),"mode":mode});
        }
        self.incus
            .exec(
                b,
                &[
                    "python3".into(),
                    "-c".into(),
                    include_str!("../../../ops/guest/upgrade.py").into(),
                ],
                Some(serde_json::to_vec(&files)?),
            )
            .await?;
        // Publish the combined marker only after both regular files and the
        // disabled-autostart effect have completed under the host writer lock.
        self.inspection_config(b, "user.lkjmc.inspection-helper", &helper_version())
            .await?;
        Ok(())
    }
    /// The original boot owner can finish expiry even when Core is unreachable.
    pub(crate) async fn expire_inspections(&self) -> Result<()> {
        let expired: Vec<Session> = self
            .store
            .all::<Session>("inspections")?
            .into_iter()
            .filter(|s| !s.closed && s.expires <= chrono_now())
            .collect();
        if expired.is_empty() {
            return Ok(());
        }
        let _lock = self.lock()?;
        crate::management::guard(&self.config).await?;
        for mut session in expired {
            let b = self.binding(session.server)?;
            let instance = self.incus.instance(&b).await?;
            self.incus.verify(&b, &instance)?;
            if instance["status"] == "Running"
                && instance["expanded_config"]["user.lkjmc.inspection-id"] == session.id.to_string()
            {
                self.incus
                    .helper(&b, "inspection_close", json!({"id":session.id}))
                    .await?;
                if session.booted {
                    self.incus.power(&b, false).await?;
                }
            } else {
                ensure!(
                    instance["status"] == "Stopped"
                        || instance["expanded_config"]["user.lkjmc.inspection-id"]
                            != session.id.to_string(),
                    "Guest transition is still in progress"
                );
            }
            session.closed = true;
            self.store.write("inspections", session.server, &session)?;
        }
        Ok(())
    }
    pub(crate) async fn inspection_config(
        &self,
        b: &Binding,
        key: &str,
        value: &str,
    ) -> Result<()> {
        self.incus
            .run(
                &b.project,
                &[
                    "config".into(),
                    "set".into(),
                    b.instance.clone(),
                    format!("{key}={value}"),
                ],
                None,
            )
            .await?;
        Ok(())
    }
    pub(crate) async fn inspect(&self, job: &Value, server: &Value) -> Result<Value> {
        let sid: Uuid = server["id"].as_str().context("Missing server")?.parse()?;
        let id: Uuid = job["payload"]["inspection"]["id"]
            .as_str()
            .context("Missing inspection ID")?
            .parse()?;
        let b = self.binding(sid)?;
        ensure!(
            b.custom && server["kind"] == "custom" && server["desired"] == "stopped",
            "Inspection requires a stopped custom game target"
        );
        let instance = self.incus.instance(&b).await?;
        self.incus.verify(&b, &instance)?;
        let open = job["payload"]["open"] == true;
        let mut session = self
            .store
            .read::<Session>("inspections", sid)?
            .filter(|s| s.id == id);
        if open && server["inspection_valid"] == true && session.as_ref().is_none_or(|s| s.live()) {
            if session.is_none() {
                // This marker is written only after a reviewed helper + disabled service check.
                // Crucially, verify it BEFORE booting, not after autostart could have happened.
                ensure!(
                    instance["expanded_config"]["user.lkjmc.inspection-helper"] == helper_version(),
                    "Inspection requires the reviewed guest helper/policy and disabled-autostart upgrade"
                );
                if let Some(previous) = self.store.read::<Session>("inspections", sid)? {
                    ensure!(
                        previous.closed,
                        "An earlier inspection boot still needs reconciliation"
                    );
                }
                let s = Session::admit(
                    id,
                    sid,
                    job["payload"]["inspection"]["expires_unix"]
                        .as_i64()
                        .context("Missing deadline")?,
                    instance["status"].as_str().unwrap_or(""),
                )?;
                // Intent precedes the Incus effect, so a crash after boot cannot orphan ownership.
                self.store.write("inspections", sid, &s)?;
                session = Some(s);
            }
            let s = session.as_ref().unwrap();
            self.inspection_config(&b, "user.lkjmc.inspection-id", &id.to_string())
                .await?;
            self.ensure_guest(&b, server).await?; // existing capacity checks, unchanged CPU/RAM
            let fresh = self
                .client
                .request(
                    &format!("/internal/v1/jobs/{}/context", job["id"].as_str().unwrap()),
                    Some(json!({"lease_token":job["lease_token"]})),
                )
                .await?;
            if fresh["server"]["inspection_valid"] == true && s.live() {
                self.incus
                    .helper(&b, "inspection_open", json!({"id":id}))
                    .await?;
                self.client.request("/internal/v1/observations", Some(json!({"server_id":sid,"observed":"stopped","players":0,"metrics":{"guest_ready":true,"inspection_id":id}}))).await?;
                let mut ready = session.as_ref().unwrap().clone();
                ready.expires = ready.expires.min(chrono_now() + 600);
                self.store.write("inspections", sid, &ready)?;
                return Ok(
                    json!({"effect":"committed","inspection_id":id,"open":true,"guest_ready":true,"game_stopped":true,"expires_unix":ready.expires}),
                );
            }
        }
        if let Some(mut s) = session {
            if !s.closed {
                let current = self.incus.instance(&b).await?;
                self.incus.verify(&b, &current)?;
                if current["status"] == "Running" {
                    ensure!(
                        current["expanded_config"]["user.lkjmc.inspection-id"] == id.to_string(),
                        "Guest inspection ownership changed; refusing shutdown"
                    );
                    // Guest lock + stopped PID/cgroup checks refuse a newly running game.
                    self.incus
                        .helper(&b, "inspection_close", json!({"id":id}))
                        .await?;
                    if s.booted {
                        self.incus.power(&b, false).await?;
                    }
                } else {
                    ensure!(
                        current["status"] == "Stopped",
                        "Guest shutdown is still in progress"
                    );
                }
                s.closed = true;
                self.store.write("inspections", sid, &s)?;
            }
        }
        if open {
            return Err(crate::incus::GuestFailure::system(
                "text.the_file_session_expired_or_access_changed_cleanup_is_c_26099b1475",
                true,
            )
            .into());
        }
        Ok(
            json!({"effect":"committed","inspection_id":id,"open":false,"guest_ready":false,"game_stopped":true}),
        )
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn intent_preserves_boot_ownership_deadline_and_closed_replay() {
        let id = Uuid::new_v4();
        let server = Uuid::new_v4();
        for (status, booted) in [("Running", false), ("Stopped", true)] {
            let s = Session::admit(id, server, chrono_now() + 300, status).unwrap();
            let mut recovered: Session =
                serde_json::from_slice(&serde_json::to_vec(&s).unwrap()).unwrap();
            assert_eq!(recovered.booted, booted);
            assert_eq!(recovered.expires, s.expires);
            assert!(recovered.live());
            recovered.closed = true;
            assert!(!recovered.live());
        }
        assert!(
            Session::admit(id, server, chrono_now() - 1, "Stopped")
                .unwrap()
                .live()
                == false
        );
        assert!(Session::admit(id, server, chrono_now() + 300, "Starting").is_err());
    }
}
