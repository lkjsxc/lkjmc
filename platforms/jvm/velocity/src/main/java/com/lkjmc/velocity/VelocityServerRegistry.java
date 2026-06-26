package com.lkjmc.velocity;

import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonClient;
import com.lkjmc.common.daemon.DaemonJson;
import com.lkjmc.common.daemon.DaemonRequest;
import com.velocitypowered.api.proxy.ProxyServer;
import com.velocitypowered.api.proxy.server.ServerInfo;
import java.net.InetSocketAddress;
import java.util.List;
import java.util.Map;
import java.util.UUID;

public final class VelocityServerRegistry {
    private final ProxyServer proxy;
    private final DaemonClient client;

    public VelocityServerRegistry(ProxyServer proxy, DaemonClient client) {
        this.proxy = proxy;
        this.client = client;
    }

    public void refresh() {
        var request = new DaemonRequest(
            UUID.randomUUID(), new DaemonActor("velocity-plugin", "velocity"), "instance.list", Map.of()
        );
        client.send(request).thenAccept(response -> {
            if (!response.ok()) {
                return;
            }
            DaemonJson.array(response.body(), "instances").ifPresent(instances -> {
                for (var element : instances) {
                    if (element.isJsonObject()) {
                        var instance = element.getAsJsonObject();
                        var id = DaemonJson.string(instance, "id").orElse("");
                        var port = DaemonJson.integer(instance, "serverPort").orElse(0L).intValue();
                        if (!id.isBlank() && port > 0) {
                            register(id, port);
                        }
                    }
                }
            });
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
