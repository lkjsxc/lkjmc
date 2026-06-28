package com.lkjmc.velocity;

import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonClient;
import com.lkjmc.common.daemon.DaemonJson;
import com.lkjmc.common.daemon.DaemonRequest;
import com.velocitypowered.api.proxy.ProxyServer;
import com.velocitypowered.api.proxy.server.ServerInfo;
import java.net.InetSocketAddress;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.UUID;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ConcurrentHashMap;

public final class VelocityServerRegistry {
    private final ProxyServer proxy;
    private final DaemonClient client;
    private final Set<String> managedServers = ConcurrentHashMap.newKeySet();

    public VelocityServerRegistry(ProxyServer proxy, DaemonClient client) {
        this.proxy = proxy;
        this.client = client;
    }

    public CompletableFuture<Void> refresh() {
        var request = new DaemonRequest(
            UUID.randomUUID(), new DaemonActor("velocity-plugin", "velocity"), "instance.list", Map.of()
        );
        return client.send(request).thenAccept(response -> {
            if (!response.ok()) {
                return;
            }
            DaemonJson.array(response.body(), "instances").ifPresent(instances -> {
                var desired = new HashSet<String>();
                for (var element : instances) {
                    if (element.isJsonObject()) {
                        var instance = element.getAsJsonObject();
                        var id = DaemonJson.string(instance, "id").orElse("");
                        var port = DaemonJson.integer(instance, "serverPort").orElse(0L).intValue();
                        if (!id.isBlank() && port > 0 && shouldRegister(instance)) {
                            desired.add(id);
                            register(id, port);
                        }
                    }
                }
                unregisterMissing(desired);
            });
        });
    }

    public List<String> registeredServers() {
        return proxy.getAllServers().stream()
            .map(server -> server.getServerInfo().getName())
            .sorted()
            .toList();
    }

    static boolean shouldRegister(com.google.gson.JsonObject instance) {
        return DaemonJson.bool(instance, "proxyRegistration");
    }

    private void register(String id, int port) {
        managedServers.add(id);
        if (proxy.getServer(id).isPresent()) {
            return;
        }
        proxy.registerServer(new ServerInfo(id, new InetSocketAddress("127.0.0.1", port)));
    }

    private void unregisterMissing(Set<String> desired) {
        for (var id : Set.copyOf(managedServers)) {
            if (desired.contains(id)) {
                continue;
            }
            proxy.getServer(id).ifPresent(server -> proxy.unregisterServer(server.getServerInfo()));
            managedServers.remove(id);
        }
    }
}
