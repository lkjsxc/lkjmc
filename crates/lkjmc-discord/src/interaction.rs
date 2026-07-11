use std::collections::HashMap;

use serde_json::{json, Value};

use crate::config::Config;

pub(crate) fn handle_interaction(
    config: &Config,
    headers: &HashMap<String, String>,
    body: &str,
) -> (u16, Value) {
    if let Err(error) = crate::signature::verify(config, headers, body) {
        return (401, response(&format!("signature denied: {error}")));
    }
    let Ok(value) = serde_json::from_str::<Value>(body) else {
        return (400, response("invalid interaction JSON"));
    };
    (200, interaction_response(&value))
}

fn interaction_response(value: &Value) -> Value {
    if value.get("type").and_then(Value::as_i64) == Some(1) {
        json!({"type": 1})
    } else {
        response("Discord commands are withdrawn")
    }
}

fn response(content: &str) -> Value {
    json!({"type": 4, "data": {"content": content.chars().take(1800).collect::<String>(), "flags": 64}})
}

#[cfg(test)]
mod tests {
    use super::interaction_response;
    use serde_json::json;

    #[test]
    fn non_ping_interaction_cannot_create_a_command_plan() {
        let response = interaction_response(&json!({"type": 2, "data": {"name": "lkjmc"}}));
        assert_eq!(
            response["data"]["content"],
            "Discord commands are withdrawn"
        );
    }
}
