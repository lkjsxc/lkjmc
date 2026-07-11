package com.lkjmc.paper;

import java.util.Objects;
import org.bukkit.plugin.java.JavaPlugin;

public final class LkjmcPaperPlugin extends JavaPlugin {
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
        getLogger().info("lkjmc local documentation UI enabled");
    }
}
