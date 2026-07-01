package com.lkjmc.paper;

import com.lkjmc.common.claim.ClaimCache;
import com.lkjmc.common.config.RuntimeConfigValidator;
import com.lkjmc.common.daemon.DaemonClient;
import com.lkjmc.common.daemon.HttpDaemonClient;
import com.lkjmc.common.i18n.LocaleResolver;
import com.lkjmc.common.i18n.MessageCatalog;
import com.lkjmc.common.permission.PermissionSnapshotCache;
import com.lkjmc.common.transfer.ProfileTransferMessages;
import java.util.Objects;
import java.util.Optional;
import org.bukkit.plugin.java.JavaPlugin;

public final class LkjmcPaperPlugin extends JavaPlugin {
    private SchedulerBridge scheduler;
    private MessageCatalog catalog;
    private PlayerLocaleService locales;
    private Optional<DaemonClient> daemon = Optional.empty();
    private PermissionSnapshotCache adminGrants = PermissionSnapshotCache.disabled();
    private final ClaimCache claims = new ClaimCache();

    @Override
    public void onEnable() {
        this.scheduler = new FoliaSchedulerBridge(this);
        this.catalog = MessageCatalog.fromResources("en", "en", "ja");
        var runtimeConfig = RuntimeConfigValidator.fromEnv();
        if (!runtimeConfig.valid()) {
            getLogger().warning("lkjmc runtime config invalid: " + runtimeConfig.code());
        }
        this.daemon = runtimeConfig.valid() ? HttpDaemonClient.fromEnv().map(client -> (DaemonClient) client) : Optional.empty();
        this.adminGrants = daemon.map(client -> new PermissionSnapshotCache(client,
            "paper-plugin", instanceId())).orElseGet(PermissionSnapshotCache::disabled);
        var resolver = new LocaleResolver("en");
        this.locales = new PlayerLocaleService(this, resolver, daemon);
        var token = new HotbarMenuTokenService(this, catalog);
        var inventorySync = new InventorySyncService(this, token, daemon);
        var renderer = new com.lkjmc.common.i18n.MessageRenderer(catalog, resolver);
        var textInput = new MenuTextInputService(this, renderer);
        var menu = new MenuInventoryAdapter(this, catalog, resolver, inventorySync, textInput);
        var commands = new PaperCommands(this, menu, catalog, resolver);
        var docs = new DocsCommandAdapter(this, renderer, player -> menu.openRoot(player));
        var actionBars = new PassiveActionBarService(this, renderer);
        var claimSnapshots = new ClaimSnapshotService(this, claims);
        var randomTeleports = new RandomTeleportService(this, renderer);
        var endReturns = new EndExpeditionReturnService(this, renderer);
        var lkjmc = Objects.requireNonNull(getCommand("lkjmc"));
        lkjmc.setExecutor(commands);
        lkjmc.setTabCompleter(new PaperLkjmcTabCompleter(this));
        Objects.requireNonNull(getCommand("menu")).setExecutor(commands);
        Objects.requireNonNull(getCommand("lang")).setExecutor(commands);
        Objects.requireNonNull(getCommand("points")).setExecutor(commands);
        Objects.requireNonNull(getCommand("sethome")).setExecutor(commands);
        Objects.requireNonNull(getCommand("home")).setExecutor(commands);
        Objects.requireNonNull(getCommand("setwarp")).setExecutor(commands);
        Objects.requireNonNull(getCommand("warp")).setExecutor(commands);
        Objects.requireNonNull(getCommand("tpa")).setExecutor(commands);
        Objects.requireNonNull(getCommand("tpaccept")).setExecutor(commands);
        Objects.requireNonNull(getCommand("rtp")).setExecutor(commands);
        Objects.requireNonNull(getCommand("achievements")).setExecutor(commands);
        Objects.requireNonNull(getCommand("hud")).setExecutor(commands);
        Objects.requireNonNull(getCommand("shop")).setExecutor(commands);
        Objects.requireNonNull(getCommand("buy")).setExecutor(commands);
        Objects.requireNonNull(getCommand("exchange")).setExecutor(commands);
        Objects.requireNonNull(getCommand("docs")).setExecutor(docs);
        Objects.requireNonNull(getCommand("kit")).setExecutor(new KitCommandAdapter(this, renderer));
        Objects.requireNonNull(getCommand("vote")).setExecutor(new VoteCommandAdapter(this, renderer));
        Objects.requireNonNull(getCommand("mail")).setExecutor(new MailCommandAdapter(this, renderer));
        var moderation = new ModerationCommandAdapter(this, renderer);
        Objects.requireNonNull(getCommand("report")).setExecutor(new ReportCommandAdapter(this, renderer));
        Objects.requireNonNull(getCommand("reports")).setExecutor(new ReportsCommandAdapter(this, renderer));
        var warnings = new WarningCommandAdapter(this, renderer);
        Objects.requireNonNull(getCommand("warn")).setExecutor(warnings);
        Objects.requireNonNull(getCommand("warnings")).setExecutor(warnings);
        var notes = new NoteCommandAdapter(this, renderer);
        Objects.requireNonNull(getCommand("note")).setExecutor(notes);
        Objects.requireNonNull(getCommand("notes")).setExecutor(notes);
        Objects.requireNonNull(getCommand("ban")).setExecutor(moderation);
        Objects.requireNonNull(getCommand("unban")).setExecutor(moderation);
        Objects.requireNonNull(getCommand("mute")).setExecutor(moderation);
        Objects.requireNonNull(getCommand("unmute")).setExecutor(moderation);
        Objects.requireNonNull(getCommand("daily")).setExecutor(new DailyCommandAdapter(this, renderer));
        Objects.requireNonNull(getCommand("endexpedition")).setExecutor(new EndExpeditionCommandAdapter(this, renderer, endReturns));
        Objects.requireNonNull(getCommand("announce")).setExecutor(new AnnouncementCommandAdapter(this, renderer));
        Objects.requireNonNull(getCommand("claim")).setExecutor(new ClaimCommandAdapter(this, renderer, claimSnapshots));
        getServer().getPluginManager().registerEvents(locales, this);
        getServer().getPluginManager().registerEvents(menu, this);
        getServer().getPluginManager().registerEvents(docs, this);
        getServer().getPluginManager().registerEvents(textInput, this);
        getServer().getPluginManager().registerEvents(new HotbarMenuListener(menu, catalog, locales, token, inventorySync), this);
        getServer().getPluginManager().registerEvents(new PlayerLifecycleListener(this), this);
        getServer().getPluginManager().registerEvents(new TeleportArrivalListener(this), this);
        getServer().getPluginManager().registerEvents(new PortalAccessListener(randomTeleports), this);
        getServer().getPluginManager().registerEvents(new ChatMuteListener(this, renderer), this);
        getServer().getPluginManager().registerEvents(new ClaimProtectionListener(this, renderer), this);
        getServer().getPluginManager().registerEvents(actionBars, this);
        actionBars.start();
        claimSnapshots.start();
        endReturns.startExpiryWatcher();
        new ClaimLiveSmoke(this, claimSnapshots).start();
        getServer().getMessenger().registerIncomingPluginChannel(this,
            ProfileTransferMessages.CHANNEL, new ProfileTransferListener(this));
        getServer().getMessenger().registerOutgoingPluginChannel(this, ProfileTransferMessages.CHANNEL);
        new ServerHeartbeat(this, scheduler, daemon, instanceId()).start();
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

    public PlayerLocaleService localeService() {
        return locales;
    }

    public PermissionSnapshotCache adminGrants() {
        return adminGrants;
    }

    public ClaimCache claims() {
        return claims;
    }

    private static String instanceId() {
        return System.getenv().getOrDefault("LKJMC_INSTANCE_ID", "paper");
    }
}
