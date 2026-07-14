package com.lkjmc.paper;

import com.lkjmc.common.attestation.AttestationVerifier;
import com.lkjmc.common.runtime.JvmPluginRuntime;
import com.lkjmc.common.sync.SyncBootstrap;
import com.lkjmc.common.sync.SyncKey;
import java.time.Duration;
import java.util.List;
import java.util.Objects;
import org.bukkit.plugin.java.JavaPlugin;

public final class LkjmcPaperPlugin extends JavaPlugin {
    private JvmPluginRuntime runtime;

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
        if (runtime != null) runtime.closeAsync(Duration.ofSeconds(2));
        runtime = new JvmPluginRuntime(SyncBootstrap.fromEnvironment(System.getenv()), "paper");
        runtime.subscribe(List.of(new SyncKey("menus", "global")));
        var scheduler = new PaperSchedulerBridge(this);
        new ProfileApplicationAdapter(scheduler, runtime.effects(), AttestationVerifier.unavailable());
        new FreshAuthorityAdapter();
        new ActionbarSnapshotAdapter(scheduler);
        getLogger().info("lkjmc local UI enabled; attested player workflows unavailable");
    }

    @Override
    public void onDisable() {
        if (runtime != null) {
            runtime.closeAsync(Duration.ofSeconds(2));
            runtime = null;
        }
    }
}
