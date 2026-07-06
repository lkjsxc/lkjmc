package com.lkjmc.paper.ui;

import com.lkjmc.common.ui.kernel.TextRef;
import com.lkjmc.common.ui.kernel.UiMsg;
import com.lkjmc.paper.SchedulerBridge;
import java.time.Duration;
import java.time.Instant;
import java.util.Map;
import java.util.UUID;
import java.util.concurrent.ConcurrentHashMap;
import java.util.function.Function;
import org.bukkit.entity.Player;
import org.bukkit.event.EventHandler;
import org.bukkit.event.Listener;
import org.bukkit.event.player.AsyncPlayerChatEvent;
import org.bukkit.event.player.PlayerQuitEvent;

public final class UiTextInput implements Listener {
    private static final Duration INPUT_TTL = Duration.ofSeconds(60);
    private final SchedulerBridge scheduler;
    private final UiText text;
    private final Function<Player, String> locales;
    private final Map<UUID, Pending> pending = new ConcurrentHashMap<>();

    public UiTextInput(SchedulerBridge scheduler, UiText text, Function<Player, String> locales) {
        this.scheduler = scheduler;
        this.text = text;
        this.locales = locales;
    }

    void start(Player player, TextRef prompt, String commandPrefix, UiSessionService sessions) {
        var input = new Pending(commandPrefix, UUID.randomUUID(), Instant.now().plus(INPUT_TTL), sessions);
        pending.put(player.getUniqueId(), input);
        player.sendMessage(text.chat(locale(player), prompt));
        player.sendMessage(text.chat(locale(player), TextRef.key("menu.input.cancel.lore")));
        scheduler.runPlayerLater(player, () -> expire(player, input), INPUT_TTL);
    }

    @EventHandler
    public void onChat(AsyncPlayerChatEvent event) {
        var input = pending.remove(event.getPlayer().getUniqueId());
        if (input == null) {
            return;
        }
        if (input.expired()) {
            scheduler.runPlayer(event.getPlayer(), () -> message(event.getPlayer(), "menu.input.expired"));
            return;
        }
        event.setCancelled(true);
        var value = event.getMessage() == null ? "" : event.getMessage().trim();
        scheduler.runPlayer(event.getPlayer(), () -> handle(event.getPlayer(), input, value));
    }

    @EventHandler
    public void onQuit(PlayerQuitEvent event) {
        pending.remove(event.getPlayer().getUniqueId());
    }

    private void expire(Player player, Pending input) {
        if (pending.remove(player.getUniqueId(), input)) {
            message(player, "menu.input.expired");
        }
    }

    private void handle(Player player, Pending input, String value) {
        if (value.equalsIgnoreCase("cancel")) {
            message(player, "menu.input.cancelled");
            return;
        }
        if (value.isBlank()) {
            message(player, "menu.input.invalid");
            return;
        }
        if (input.commandPrefix().contains("{input}")) {
            input.sessions().dispatch(player,
                new UiMsg.TextSubmitted("", input.commandPrefix().replace("{input}", value)));
        } else {
            input.sessions().dispatch(player, new UiMsg.TextSubmitted(value, input.commandPrefix()));
        }
    }

    private void message(Player player, String key) {
        player.sendMessage(text.chat(locale(player), TextRef.key(key)));
    }

    private String locale(Player player) {
        return locales == null ? "en" : locales.apply(player);
    }

    private record Pending(String commandPrefix, UUID token, Instant expiresAt,
                           UiSessionService sessions) {
        Pending {
            commandPrefix = commandPrefix == null ? "" : commandPrefix;
        }
        boolean expired() {
            return Instant.now().isAfter(expiresAt);
        }
    }
}
