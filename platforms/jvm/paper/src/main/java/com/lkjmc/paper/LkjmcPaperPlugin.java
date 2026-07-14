package com.lkjmc.paper;

import com.lkjmc.common.sync.SyncBootstrap;
import com.lkjmc.common.sync.SyncCoordinator;
import com.lkjmc.common.sync.SyncKey;
import java.util.Objects;
import java.util.Optional;
import org.bukkit.plugin.java.JavaPlugin;

public final class LkjmcPaperPlugin extends JavaPlugin {
    private Optional<SyncCoordinator> coordinator = Optional.empty();

    @Override
    public void onEnable() {
        var docs = new LocalDocsMenu(this);
        var tokens = new HotbarMenuTokenService(this);
        var sync = new InventorySyncService(tokens);
        var commands = new DocsCommandAdapter(docs);
        Objects.requireNonNull(getCommand("menu")).setExecutor(commands);
        Objects.requireNonNull(getCommand("docs")).setExecutor(commands);
        getServer().getPluginManager().registerEvents(docs, this);
        getServer().getPluginManager().registerEvents(new HotbarMenuListener(docs, tokens, sync), this);
        coordinator = SyncBootstrap.fromEnvironment(System.getenv());
        coordinator.ifPresent(value -> value.subscribe(new SyncKey("menus", "global")));
        getLogger().info("lkjmc local documentation UI enabled");
    }

    @Override
    public void onDisable() {
        coordinator.ifPresent(SyncCoordinator::close);
        coordinator = Optional.empty();
    }
}
