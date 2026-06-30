package com.lkjmc.velocity;

import com.lkjmc.common.daemon.DaemonClient;
import com.lkjmc.common.permission.PermissionSnapshotCache;
import com.velocitypowered.api.command.CommandMeta;
import com.velocitypowered.api.proxy.ProxyServer;
import java.util.Optional;

public final class VelocityCommands {
    private final ProxyServer proxy;
    private final Optional<DaemonClient> daemon;
    private final Optional<VelocityServerRegistry> registry;
    private final VelocityRestartAdapter restart;
    private final ProfileSaveBridge transfers;
    private final PermissionSnapshotCache adminGrants;

    public VelocityCommands(
        ProxyServer proxy,
        Optional<DaemonClient> daemon,
        Optional<VelocityServerRegistry> registry,
        VelocityRestartAdapter restart,
        ProfileSaveBridge transfers
    ) {
        this(proxy, daemon, registry, restart, transfers, PermissionSnapshotCache.disabled());
    }

    public VelocityCommands(
        ProxyServer proxy,
        Optional<DaemonClient> daemon,
        Optional<VelocityServerRegistry> registry,
        VelocityRestartAdapter restart,
        ProfileSaveBridge transfers,
        PermissionSnapshotCache adminGrants
    ) {
        this.proxy = proxy;
        this.daemon = daemon == null ? Optional.empty() : daemon;
        this.registry = registry == null ? Optional.empty() : registry;
        this.restart = restart;
        this.transfers = transfers;
        this.adminGrants = adminGrants == null ? PermissionSnapshotCache.disabled() : adminGrants;
    }

    public void register() {
        var commands = proxy.getCommandManager();
        var lkjmc = new VelocityLkjmcCommand(proxy, daemon, registry, restart, transfers, adminGrants);
        commands.register(VelocityLkjmcBrigadier.create(lkjmc));
        CommandMeta hub = commands.metaBuilder("hub").build();
        commands.register(hub, new VelocityHubCommand(proxy, transfers));
    }
}
