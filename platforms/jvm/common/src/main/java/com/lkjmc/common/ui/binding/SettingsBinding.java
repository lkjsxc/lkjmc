package com.lkjmc.common.ui.binding;

import com.google.gson.JsonObject;
import com.lkjmc.common.ui.document.ItemRole;
import com.lkjmc.common.ui.kernel.FrameSlot;
import com.lkjmc.common.ui.kernel.RouteView;
import java.util.List;
import java.util.Map;

public final class SettingsBinding extends BasicBinding {
    public SettingsBinding() {
        super("settings", "daemon", List.of("player.settings.get"));
    }

    @Override
    public BindingResult decode(JsonObject body, BindingContext ctx) {
        var uuid = Jsons.string(body, "playerUuid", id());
        var language = Jsons.string(body, "language", id());
        var hud = Jsons.bool(body, "hudEnabled", id());
        var menu = Jsons.bool(body, "menuEnabled", id());
        var slots = List.of(
            Views.keyedSlot(20, "BOOK", "menu.language.title", ItemRole.NAVIGATION,
                Views.open("language"), ctx.params(), "menu.language.lore"),
            toggle(22, "CLOCK", "menu.hud.toggle", "hud", hud,
                hud ? "hud.disabled" : "hud.enabled", "hud.failed", ctx),
            toggle(24, "NETHER_STAR", "menu.hotbar-token.toggle", "menu-token", menu,
                menu ? "hotbar.menu.disabled" : "hotbar.menu.enabled", "hotbar.menu.failed", ctx));
        return Views.data(new RouteView.DetailView(slots, List.of(Views.lit(uuid), Views.lit(language),
            Views.key(hud ? "hud.enabled" : "hud.disabled"),
            Views.key(menu ? "hotbar.menu.enabled" : "hotbar.menu.disabled"))));
    }

    private FrameSlot toggle(int slot, String material, String key, String setting, boolean current,
                             String ok, String fail, BindingContext ctx) {
        var state = setting.equals("hud")
            ? current ? "hud.enabled" : "hud.disabled"
            : current ? "hotbar.menu.enabled" : "hotbar.menu.disabled";
        var lore = List.of(Views.key(state), Views.key(setting.equals("hud")
            ? "menu.hud.toggle.lore" : "menu.hotbar-token.toggle.lore"));
        var body = Map.of("playerUuid", ctx.playerUuid(), "name", ctx.playerName(), "settingKey", setting);
        return Views.slot(slot, material, Views.key(key), lore, ItemRole.ACTION,
            Views.daemon("player.settings.toggle", body, ok, fail, true), ctx.params());
    }
}
