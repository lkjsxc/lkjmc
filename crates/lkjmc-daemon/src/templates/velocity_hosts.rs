use serde_json::Value;

pub fn forced_hosts(config: &Value, backend: &str) -> String {
    let Some(hosts) = config.get("publicHosts").and_then(Value::as_array) else {
        return String::new();
    };
    hosts
        .iter()
        .filter_map(Value::as_str)
        .filter(|host| !host.is_empty())
        .map(|host| format!("{} = [\"{}\"]\n", toml_key(host), backend))
        .collect::<String>()
}

fn toml_key(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}
