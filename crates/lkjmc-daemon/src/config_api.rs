use lkjmc_core::command::CommandEnvelope;
use serde_json::json;

use crate::api;
use crate::app::AppState;

type Response = lkjmc_core::command::CommandResponse;

pub fn reload(state: &AppState, request: CommandEnvelope) -> Response {
    let Some(path) = state.config_path() else {
        return api::error(
            request,
            "config.no_path",
            "daemon was not started with a config file",
            false,
        );
    };
    match state.reload_from_file(&path) {
        Ok(()) => api::ok(
            request,
            json!({
                "configPath": path,
                "databaseConfigured": state.database_url().is_some(),
                "configRoot": state.config_root(),
                "logRoot": state.log_root(),
                "jarRoot": state.jar_root(),
                "dataRoot": state.data_root()
            }),
        ),
        Err(error) => api::error(request, "config.reload_failed", error, false),
    }
}
