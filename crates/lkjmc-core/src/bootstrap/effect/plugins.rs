use super::BootstrapEffect;
use crate::bootstrap::desired::DesiredInstance;
use crate::bootstrap::plugin::PluginId;

pub(super) fn install_plugins(
    effects: &mut Vec<BootstrapEffect>,
    desired: &DesiredInstance,
    required_plugin: PluginId,
    optional_plugins: &[PluginId],
) {
    effects.push(BootstrapEffect::InstallPlugin {
        id: desired.id.clone(),
        plugin: required_plugin,
    });
    for plugin in optional_plugins {
        effects.push(BootstrapEffect::InstallPlugin {
            id: desired.id.clone(),
            plugin: *plugin,
        });
    }
}
