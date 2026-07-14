package com.lkjmc.bindings;

import java.util.List;

public record ProfileEnvelope(
        String schema,
        List<ProfileSlot> inventory,
        List<ProfileSlot> armor,
        ProfileItem offhand,
        int selectedHotbarSlot,
        List<ProfileSlot> enderChest,
        Experience experience,
        Vitals vitals,
        List<PotionEffect> potionEffects,
        GameMode gameMode,
        List<PluginDatum> pluginData,
        List<SavedLocation> homes,
        List<SavedLocation> warps,
        long points,
        List<String> achievements,
        ProfileSettings settings,
        String language
) {
    public ProfileEnvelope {
        java.util.Objects.requireNonNull(schema, "schema");
        if (schema.isBlank()) throw new IllegalArgumentException("schema");
        inventory = List.copyOf(inventory);
        armor = List.copyOf(armor);
        enderChest = List.copyOf(enderChest);
        java.util.Objects.requireNonNull(experience, "experience");
        java.util.Objects.requireNonNull(vitals, "vitals");
        potionEffects = List.copyOf(potionEffects);
        pluginData = List.copyOf(pluginData);
        homes = List.copyOf(homes);
        warps = List.copyOf(warps);
        achievements = List.copyOf(achievements);
        java.util.Objects.requireNonNull(settings, "settings");
        java.util.Objects.requireNonNull(language, "language");
        if (language.isBlank()) throw new IllegalArgumentException("language");
    }
}
