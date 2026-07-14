package com.lkjmc.paper;

import com.lkjmc.common.attestation.AttestationVerifier;
import com.lkjmc.common.runtime.JvmPluginRuntime;
import com.lkjmc.common.runtime.SerializedRuntimeOwner;
import com.lkjmc.common.sync.SyncBootstrap;
import com.lkjmc.common.sync.SyncKey;
import java.time.Duration;
import java.util.List;
import java.util.Objects;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.CompletionStage;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.atomic.AtomicLong;
import java.util.function.Consumer;
import java.util.function.Function;
import java.util.function.Supplier;
import org.bukkit.event.HandlerList;
import org.bukkit.plugin.java.JavaPlugin;

public final class LkjmcPaperPlugin extends JavaPlugin {
    private final Lifecycle lifecycle = new Lifecycle(Duration.ofSeconds(2),
            action -> new PaperSchedulerBridge(this).mainOrGlobal(action));

    @Override
    public synchronized void onEnable() {
        lifecycle.enable(() -> HandlerList.unregisterAll(this), () -> new JvmPluginRuntime(
                SyncBootstrap.fromEnvironment(System.getenv()), "paper"), this::install);
        getLogger().info("lkjmc local UI enable admitted; attested player workflows unavailable");
    }

    private void install(JvmPluginRuntime runtime) {
        var docs = new LocalDocsMenu(this);
        var tokens = new HotbarMenuTokenService(this);
        var sync = new InventorySyncService(tokens);
        var commands = new DocsCommandAdapter(docs);
        Objects.requireNonNull(getCommand("menu")).setExecutor(commands);
        Objects.requireNonNull(getCommand("docs")).setExecutor(commands);
        getServer().getPluginManager().registerEvents(docs, this);
        getServer().getPluginManager().registerEvents(new HotbarMenuListener(docs, tokens, sync), this);
        runtime.subscribe(List.of(new SyncKey("menus", "global")));
        var scheduler = new PaperSchedulerBridge(this);
        new ProfileApplicationAdapter(scheduler, runtime.effects(), AttestationVerifier.unavailable());
        new FreshAuthorityAdapter();
        new ActionbarSnapshotAdapter(scheduler);
    }

    @Override
    public synchronized void onDisable() {
        lifecycle.disable(() -> HandlerList.unregisterAll(this));
    }

    public static final class Lifecycle {
        private final Duration timeout;
        private final Function<Runnable, CompletionStage<Void>> scheduler;
        private final SerializedRuntimeOwner owner;
        private final AtomicBoolean enabled = new AtomicBoolean();
        private final AtomicLong generation = new AtomicLong();

        public Lifecycle(Duration timeout, Function<Runnable, CompletionStage<Void>> scheduler) {
            if (scheduler == null) throw new IllegalArgumentException("scheduler required");
            this.timeout = timeout;
            this.scheduler = scheduler;
            this.owner = new SerializedRuntimeOwner(timeout);
        }

        public CompletableFuture<Void> enable(Runnable uninstall,
                Supplier<JvmPluginRuntime> factory, Consumer<JvmPluginRuntime> install) {
            long admitted = generation.incrementAndGet();
            enabled.set(true);
            return owner.replace(() -> dispatch(uninstall), factory, runtime -> {
                if (enabled.get() && generation.get() == admitted) {
                    dispatch(() -> {
                        if (enabled.get() && generation.get() == admitted) install.accept(runtime);
                    });
                }
            });
        }

        public CompletableFuture<Void> disable(Runnable uninstall) {
            enabled.set(false);
            generation.incrementAndGet();
            uninstall.run();
            return owner.closeAsync(() -> {});
        }

        public boolean awaitIdle(Duration wait) throws InterruptedException { return owner.awaitIdle(wait); }
        public int activeRuntimes() { return owner.activeRuntimes(); }
        public int maximumActiveRuntimes() { return owner.maximumActiveRuntimes(); }

        private void dispatch(Runnable action) {
            try {
                scheduler.apply(action).toCompletableFuture()
                        .get(timeout.toNanos(), TimeUnit.NANOSECONDS);
            } catch (InterruptedException interrupted) {
                Thread.currentThread().interrupt();
                throw new IllegalStateException("Paper lifecycle interrupted", interrupted);
            } catch (Exception failure) {
                throw new IllegalStateException("Paper lifecycle stage failed", failure);
            }
        }
    }
}
