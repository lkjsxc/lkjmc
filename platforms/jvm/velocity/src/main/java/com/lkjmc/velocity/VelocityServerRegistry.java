package com.lkjmc.velocity;

import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonClient;
import com.lkjmc.common.daemon.DaemonRequest;
import com.velocitypowered.api.proxy.ProxyServer;
import com.velocitypowered.api.proxy.server.ServerInfo;
import java.net.InetSocketAddress;
import java.util.List;
import java.util.Map;
import java.util.UUID;
import java.util.regex.Pattern;

public final class VelocityServerRegistry {
    private static final Pattern INSTANCE = Pattern.compile(
        "\\{[^{}]*\\\"id\\\":\\\"([^\\\"]+)\\\"[^{}]*\\\"serverPort\\\":(\\d+)[^{}]*}"
    );

    private final ProxyServer proxy;
    private final DaemonClient client;

    public VelocityServerRegistry(ProxyServer proxy, DaemonClient client) {
        this.proxy = proxy;
        this.client = client;
    }

    public void refresh() {
        var request = new DaemonRequest(
            UUID.randomUUID(),
            new DaemonActor("velocity-plugin", "velocity"),
            "instance.list",
            Map.of()
        );
        client.send(request).thenAccept(response -> {
            if (!response.ok()) {
                return;
            }
            var raw = response.body().getOrDefault("raw", "").toString();
            var matcher = INSTANCE.matcher(raw);
            while (matcher.find()) {
                register(matcher.group(1), Integer.parseInt(matcher.group(2)));
            }
        });
    }

    public List<String> registeredServers() {
        return proxy.getAllServers().stream()
            .map(server -> server.getServerInfo().getName())
            .sorted()
            .toList();
    }

    private void register(String id, int port) {
        if (proxy.getServer(id).isPresent()) {
            return;
        }
        proxy.registerServer(new ServerInfo(id, new InetSocketAddress("127.0.0.1", port)));
    }
}
