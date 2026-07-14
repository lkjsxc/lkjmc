use std::net::TcpListener;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::process::Command;

use lkjmc_core::config::LkjmcConfig;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::app::AppState;

pub(super) fn build_state(root: &Path, url: String) -> Result<AppState, String> {
    let state = AppState::with_config_path(
        Some(url),
        1,
        path(root, "config"),
        path(root, "logs"),
        path(root, "jars"),
        path(root, "data"),
        Some(path(root, "lkjmc.json")),
        None,
        None,
    );
    state.with_runtime_metadata(path(root, "run/daemon.sock"), None, false)?;
    Ok(state)
}

pub(super) fn write_config(root: &Path, valid_proxy: bool) -> Result<LkjmcConfig, String> {
    let mut value: Value = serde_json::from_str(include_str!(
        "../../../../../../../config/defaults/daemon.json.example"
    ))
    .map_err(|error| error.to_string())?;
    value["configRoot"] = json!(path(root, "config"));
    value["dataRoot"] = json!(path(root, "data"));
    value["logRoot"] = json!(path(root, "logs"));
    value["socketPath"] = json!(path(root, "run/daemon.sock"));
    value["jars"]["root"] = json!(path(root, "jars"));
    value["assets"]["root"] = json!(path(root, "assets"));
    value["daemonHttp"]["tokenFile"] = json!(path(root, "config/http.token"));
    value["network"]["forwarding"]["secretFile"] = json!(path(root, "config/forwarding.secret"));
    value["network"]["listeners"][0]["port"] = json!(free_port()?);
    value["network"]["listeners"][1]["port"] = json!(free_port()?);
    let hub_probe = probe_jar(root, "HubProbe")?;
    let proxy_probe = probe_jar(root, "ProxyProbe")?;
    let hub = asset(root, "folia-server", &hub_probe)?;
    let proxy_bytes = if valid_proxy {
        proxy_probe.as_slice()
    } else {
        b"invalid jar"
    };
    let proxy = asset(root, "velocity-server", proxy_bytes)?;
    value["network"]["assets"] = json!([hub, proxy]);
    value["network"]["instances"][0]["assetIds"] = json!(["folia-server"]);
    value["network"]["instances"][1]["assetIds"] = json!(["velocity-server"]);
    value["network"]["capabilities"]["mountedAssets"] = json!(true);
    let text = serde_json::to_string_pretty(&value).map_err(|error| error.to_string())?;
    let config = LkjmcConfig::from_json_str(&text).map_err(|error| error.to_string())?;
    std::fs::write(root.join("lkjmc.json"), text).map_err(|error| error.to_string())?;
    write_secret(&config.network.forwarding.secret_file)?;
    Ok(config)
}

pub(super) fn insert_instance(
    client: &mut postgres::Client,
    config: &LkjmcConfig,
    id: &str,
    kind: &str,
    command: &str,
) -> Result<(), String> {
    let instance = config
        .network
        .instances
        .iter()
        .find(|value| value.id == id)
        .ok_or("fixture instance missing")?;
    let port = config
        .network
        .listener(&instance.listener)
        .ok_or("fixture listener missing")?
        .port;
    lkjmc_store::instance::insert(
        client,
        id,
        None,
        kind,
        "running",
        &launch_config(id, port, command, &config.network.forwarding.secret_file),
    )
    .map_err(|error| error.to_string())
}

pub(super) fn launch_config(id: &str, port: u16, command: &str, secret: &str) -> Value {
    let script = format!("import socket,time;s=socket.socket();s.setsockopt(socket.SOL_SOCKET,socket.SO_REUSEADDR,1);s.bind(('127.0.0.1',{port}));s.listen();print('Done (0.1s)!',flush=True);time.sleep(300)");
    json!({"template":"default","serverPort":port,"forwardingSecretFile":secret,
        "launch":{"command":command,"args":["-u","-c",script]},"id":id})
}

fn write_secret(path: &str) -> Result<(), String> {
    let path = Path::new(path);
    std::fs::create_dir_all(path.parent().ok_or("secret parent missing")?)
        .map_err(|error| error.to_string())?;
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o600)
        .open(path)
        .map_err(|error| error.to_string())?;
    file.write_all(Uuid::new_v4().simple().to_string().as_bytes())
        .map_err(|error| error.to_string())
}

fn probe_jar(root: &Path, class: &str) -> Result<Vec<u8>, String> {
    let build = root.join(format!("probe-jar-{class}"));
    std::fs::create_dir_all(&build).map_err(|error| error.to_string())?;
    let source = build.join(format!("{class}.java"));
    let java = format!(
        "import java.net.*; public class {class} {{ public static void main(String[] a) throws Exception {{ int p=Integer.parseInt(System.getenv(\"LKJMC_SERVER_PORT\")); try(ServerSocket s=new ServerSocket(p,50,InetAddress.getByName(\"127.0.0.1\"))){{ System.out.println(\"Done (0.1s)!\"); Thread.sleep(300000); }} }} }}"
    );
    std::fs::write(&source, java).map_err(|error| error.to_string())?;
    run(
        Command::new("javac").arg("-d").arg(&build).arg(&source),
        "javac",
    )?;
    let jar = build.join("network-probe.jar");
    run(
        Command::new("jar")
            .arg("--create")
            .arg("--file")
            .arg(&jar)
            .arg("--main-class")
            .arg(class)
            .arg("-C")
            .arg(&build)
            .arg(format!("{class}.class")),
        "jar",
    )?;
    std::fs::read(jar).map_err(|error| error.to_string())
}

fn run(command: &mut Command, label: &str) -> Result<(), String> {
    let output = command
        .output()
        .map_err(|error| format!("{label}: {error}"))?;
    output.status.success().then_some(()).ok_or_else(|| {
        format!(
            "{label} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

fn asset(root: &Path, id: &str, bytes: &[u8]) -> Result<Value, String> {
    let file = root.join("assets").join(format!("{id}.jar"));
    std::fs::create_dir_all(file.parent().ok_or("asset parent missing")?)
        .map_err(|error| error.to_string())?;
    std::fs::write(&file, bytes).map_err(|error| error.to_string())?;
    Ok(json!({"id":id,"kind":"server","path":file,
        "sha256":format!("{:x}", Sha256::digest(bytes)),"required":true}))
}

fn free_port() -> Result<u16, String> {
    Ok(TcpListener::bind("127.0.0.1:0")
        .map_err(|error| error.to_string())?
        .local_addr()
        .map_err(|error| error.to_string())?
        .port())
}

fn path(root: &Path, child: &str) -> String {
    root.join(child).to_string_lossy().into()
}
