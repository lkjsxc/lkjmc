package com.lkjmc.paper;

import com.lkjmc.common.i18n.LocaleResolver;
import com.lkjmc.common.i18n.MessageCatalog;
import java.util.Objects;
import org.bukkit.plugin.java.JavaPlugin;

public final class LkjmcPaperPlugin extends JavaPlugin {
    private SchedulerBridge scheduler;
    private MessageCatalog catalog;

    @Override
    public void onEnable() {
        this.scheduler = new FoliaSchedulerBridge(this);
        this.catalog = MessageCatalog.fromResources("en", "en", "ja");
        var resolver = new LocaleResolver("en");
        var menu = new MenuInventoryAdapter(catalog, resolver);
        Objects.requireNonNull(getCommand("lkjmc")).setExecutor(new PaperCommands(this, menu));
        Objects.requireNonNull(getCommand("menu")).setExecutor(new PaperCommands(this, menu));
        getLogger().info("lkjmc Paper plugin enabled");
    }

    @Override
    public void onDisable() {
        if (scheduler != null) {
            scheduler.cancelAll();
        }
    }

    public SchedulerBridge scheduler() {
        return scheduler;
    }

    public MessageCatalog catalog() {
        return catalog;
    }
}
