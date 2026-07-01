package com.lkjmc.paper;

import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonClient;
import com.lkjmc.common.daemon.DaemonJson;
import com.lkjmc.common.daemon.DaemonRequest;
import com.lkjmc.common.i18n.LocaleResolver;
import com.google.gson.JsonObject;
import java.util.Map;
import java.util.Optional;
import java.util.UUID;
import java.util.concurrent.ConcurrentHashMap;
import org.bukkit.entity.Player;
import org.bukkit.event.EventHandler;
import org.bukkit.event.Listener;
import org.bukkit.event.player.PlayerJoinEvent;
import org.bukkit.event.player.PlayerQuitEvent;

public final class PlayerLocaleService implements Listener {
    private final LkjmcPaperPlugin plugin;
    private final LocaleResolver resolver;
    private final Optional<DaemonClient> daemon;
    private final ConcurrentHashMap<UUID, String> persisted = new ConcurrentHashMap<>();

    PlayerLocaleService(LkjmcPaperPlugin plugin, LocaleResolver resolver, Optional<DaemonClient> daemon) {
        this.plugin = plugin;
        this.resolver = resolver;
        this.daemon = daemon == null ? Optional.empty() : daemon;
    }

    public String locale(Player player) {
        return resolver.resolve(Optional.ofNullable(persisted.get(player.getUniqueId())), platform(player));
    }

    public void updateFromResponse(Player player, JsonObject body) {
        DaemonJson.string(body, "language").ifPresent(language -> update(player, language));
    }

    @EventHandler
    public void onJoin(PlayerJoinEvent event) {
        load(event.getPlayer());
    }

    @EventHandler
    public void onQuit(PlayerQuitEvent event) {
        persisted.remove(event.getPlayer().getUniqueId());
    }

    void update(Player player, String language) {
        persisted.put(player.getUniqueId(), resolver.resolve(Optional.ofNullable(language), Optional.empty()));
    }

    void load(Player player) {
        daemon.ifPresent(client -> client.send(request(player)).thenAccept(response -> {
            if (response.ok()) {
                plugin.scheduler().runPlayer(player, () -> updateFromResponse(player, response.body()));
            }
        }));
    }

    private Optional<String> platform(Player player) {
        return Optional.ofNullable(player.locale()).map(java.util.Locale::toLanguageTag);
    }

    private static DaemonRequest request(Player player) {
        return new DaemonRequest(UUID.randomUUID(), new DaemonActor("paper-plugin", instanceId()),
            "player.settings.get", Map.of("playerUuid", player.getUniqueId().toString()));
    }

    private static String instanceId() {
        return System.getenv().getOrDefault("LKJMC_INSTANCE_ID", "paper");
    }
}
