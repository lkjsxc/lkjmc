pub fn next_default<F>(player_name: &str, mut exists: F) -> Result<String, String>
where
    F: FnMut(&str) -> Result<bool, String>,
{
    for suffix in 1..=1000 {
        let candidate = default_name(player_name, suffix);
        if !exists(&candidate)? {
            return Ok(candidate);
        }
    }
    Err("no available generated party name".to_string())
}

fn default_name(player_name: &str, suffix: usize) -> String {
    let owner = safe_owner(player_name);
    if suffix == 1 {
        format!("{owner}'s Party")
    } else {
        format!("{owner}'s Party {suffix}")
    }
}

fn safe_owner(player_name: &str) -> String {
    let safe: String = player_name
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-'))
        .take(24)
        .collect();
    if safe.is_empty() {
        "Player".to_string()
    } else {
        safe
    }
}

#[cfg(test)]
mod tests {
    use super::next_default;

    #[test]
    fn generates_duplicate_free_party_names() -> Result<(), String> {
        let taken = ["Alex's Party", "Alex's Party 2"];
        let name = next_default("Alex", |candidate| Ok(taken.contains(&candidate)))?;
        assert_eq!(name, "Alex's Party 3");
        Ok(())
    }

    #[test]
    fn sanitizes_blank_or_unsafe_player_names() -> Result<(), String> {
        let name = next_default("<?> ", |_| Ok(false))?;
        assert_eq!(name, "Player's Party");
        Ok(())
    }
}
