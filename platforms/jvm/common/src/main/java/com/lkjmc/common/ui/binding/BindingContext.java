package com.lkjmc.common.ui.binding;

import java.util.Map;
import java.util.Optional;
import java.util.TreeMap;

public record BindingContext(
    String playerUuid,
    String playerName,
    String locale,
    Map<String, String> params,
    PermissionsView permissions,
    LocalData local
) {
    public BindingContext {
        playerUuid = playerUuid == null ? "" : playerUuid;
        playerName = playerName == null ? "" : playerName;
        locale = locale == null || locale.isBlank() ? "en" : locale;
        params = Map.copyOf(params == null ? Map.of() : new TreeMap<>(params));
        permissions = permissions == null ? PermissionsView.none() : permissions;
        local = local == null ? LocalData.empty() : local;
    }

    public Optional<String> param(String name) {
        var value = params.get(name);
        return value == null || value.isBlank() ? Optional.empty() : Optional.of(value);
    }
}
