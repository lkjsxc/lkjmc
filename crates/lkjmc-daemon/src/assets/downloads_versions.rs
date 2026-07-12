use std::cmp::Reverse;
use std::collections::BTreeSet;

pub(crate) const DEFAULT_JAVA21_RELEASE: &str = "1.21.11";
pub(crate) const DEFAULT_JAVA21_VELOCITY_RELEASE: &str = "3.4.0-SNAPSHOT";

pub(crate) fn candidate_versions(
    project: &str,
    explicit: Option<&str>,
    available: Vec<String>,
) -> Result<Vec<String>, String> {
    if let Some(version) = explicit {
        if project == "velocity" && !is_java21_velocity(version) {
            return Err(format!(
                "Velocity {version} is incompatible with the Java 21 runtime; use {DEFAULT_JAVA21_VELOCITY_RELEASE}"
            ));
        }
        return Ok(vec![version.to_string()]);
    }
    Ok(match project {
        "paper" | "folia" => java21_candidates(available),
        "velocity" => java21_velocity_candidates(available),
        _ => newest_first(available),
    })
}

fn java21_candidates(available: Vec<String>) -> Vec<String> {
    let mut values = vec![DEFAULT_JAVA21_RELEASE.to_string()];
    values.extend(
        newest_first(available)
            .into_iter()
            .filter(|value| is_java21(value)),
    );
    dedupe(values)
}

fn java21_velocity_candidates(available: Vec<String>) -> Vec<String> {
    let mut values = vec![DEFAULT_JAVA21_VELOCITY_RELEASE.to_string()];
    values.extend(
        available
            .into_iter()
            .filter(|value| is_java21_velocity(value)),
    );
    dedupe(values)
}

pub(crate) fn newest_first(mut values: Vec<String>) -> Vec<String> {
    values.sort_by_key(|value| Reverse(version_key(value)));
    values
}

fn is_java21(value: &str) -> bool {
    !value.contains('-') && (value == "1.21" || value.starts_with("1.21."))
}

fn is_java21_velocity(value: &str) -> bool {
    value == DEFAULT_JAVA21_VELOCITY_RELEASE
}

fn dedupe(values: Vec<String>) -> Vec<String> {
    let mut seen = BTreeSet::new();
    values
        .into_iter()
        .filter(|value| seen.insert(value.clone()))
        .collect()
}

fn version_key(value: &str) -> Vec<i64> {
    value
        .split(['.', '-'])
        .map(|part| part.parse::<i64>().unwrap_or(-1))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_version_is_only_candidate() -> Result<(), String> {
        assert_eq!(
            candidate_versions("folia", Some("1.20.6"), vec![])?,
            vec!["1.20.6"]
        );
        Ok(())
    }

    #[test]
    fn java21_candidates_prefer_default_then_available_stable_releases() -> Result<(), String> {
        let versions = vec!["26.2", "1.21.8", "1.21.11-rc1", "1.21.6", "1.20.6"]
            .into_iter()
            .map(ToString::to_string)
            .collect();
        assert_eq!(
            candidate_versions("folia", None, versions)?,
            vec!["1.21.11", "1.21.8", "1.21.6"]
        );
        Ok(())
    }

    #[test]
    fn velocity_stays_on_the_java21_stream() -> Result<(), String> {
        let versions = vec!["4.0.0-SNAPSHOT", "3.5.0-SNAPSHOT", "3.4.0-SNAPSHOT"]
            .into_iter()
            .map(ToString::to_string)
            .collect();
        assert_eq!(
            candidate_versions("velocity", None, versions)?,
            vec!["3.4.0-SNAPSHOT"]
        );
        Ok(())
    }

    #[test]
    fn velocity_rejects_an_incompatible_explicit_stream() -> Result<(), String> {
        let error = match candidate_versions("velocity", Some("3.5.0-SNAPSHOT"), vec![]) {
            Ok(_) => return Err("Velocity 3.5 must not bypass Java 21 compatibility".to_string()),
            Err(error) => error,
        };
        assert!(error.contains(DEFAULT_JAVA21_VELOCITY_RELEASE));
        Ok(())
    }
}
