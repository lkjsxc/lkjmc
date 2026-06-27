package com.lkjmc.paper;

import com.lkjmc.common.menu.MenuActionPayload;
import com.lkjmc.common.menu.MenuId;
import com.lkjmc.common.menu.MenuMetadata;
import com.lkjmc.common.menu.MenuRoute;
import java.net.URLDecoder;
import java.net.URLEncoder;
import java.nio.charset.StandardCharsets;
import java.util.Arrays;
import java.util.Map;
import java.util.stream.Collectors;
import org.bukkit.NamespacedKey;
import org.bukkit.inventory.ItemStack;
import org.bukkit.inventory.meta.ItemMeta;
import org.bukkit.persistence.PersistentDataType;

final class MenuMetadataCodec {
    private final NamespacedKey menuKey;
    private final NamespacedKey routeKey;
    private final NamespacedKey routeParamsKey;
    private final NamespacedKey slotKey;
    private final NamespacedKey actionKey;
    private final NamespacedKey payloadKey;
    private final NamespacedKey sessionKey;
    private final NamespacedKey epochKey;
    private final NamespacedKey inertKey;

    MenuMetadataCodec(LkjmcPaperPlugin plugin) {
        menuKey = new NamespacedKey(plugin, "menu_id");
        routeKey = new NamespacedKey(plugin, "menu_route");
        routeParamsKey = new NamespacedKey(plugin, "menu_route_params");
        slotKey = new NamespacedKey(plugin, "menu_slot");
        actionKey = new NamespacedKey(plugin, "menu_action");
        payloadKey = new NamespacedKey(plugin, "menu_payload");
        sessionKey = new NamespacedKey(plugin, "menu_session");
        epochKey = new NamespacedKey(plugin, "menu_epoch");
        inertKey = new NamespacedKey(plugin, "menu_inert");
    }

    void write(ItemMeta meta, MenuMetadata metadata) {
        var pdc = meta.getPersistentDataContainer();
        pdc.set(menuKey, PersistentDataType.STRING, metadata.menuId().value());
        pdc.set(routeKey, PersistentDataType.STRING, metadata.route().id().value());
        pdc.set(routeParamsKey, PersistentDataType.STRING, encode(metadata.route().params()));
        pdc.set(slotKey, PersistentDataType.INTEGER, metadata.slot());
        pdc.set(actionKey, PersistentDataType.STRING, metadata.actionKey());
        pdc.set(payloadKey, PersistentDataType.STRING, metadata.payload().value());
        pdc.set(sessionKey, PersistentDataType.STRING, metadata.sessionId());
        pdc.set(epochKey, PersistentDataType.LONG, metadata.renderEpoch());
        if (metadata.inert()) {
            pdc.set(inertKey, PersistentDataType.BYTE, (byte) 1);
        }
    }

    boolean hasAny(ItemStack item) {
        return item != null && item.hasItemMeta()
            && item.getItemMeta().getPersistentDataContainer().has(menuKey, PersistentDataType.STRING);
    }

    MenuMetadata read(ItemStack item) {
        if (!hasAny(item)) {
            return null;
        }
        var pdc = item.getItemMeta().getPersistentDataContainer();
        var menu = pdc.get(menuKey, PersistentDataType.STRING);
        var route = pdc.get(routeKey, PersistentDataType.STRING);
        var routeParams = pdc.get(routeParamsKey, PersistentDataType.STRING);
        var slot = pdc.get(slotKey, PersistentDataType.INTEGER);
        var action = pdc.get(actionKey, PersistentDataType.STRING);
        var payload = pdc.get(payloadKey, PersistentDataType.STRING);
        var session = pdc.get(sessionKey, PersistentDataType.STRING);
        var epoch = pdc.get(epochKey, PersistentDataType.LONG);
        if (menu == null || route == null || slot == null || action == null || epoch == null) {
            return null;
        }
        return new MenuMetadata(new MenuId(menu), new MenuRoute(new MenuId(route), decode(routeParams)), slot, action,
            new MenuActionPayload(payload), session, epoch, pdc.has(inertKey, PersistentDataType.BYTE));
    }

    private static String encode(Map<String, String> params) {
        return params.entrySet().stream()
            .map(entry -> esc(entry.getKey()) + "=" + esc(entry.getValue()))
            .collect(Collectors.joining("&"));
    }

    private static Map<String, String> decode(String params) {
        if (params == null || params.isBlank()) {
            return Map.of();
        }
        return Arrays.stream(params.split("&"))
            .map(part -> part.split("=", 2))
            .filter(part -> part.length == 2)
            .collect(Collectors.toMap(part -> unesc(part[0]), part -> unesc(part[1])));
    }

    private static String esc(String value) {
        return URLEncoder.encode(value == null ? "" : value, StandardCharsets.UTF_8);
    }

    private static String unesc(String value) {
        return URLDecoder.decode(value, StandardCharsets.UTF_8);
    }
}
