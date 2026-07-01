use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebRequest {
    pub method: String,
    pub path: String,
    headers: BTreeMap<String, String>,
    pub body: String,
}

impl WebRequest {
    pub fn parse(raw: &str) -> Option<Self> {
        let (head, body) = raw.split_once("\r\n\r\n").unwrap_or((raw, ""));
        let mut lines = head.lines();
        let mut first = lines.next()?.split_whitespace();
        let method = first.next()?.to_string();
        let path = first.next()?.to_string();
        let mut headers = BTreeMap::new();
        for line in lines {
            let Some((name, value)) = line.split_once(':') else {
                continue;
            };
            headers.insert(name.trim().to_ascii_lowercase(), value.trim().to_string());
        }
        Some(Self {
            method,
            path,
            headers,
            body: body.to_string(),
        })
    }

    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .get(&name.to_ascii_lowercase())
            .map(String::as_str)
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

    #[test]
    fn parses_cookie_and_form_values() {
        let request = WebRequest::parse(
            "POST /web/login HTTP/1.1\r\nCookie: a=1; lkjmc_session=s\r\n\r\npassword=a+b%21",
        )
        .unwrap();
        assert_eq!(request.cookie("lkjmc_session").as_deref(), Some("s"));
        assert_eq!(request.form_value("password").as_deref(), Some("a b!"));
    }
}
