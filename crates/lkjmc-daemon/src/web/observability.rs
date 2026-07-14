use serde_json::json;

use crate::app::AppState;
use crate::web::api::{page, reply, WebReply};
use crate::web::html::escape;

pub(crate) fn view(state: &AppState, csrf: Option<&str>) -> WebReply {
    let readiness = crate::observability::health::readiness_body(state)
        .unwrap_or_else(|code| json!({"ready":false,"errorClass":code,"source":"daemon-local"}));
    let (_, in_flight) = state.admission_diagnostics();
    let metrics = state.metrics().export(in_flight);
    let events = state
        .request_database_connection()
        .and_then(|mut client| {
            lkjmc_store::observability::query(
                &mut *client,
                lkjmc_store::observability::EventQuery {
                    request_id: None,
                    operation_id: None,
                    correlation_id: None,
                    limit: 50,
                },
            )
        })
        .unwrap_or_default();
    let form = csrf.map(|value| format!(
        "<form method=post action=/web/support-bundle><input type=hidden name=_csrf value=\"{}\"><button>create support bundle</button></form>",
        escape(value))).unwrap_or_default();
    let body = format!("<h1>Observability</h1><h2>Readiness</h2><pre>{}</pre><h2>Metrics</h2><pre>{}</pre><h2>Recent events</h2><pre>{}</pre>{form}",
        escape(&serde_json::to_string_pretty(&readiness).unwrap_or_default()), escape(&metrics),
        escape(&serde_json::to_string_pretty(&events).unwrap_or_default()));
    page("observability", body, csrf)
}

pub(crate) fn bundle(state: &AppState, csrf: Option<&str>) -> WebReply {
    let output = std::path::Path::new(&state.data_root())
        .join(format!("support-{}.tar", uuid::Uuid::new_v4().simple()));
    match crate::support::bundle::create(state, &output) {
        Ok(manifest) => page(
            "support bundle",
            format!(
                "<h1>Support bundle created</h1><pre>{}</pre>",
                escape(&serde_json::to_string_pretty(&manifest).unwrap_or_default())
            ),
            csrf,
        ),
        Err(_) => reply(400, "text/plain", "support bundle failed"),
    }
}
