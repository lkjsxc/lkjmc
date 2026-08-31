pub fn is_absolute_path(value: &str) -> bool {
    value.starts_with('/') && value.len() > 1
}

pub fn is_kebab_id(value: &str) -> bool {
    if value.is_empty() || value.len() > 63 {
        return false;
    }
    let mut previous_dash = false;
    let mut has_character = false;
    for character in value.chars() {
        let valid =
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-';
        if !valid {
            return false;
        }
        if character == '-' {
            if previous_dash || !has_character {
                return false;
            }
            previous_dash = true;
        } else {
            previous_dash = false;
            has_character = true;
        }
    }
    has_character && !previous_dash
}

pub fn is_non_empty(value: &str) -> bool {
    !value.trim().is_empty()
}

pub fn is_sha256(value: &str) -> bool {
    value.len() == 64 && value.chars().all(|character| character.is_ascii_hexdigit())
}

pub fn is_valid_port(port: u16) -> bool {
    port > 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_kebab_ids() {
        assert!(is_kebab_id("hub"));
        assert!(is_kebab_id("survival-one"));
        assert!(!is_kebab_id("Hub"));
        assert!(!is_kebab_id("-hub"));
        assert!(!is_kebab_id("hub-"));
        assert!(!is_kebab_id("hub--one"));
        assert!(!is_kebab_id(&"a".repeat(64)));
    }
}
