use serde_json::{json, Value};

pub fn command_payload() -> Value {
    json!([])
}

#[cfg(test)]
mod tests {
    use super::command_payload;

    #[test]
    fn withdrawal_payload_has_no_commands() {
        assert_eq!(command_payload(), serde_json::json!([]));
    }
}
