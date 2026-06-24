package com.lkjmc.paper;

import com.lkjmc.common.daemon.DaemonClient;
import com.lkjmc.common.daemon.HttpDaemonClient;
import com.lkjmc.common.i18n.LocaleResolver;
import com.lkjmc.common.transfer.ProfileTransferMessages;
import com.lkjmc.common.i18n.MessageCatalog;
import java.util.Objects;
import java.util.Optional;
import org.bukkit.plugin.java.JavaPlugin;

public final class LkjmcPaperPlugin extends JavaPlugin {
    private SchedulerBridge scheduler;
    private MessageCatalog catalog;
    private Optional<DaemonClient> daemon = Optional.empty();

    @Override
    public void onEnable() {
        this.scheduler = new FoliaSchedulerBridge(this);
        this.catalog = MessageCatalog.fromResources("en", "en", "ja");
        this.daemon = HttpDaemonClient.fromEnv().map(client -> (DaemonClient) client);
        var resolver = new LocaleResolver("en");
        var menu = new MenuInventoryAdapter(catalog, resolver);
        var renderer = new com.lkjmc.common.i18n.MessageRenderer(catalog, resolver);
        var commands = new PaperCommands(this, menu, catalog, resolver);
        var hud = new HudDisplayService(this, renderer);
        Objects.requireNonNull(getCommand("lkjmc")).setExecutor(commands);
        Objects.requireNonNull(getCommand("menu")).setExecutor(commands);
        Objects.requireNonNull(getCommand("lang")).setExecutor(commands);
        Objects.requireNonNull(getCommand("points")).setExecutor(commands);
        Objects.requireNonNull(getCommand("sethome")).setExecutor(commands);
        Objects.requireNonNull(getCommand("home")).setExecutor(commands);
        Objects.requireNonNull(getCommand("setwarp")).setExecutor(commands);
        Objects.requireNonNull(getCommand("warp")).setExecutor(commands);
        Objects.requireNonNull(getCommand("tpa")).setExecutor(commands);
        Objects.requireNonNull(getCommand("tpaccept")).setExecutor(commands);
        Objects.requireNonNull(getCommand("achievements")).setExecutor(commands);
        Objects.requireNonNull(getCommand("hud")).setExecutor(commands);
        Objects.requireNonNull(getCommand("shop")).setExecutor(commands);
        Objects.requireNonNull(getCommand("buy")).setExecutor(commands);
        getServer().getPluginManager().registerEvents(new HotbarMenuListener(this, menu), this);
        getServer().getPluginManager().registerEvents(new PlayerLifecycleListener(this), this);
        getServer().getPluginManager().registerEvents(new TeleportArrivalListener(this), this);
        getServer().getPluginManager().registerEvents(hud, this);
        hud.start();
        getServer().getMessenger().registerIncomingPluginChannel(this,
            ProfileTransferMessages.CHANNEL, new ProfileTransferListener(this));
        getServer().getMessenger().registerOutgoingPluginChannel(this, ProfileTransferMessages.CHANNEL);
        new ServerHeartbeat(scheduler, daemon, System.getenv("LKJMC_INSTANCE_ID")).start();
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

    public Optional<DaemonClient> daemon() {
        return daemon;
    }

    public MessageCatalog catalog() {
        return catalog;
    }
}
