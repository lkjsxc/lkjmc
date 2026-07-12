use std::collections::BTreeMap;

use axum::http::HeaderMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebRequest {
    pub method: String,
    pub path: String,
    headers: BTreeMap<String, String>,
    pub body: String,
    source: String,
}

impl WebRequest {
    pub fn new(
        method: &str,
        path: &str,
        headers: &HeaderMap,
        body: String,
        source: Option<String>,
    ) -> Self {
        let headers = headers
            .iter()
            .filter_map(|(name, value)| {
                Some((
                    name.as_str().to_ascii_lowercase(),
                    value.to_str().ok()?.to_string(),
                ))
            })
            .collect();
        Self {
            method: method.to_string(),
            path: path.to_string(),
            headers,
            body,
            source: source.unwrap_or_else(|| "unattributed".into()),
        }
    }

    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .get(&name.to_ascii_lowercase())
            .map(String::as_str)
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn route(&self) -> &str {
        self.path.split('?').next().unwrap_or(&self.path)
    }

    pub fn cookie(&self, name: &str) -> Option<String> {
        self.header("cookie")?.split(';').find_map(|part| {
            let (key, value) = part.trim().split_once('=')?;
            (key == name).then(|| value.to_string())
        })
    }

    pub fn form_value(&self, key: &str) -> Option<String> {
        self.body.split('&').find_map(|pair| {
            let (name, value) = pair.split_once('=')?;
            (decode(name) == key).then(|| decode(value))
        })
    }
}

pub fn decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'+' => {
                out.push(b' ');
                index += 1;
            }
            b'%' if index + 2 < bytes.len() => {
                if let Ok(hex) = u8::from_str_radix(&input[index + 1..index + 3], 16) {
                    out.push(hex);
                    index += 3;
                } else {
                    out.push(bytes[index]);
                    index += 1;
                }
            }
            value => {
                out.push(value);
                index += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::header::COOKIE;

    #[test]
    fn parses_cookie_and_form_values() -> Result<(), String> {
        let mut headers = HeaderMap::new();
        headers.insert(
            COOKIE,
            "a=1; lkjmc_session=s".parse().map_err(|_| "cookie")?,
        );
        let request = WebRequest::new(
            "POST",
            "/web/login",
            &headers,
            "password=a+b%21".into(),
            None,
        );
        assert_eq!(request.cookie("lkjmc_session").as_deref(), Some("s"));
        assert_eq!(request.form_value("password").as_deref(), Some("a b!"));
        Ok(())
    }
}
