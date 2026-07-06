package com.lkjmc.common.ui.binding;

import com.google.gson.JsonObject;
import com.lkjmc.common.ui.document.ItemRole;
import com.lkjmc.common.ui.kernel.EntryView;
import com.lkjmc.common.ui.kernel.RouteView;
import java.util.Comparator;
import java.util.List;

public final class PickerBindings {
    private PickerBindings() {}

    public static List<MenuBinding> bindings() {
        return List.of(new Picker("teleport-picker", "tpa"),
            new Picker("party-invite-picker", "party invite"),
            new Picker("claim-trust-picker", "claim trust"));
    }

    private static final class Picker extends BasicBinding {
        private final String command;
        Picker(String id, String command) {
            super(id, "local", List.of());
            this.command = command;
        }
        @Override public BindingResult decode(JsonObject body, BindingContext ctx) {
            var entries = ctx.local().onlinePlayers().stream()
                .filter(player -> !player.uuid().equals(ctx.playerUuid()))
                .filter(player -> !player.name().isBlank())
                .sorted(Comparator.comparing(LocalData.OnlinePlayer::name))
                .map(this::entry).toList();
            return entries.isEmpty() ? BindingResult.empty()
                : Views.data(new RouteView.ListView(entries, Views.keys("menu.player-picker.info.lore")));
        }
        private EntryView entry(LocalData.OnlinePlayer player) {
            return Views.entry("PLAYER_HEAD", Views.lit(player.name()),
                List.of(Views.lit(player.serverId()), Views.key("menu.player-picker.select.lore")),
                ItemRole.ACTION, Views.command(command + " " + player.name()));
        }
    }
}
