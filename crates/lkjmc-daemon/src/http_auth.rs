pub(crate) fn authorized(request: &str, token: Option<&str>) -> bool {
    let Some(token) = token else {
        return false;
    };
    if token.trim().is_empty() {
        return false;
    }
    request.lines().any(|line| {
        authorization_value(line)
            .and_then(bearer_credential)
            .is_some_and(|credential| constant_time_eq(credential.as_bytes(), token.as_bytes()))
    })
}

fn authorization_value(line: &str) -> Option<&str> {
    let (name, value) = line.split_once(':')?;
    name.trim()
        .eq_ignore_ascii_case("authorization")
        .then_some(value.trim())
}

fn bearer_credential(value: &str) -> Option<&str> {
    let scheme_end = value.find(|character: char| character.is_ascii_whitespace())?;
    let (scheme, rest) = value.split_at(scheme_end);
    if !scheme.eq_ignore_ascii_case("bearer") {
        return None;
    }
    let credential = rest.trim_start_matches(|character: char| character.is_ascii_whitespace());
    (!credential.is_empty()).then_some(credential)
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let mut diff = 0_u8;
    for (left, right) in left.iter().zip(right.iter()) {
        diff |= left ^ right;
    }
    diff == 0
}

#[cfg(test)]
mod tests {
    use super::authorized;

    #[test]
    fn http_api_accepts_mixed_case_base64_token() {
        let token = "AbCdEFghIJ09+/==";
        assert!(authorized(
            &request("Authorization: Bearer AbCdEFghIJ09+/=="),
            Some(token)
        ));
    }

    #[test]
    fn http_api_accepts_lowercase_header_name() {
        assert!(authorized(
            &request("authorization: Bearer AbCd"),
            Some("AbCd")
        ));
    }

    #[test]
    fn http_api_accepts_uppercase_bearer_scheme() {
        assert!(authorized(
            &request("Authorization: BEARER AbCd"),
            Some("AbCd")
        ));
    }

    #[test]
    fn http_api_rejects_wrong_token() {
        assert!(!authorized(
            &request("Authorization: Bearer wrong"),
            Some("AbCd")
        ));
    }

    #[test]
    fn http_api_rejects_missing_token_configuration() {
        assert!(!authorized(&request("Authorization: Bearer AbCd"), None));
    }

    #[test]
    fn http_api_rejects_missing_authorization_header() {
        assert!(!authorized(&request("content-length: 0"), Some("AbCd")));
    }

    #[test]
    fn http_api_rejects_credential_case_change() {
        assert!(!authorized(
            &request("Authorization: Bearer abcd"),
            Some("AbCd")
        ));
    }

    fn request(header: &str) -> String {
        format!("POST / HTTP/1.1\r\n{header}\r\ncontent-length: 0\r\n\r\n")
    }
}
