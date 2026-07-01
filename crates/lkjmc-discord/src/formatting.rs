use serde_json::Value;

pub fn format_body(body: &Value) -> Option<String> {
    if let Some(instances) = body.get("instances").and_then(Value::as_array) {
        return Some(format_instances(instances));
    }
    if let Some(reports) = body.get("reports").and_then(Value::as_array) {
        return Some(format_reports(reports));
    }
    None
}

fn format_instances(instances: &[Value]) -> String {
    if instances.is_empty() {
        return "servers: none".to_string();
    }
    let rows = instances
        .iter()
        .take(10)
        .filter_map(Value::as_object)
        .map(|object| {
            let id = text(object.get("id"));
            let desired = text(object.get("desiredState"));
            let observed = text(object.get("observedState"));
            let players = object
                .get("presence")
                .and_then(|value| value.get("playerCount"))
                .and_then(Value::as_i64)
                .map_or("?".to_string(), |value| value.to_string());
            format!("{id} desired={desired} observed={observed} players={players}")
        })
        .collect::<Vec<_>>();
    format!("servers:\n{}", rows.join("\n"))
}

fn format_reports(reports: &[Value]) -> String {
    if reports.is_empty() {
        return "reports: none".to_string();
    }
    let rows = reports
        .iter()
        .take(10)
        .filter_map(Value::as_object)
        .map(|object| {
            let id = text(object.get("id"));
            let status = text(object.get("status"));
            let reason = text(object.get("reason"));
            format!("{id} {status}: {reason}")
        })
        .collect::<Vec<_>>();
    format!("reports:\n{}", rows.join("\n"))
}

fn text(value: Option<&Value>) -> String {
    value.and_then(Value::as_str).unwrap_or("-").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn formats_server_list_for_discord() {
        let body = json!({"instances":[{"id":"hub","desiredState":"running","observedState":"process-healthy","presence":{"playerCount":3}}]});
        assert_eq!(
            format_body(&body),
            Some("servers:\nhub desired=running observed=process-healthy players=3".to_string())
        );
    }

    #[test]
    fn formats_empty_reports() {
        assert_eq!(
            format_body(&json!({"reports":[]})),
            Some("reports: none".to_string())
        );
    }
}
