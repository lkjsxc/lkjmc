package com.lkjmc.paper.ui;

import com.lkjmc.common.ui.kernel.MenuMetadata;
import com.lkjmc.common.ui.kernel.MenuRoute;
import java.net.URLDecoder;
import java.net.URLEncoder;
import java.nio.charset.StandardCharsets;
import java.util.Arrays;
import java.util.Map;
import java.util.TreeMap;
import java.util.stream.Collectors;
import org.bukkit.NamespacedKey;
import org.bukkit.inventory.ItemStack;
import org.bukkit.inventory.meta.ItemMeta;
import org.bukkit.persistence.PersistentDataContainer;
import org.bukkit.persistence.PersistentDataType;
import org.bukkit.plugin.Plugin;

public final class UiMetadataCodec {
    private final NamespacedKey markerKey;
    private final NamespacedKey routeKey;
    private final NamespacedKey paramsKey;
    private final NamespacedKey slotKey;
    private final NamespacedKey actionKey;
    private final NamespacedKey payloadKey;
    private final NamespacedKey sessionKey;
    private final NamespacedKey epochKey;

    public UiMetadataCodec(Plugin plugin) {
        this(plugin.getName().toLowerCase(java.util.Locale.ROOT));
    }

    UiMetadataCodec(String namespace) {
        markerKey = key(namespace, "ui_marker");
        routeKey = key(namespace, "ui_route");
        paramsKey = key(namespace, "ui_route_params");
        slotKey = key(namespace, "ui_slot");
        actionKey = key(namespace, "ui_action");
        payloadKey = key(namespace, "ui_payload");
        sessionKey = key(namespace, "ui_session");
        epochKey = key(namespace, "ui_epoch");
    }

    public void write(ItemMeta meta, MenuMetadata metadata) {
        write(meta.getPersistentDataContainer(), metadata);
    }

    void write(PersistentDataContainer pdc, MenuMetadata metadata) {
        pdc.set(markerKey, PersistentDataType.BYTE, (byte) 1);
        pdc.set(routeKey, PersistentDataType.STRING, metadata.route().id());
        pdc.set(paramsKey, PersistentDataType.STRING, encode(metadata.params()));
        pdc.set(slotKey, PersistentDataType.INTEGER, metadata.slot());
        pdc.set(actionKey, PersistentDataType.STRING, metadata.actionKey());
        pdc.set(payloadKey, PersistentDataType.STRING, encode(metadata.payload()));
        pdc.set(sessionKey, PersistentDataType.STRING, metadata.sessionId());
        pdc.set(epochKey, PersistentDataType.LONG, metadata.epoch());
    }

    public boolean hasAny(ItemStack item) {
        return item != null && item.hasItemMeta() && hasAny(item.getItemMeta().getPersistentDataContainer());
    }

    boolean hasAny(PersistentDataContainer pdc) {
        return pdc.has(markerKey, PersistentDataType.BYTE);
    }

    public MenuMetadata read(ItemStack item) {
        if (!hasAny(item)) {
            return null;
        }
        return read(item.getItemMeta().getPersistentDataContainer());
    }

    MenuMetadata read(PersistentDataContainer pdc) {
        if (!hasAny(pdc)) {
            return null;
        }
        try {
            var route = pdc.get(routeKey, PersistentDataType.STRING);
            var slot = pdc.get(slotKey, PersistentDataType.INTEGER);
            var action = pdc.get(actionKey, PersistentDataType.STRING);
            var session = pdc.get(sessionKey, PersistentDataType.STRING);
            var epoch = pdc.get(epochKey, PersistentDataType.LONG);
            if (route == null || slot == null || action == null || session == null || epoch == null) {
                return null;
            }
            var params = decode(pdc.get(paramsKey, PersistentDataType.STRING));
            var payload = decode(pdc.get(payloadKey, PersistentDataType.STRING));
            return new MenuMetadata(new MenuRoute(route, params), params, slot, action, payload, session, epoch);
        } catch (RuntimeException error) {
            return null;
        }
    }

    private static NamespacedKey key(String namespace, String value) {
        return new NamespacedKey(namespace, value);
    }

    private static String encode(Map<String, String> values) {
        return new TreeMap<>(values == null ? Map.of() : values).entrySet().stream()
            .map(entry -> esc(entry.getKey()) + "=" + esc(entry.getValue()))
            .collect(Collectors.joining("&"));
    }

    private static Map<String, String> decode(String value) {
        if (value == null || value.isBlank()) {
            return Map.of();
        }
        var decoded = new TreeMap<String, String>();
        Arrays.stream(value.split("&")).map(part -> part.split("=", 2))
            .filter(part -> part.length == 2)
            .forEach(part -> decoded.put(unesc(part[0]), unesc(part[1])));
        return Map.copyOf(decoded);
    }

    private static String esc(String value) {
        return URLEncoder.encode(value == null ? "" : value, StandardCharsets.UTF_8);
    }

    private static String unesc(String value) {
        return URLDecoder.decode(value, StandardCharsets.UTF_8);
    }
}
