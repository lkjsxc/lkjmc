use crate::id::InstanceId;
use crate::validation::is_kebab_id;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LaunchSource {
    JarAsset(String),
    Command,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatePlanInput {
    pub id: String,
    pub kind: String,
    pub template: String,
    pub launch_source: Option<LaunchSource>,
    pub memory_mb: Option<i64>,
    pub server_port: Option<i64>,
    pub accept_minecraft_eula: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartableInstancePlan {
    pub id: String,
    pub kind: String,
    pub template: String,
    pub launch_source: LaunchSource,
    pub memory_mb: i64,
    pub server_port: Option<i64>,
    pub eula_accepted: bool,
}

pub fn plan_startable(input: CreatePlanInput) -> Result<StartableInstancePlan, Vec<String>> {
    let mut errors = Vec::new();
    if let Err(error) = InstanceId::parse(input.id.clone()) {
        errors.push(error.to_string());
    }
    if !is_known_kind(&input.kind) {
        errors.push(format!("unsupported instance kind: {}", input.kind));
    }
    if !is_kebab_id(&input.template) {
        errors.push("invalid template id".to_string());
    }
    let memory_mb = input.memory_mb.unwrap_or(2048);
    if !(256..=65536).contains(&memory_mb) {
        errors.push("memoryMb must be between 256 and 65536".to_string());
    }
    if let Some(port) = input.server_port {
        if !(1..=65535).contains(&port) {
            errors.push("serverPort must be between 1 and 65535".to_string());
        }
    }
    if input.launch_source.is_none() {
        errors.push(
            "missing launch source: sync/import a jar asset or provide launch command".to_string(),
        );
    }
    if requires_eula(&input.kind) && !input.accept_minecraft_eula {
        errors.push("missing Minecraft EULA acknowledgement".to_string());
    }
    if !errors.is_empty() {
        return Err(errors);
    }
    let Some(launch_source) = input.launch_source else {
        return Err(vec!["missing launch source".to_string()]);
    };
    Ok(StartableInstancePlan {
        id: input.id,
        kind: input.kind,
        template: input.template,
        launch_source,
        memory_mb,
        server_port: input.server_port,
        eula_accepted: input.accept_minecraft_eula,
    })
}

pub fn requires_eula(kind: &str) -> bool {
    matches!(
        kind,
        "paper" | "folia" | "purpur" | "vanilla-custom" | "modded-custom"
    )
}

fn is_known_kind(kind: &str) -> bool {
    matches!(
        kind,
        "velocity" | "paper" | "folia" | "purpur" | "vanilla-custom" | "modded-custom"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> CreatePlanInput {
        CreatePlanInput {
            id: "hub".to_string(),
            kind: "paper".to_string(),
            template: "paper-survival".to_string(),
            launch_source: Some(LaunchSource::JarAsset("asset-1".to_string())),
            memory_mb: None,
            server_port: None,
            accept_minecraft_eula: true,
        }
    }

    #[test]
    fn defaults_memory_for_startable_server() {
        let result = plan_startable(input());
        assert_eq!(result.as_ref().map(|plan| plan.memory_mb), Ok(2048));
        assert_eq!(result.as_ref().map(|plan| plan.eula_accepted), Ok(true));
    }

    #[test]
    fn rejects_missing_launch_source_and_eula() {
        let mut input = input();
        input.launch_source = None;
        input.accept_minecraft_eula = false;
        let result = plan_startable(input);
        assert!(
            matches!(&result, Err(errors) if errors.iter().any(|error| error.contains("launch source")))
        );
        assert!(
            matches!(&result, Err(errors) if errors.iter().any(|error| error.contains("EULA")))
        );
    }

    #[test]
    fn velocity_does_not_require_minecraft_eula() {
        let mut input = input();
        input.kind = "velocity".to_string();
        input.template = "velocity-modern".to_string();
        input.accept_minecraft_eula = false;
        assert!(plan_startable(input).is_ok());
    }
}
