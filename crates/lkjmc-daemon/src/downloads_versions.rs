use std::cmp::Reverse;
use std::collections::BTreeSet;

pub(crate) const DEFAULT_JAVA21_RELEASE: &str = "1.21.11";

pub(crate) fn candidate_versions(
    project: &str,
    explicit: Option<&str>,
    available: Vec<String>,
) -> Vec<String> {
    if let Some(version) = explicit {
        return vec![version.to_string()];
    }
    match project {
        "paper" | "folia" => java21_candidates(available),
        _ => newest_first(available),
    }
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

pub(crate) fn newest_first(mut values: Vec<String>) -> Vec<String> {
    values.sort_by_key(|value| Reverse(version_key(value)));
    values
}

fn is_java21(value: &str) -> bool {
    !value.contains('-') && (value == "1.21" || value.starts_with("1.21."))
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
    fn explicit_version_is_only_candidate() {
        assert_eq!(
            candidate_versions("folia", Some("1.20.6"), vec![]),
            vec!["1.20.6"]
        );
    }

    #[test]
    fn java21_candidates_prefer_default_then_available_stable_releases() {
        let versions = vec!["26.2", "1.21.8", "1.21.11-rc1", "1.21.6", "1.20.6"]
            .into_iter()
            .map(ToString::to_string)
            .collect();
        assert_eq!(
            candidate_versions("folia", None, versions),
            vec!["1.21.11", "1.21.8", "1.21.6"]
        );
    }

    #[test]
    fn non_java_projects_use_newest_version_order() {
        let versions = vec!["3.4.0", "3.5.0-SNAPSHOT", "3.1.1"]
            .into_iter()
            .map(ToString::to_string)
            .collect();
        assert_eq!(
            candidate_versions("velocity", None, versions)[0],
            "3.5.0-SNAPSHOT"
        );
    }
}
