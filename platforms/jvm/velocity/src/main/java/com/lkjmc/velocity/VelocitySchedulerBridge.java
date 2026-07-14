package com.lkjmc.velocity;

import com.lkjmc.common.scheduler.VelocityScheduler;
import com.velocitypowered.api.proxy.ProxyServer;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.CompletionStage;

public final class VelocitySchedulerBridge implements VelocityScheduler {
    private final ProxyServer proxy;
    private final Object plugin;

    public VelocitySchedulerBridge(ProxyServer proxy, Object plugin) {
        this.proxy = proxy;
        this.plugin = plugin;
    }

    @Override
    public CompletionStage<Void> event(Runnable action) {
        return run(action);
    }

    @Override
    public CompletionStage<Void> async(Runnable action) {
        CompletableFuture<Void> result = new CompletableFuture<>();
        proxy.getScheduler().buildTask(plugin, () -> complete(action, result)).schedule();
        return result;
    }

    private CompletionStage<Void> run(Runnable action) {
        CompletableFuture<Void> result = new CompletableFuture<>();
        complete(action, result);
        return result;
    }

    private void complete(Runnable action, CompletableFuture<Void> result) {
        try {
            action.run();
            result.complete(null);
        } catch (RuntimeException failure) {
            result.completeExceptionally(failure);
        }
    }
}
