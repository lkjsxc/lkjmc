use std::os::unix::fs::MetadataExt;
use std::path::Path;

use axum::extract::connect_info::Connected;
use axum::extract::{ConnectInfo, State};
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::serve::IncomingStream;

use crate::app::AppState;

#[derive(Clone, Debug)]
pub struct UnixPeer {
    uid: Option<u32>,
    gid: Option<u32>,
}

impl Connected<IncomingStream<'_, tokio::net::UnixListener>> for UnixPeer {
    fn connect_info(stream: IncomingStream<'_, tokio::net::UnixListener>) -> Self {
        let credentials = stream.io().peer_cred().ok();
        Self {
            uid: credentials.as_ref().map(tokio::net::unix::UCred::uid),
            gid: credentials.as_ref().map(tokio::net::unix::UCred::gid),
        }
    }
}

#[derive(Clone, Debug)]
pub struct UnixPeerPolicy {
    owner_uid: u32,
    group_gid: u32,
}

impl UnixPeerPolicy {
    pub fn from_socket(path: &Path) -> Result<Self, String> {
        let metadata =
            std::fs::metadata(path).map_err(|_| "socket peer policy unavailable".to_string())?;
        Ok(Self {
            owner_uid: metadata.uid(),
            group_gid: metadata.gid(),
        })
    }

    fn allows(&self, peer: &UnixPeer) -> bool {
        peer.uid == Some(self.owner_uid) || peer.gid == Some(self.group_gid)
    }
}

pub async fn require_unix_peer(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<UnixPeer>,
    mut request: Request<axum::body::Body>,
    next: Next,
) -> Response {
    let allowed = state
        .unix_peer_policy()
        .is_some_and(|policy| policy.allows(&peer));
    let Some(uid) = peer.uid.filter(|_| allowed) else {
        let audit_state = state.clone();
        std::mem::drop(tokio::task::spawn_blocking(move || {
            crate::security_audit::denial(&audit_state, "unix", "peer-denied")
        }));
        return (
            StatusCode::FORBIDDEN,
            "{\"ok\":false,\"error\":{\"code\":\"auth.denied\"}}",
        )
            .into_response();
    };
    request
        .extensions_mut()
        .insert(crate::authz::AuthenticatedSubject::unix_peer(uid));
    next.run(request).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn peer_policy_rejects_an_unrelated_identity() {
        let policy = UnixPeerPolicy {
            owner_uid: 1000,
            group_gid: 2000,
        };
        assert!(policy.allows(&UnixPeer {
            uid: Some(1000),
            gid: Some(99),
        }));
        assert!(policy.allows(&UnixPeer {
            uid: Some(99),
            gid: Some(2000),
        }));
        assert!(!policy.allows(&UnixPeer {
            uid: Some(99),
            gid: Some(98),
        }));
    }
}
