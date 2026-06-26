use serde_json::json;

use crate::args_bootstrap::{BootstrapCommand, BootstrapOptions};
use crate::commands::daemon_command;
use crate::error::CliError;

pub fn run(socket: &str, command: BootstrapCommand, json_output: bool) -> Result<(), CliError> {
    match command {
        BootstrapCommand::Plan(options) => daemon_command(
            socket,
            "bootstrap.plan",
            body(options),
            json_output,
            "ok bootstrap plan",
        ),
        BootstrapCommand::Apply(options) => daemon_command(
            socket,
            "bootstrap.apply",
            body(options),
            json_output,
            "ok bootstrap apply",
        ),
        BootstrapCommand::Status => daemon_command(
            socket,
            "bootstrap.status",
            json!({}),
            json_output,
            "ok bootstrap status",
        ),
        BootstrapCommand::Doctor { host } => {
            let mut body = json!({});
            if let Some(host) = host {
                body["javaPublicHost"] = json!(host);
            }
            daemon_command(
                socket,
                "bootstrap.doctor",
                body,
                json_output,
                "ok bootstrap doctor",
            )
        }
    }
}

fn body(options: BootstrapOptions) -> serde_json::Value {
    let mut body = json!({
        "profile": options.profile,
        "acceptMinecraftEula": options.accept_minecraft_eula
    });
    if let Some(bedrock) = options.bedrock {
        body["bedrock"] = json!(bedrock);
    }
    if let Some(bind_host) = options.java_bind_host {
        body["javaBindHost"] = json!(bind_host);
    }
    if let Some(port) = options.java_port {
        body["javaPort"] = json!(port);
    }
    if let Some(host) = options.java_public_host {
        body["javaPublicHost"] = json!(host);
    }
    if let Some(port) = options.bedrock_port {
        body["bedrockPort"] = json!(port);
    }
    body
}
