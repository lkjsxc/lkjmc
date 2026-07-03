use lkjmc_core::command::CommandResponse;
use serde_json::json;

pub fn render(response: CommandResponse) -> String {
    if response.ok {
        let body = response.body.unwrap_or_else(|| json!({}));
        return format!(
            "<pre>{}</pre>",
            escape(&serde_json::to_string_pretty(&body).unwrap_or_default())
        );
    }
    let message = response
        .error
        .map(|error| format!("{}: {}", error.code, error.message))
        .unwrap_or_default();
    format!("<p class=error>{}</p>", escape(&message))
}

pub fn login_form(error: Option<&str>) -> String {
    let error = error
        .map(|value| format!("<p class=error>{}</p>", escape(value)))
        .unwrap_or_default();
    format!("<!doctype html><title>login</title>{error}<form method=post action=/web/login><input name=password type=password><button>login</button></form>")
}

pub fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::escape;

    #[test]
    fn escapes_html_sensitive_characters() {
        assert_eq!(escape("<&>\""), "&lt;&amp;&gt;&quot;");
    }
}
