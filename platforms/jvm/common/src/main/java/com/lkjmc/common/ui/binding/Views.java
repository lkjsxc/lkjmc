package com.lkjmc.common.ui.binding;

import com.lkjmc.common.ui.document.DocumentAction;
import com.lkjmc.common.ui.document.ItemRole;
import com.lkjmc.common.ui.kernel.EntryView;
import com.lkjmc.common.ui.kernel.FrameSlot;
import com.lkjmc.common.ui.kernel.RouteView;
import com.lkjmc.common.ui.kernel.TextRef;
import java.util.Arrays;
import java.util.List;
import java.util.Map;

final class Views {
    private Views() {}

    static EntryView entry(String material, TextRef name, List<TextRef> lore,
                           ItemRole role, DocumentAction action) {
        return new EntryView(material, name, lore, role, action);
    }

    static EntryView keyed(String material, String key, DocumentAction action,
                           ItemRole role, String... loreKeys) {
        return entry(material, key(key), keys(loreKeys), role, action);
    }

    static FrameSlot slot(int slot, String material, TextRef name, List<TextRef> lore,
                          ItemRole role, DocumentAction action, Map<String, String> params) {
        if (role.inertByRole() || action instanceof DocumentAction.None) {
            return FrameSlot.inert(slot, material, name, lore, role);
        }
        return FrameSlot.action(slot, material, name, lore, role, action, params);
    }

    static FrameSlot keyedSlot(int slot, String material, String key, ItemRole role,
                               DocumentAction action, Map<String, String> params,
                               String... loreKeys) {
        return slot(slot, material, key(key), keys(loreKeys), role, action, params);
    }

    static TextRef key(String key) {
        return TextRef.key(key);
    }

    static TextRef key(String key, Map<String, String> args) {
        return TextRef.key(key, args);
    }

    static TextRef lit(Object value) {
        return TextRef.literal(value == null ? "" : value.toString());
    }

    static List<TextRef> keys(String... keys) {
        return Arrays.stream(keys == null ? new String[0] : keys).map(TextRef::key).toList();
    }

    static DocumentAction open(String route) {
        return new DocumentAction.Open(route, Map.of());
    }

    static DocumentAction open(String route, Map<String, String> params) {
        return new DocumentAction.Open(route, params);
    }

    static DocumentAction command(String value) {
        return new DocumentAction.Command(value);
    }

    static DocumentAction disabled(String reason) {
        return new DocumentAction.Disabled(reason);
    }

    static DocumentAction daemon(String command, Map<String, String> body, String ok,
                                 String fail, boolean refresh) {
        return new DocumentAction.Daemon(command, body, ok, fail, refresh);
    }

    static BindingResult data(RouteView view) {
        return BindingResult.data(view);
    }
}
