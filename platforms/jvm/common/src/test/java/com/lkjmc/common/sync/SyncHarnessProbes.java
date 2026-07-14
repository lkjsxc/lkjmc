package com.lkjmc.common.sync;

import java.sql.Connection;
import java.time.Duration;
import java.util.ArrayList;
import java.util.List;
import java.util.UUID;

final class SyncHarnessProbes {
    private static final Duration FRESHNESS = Duration.ofSeconds(5);

    private SyncHarnessProbes() {}

    static void run(String probe, SyncHarness.Environment environment) throws Exception {
        switch (probe) {
            case "freshness-bound-pass" -> freshness(environment);
            case "reconnect-storm-pass" -> reconnect(environment);
            case "request-budget-pass" -> budget(environment);
            case "auth-invalidation-pass" -> auth(environment);
            case "shutdown-clean" -> shutdown(environment);
            default -> throw new IllegalArgumentException("unknown probe");
        }
    }

    private static void freshness(SyncHarness.Environment env) throws Exception {
        SyncKey key = settings(env.database.player());
        env.proxy.dropOneSnapshot();
        try (SyncCoordinator coordinator = new SyncCoordinator(
                SyncHarness.config(env, 4, Duration.ofSeconds(2)))) {
            coordinator.subscribe(key);
            SyncHarness.check(awaitLanguage(coordinator, key, "en"),
                    "dropped snapshot was not repaired requests=" + env.proxy.snapshotRequests()
                            + " inflight=" + coordinator.requestCount() + " last=" + env.proxy.lastSnapshot());
            long firstRevision = coordinator.view(key).orElseThrow().revision();
            env.proxy.holdNextSnapshot();
            env.database.language(env.database.player(), "ja");
            SyncHarness.check(env.proxy.awaitCaptured(Duration.ofSeconds(3)), "snapshot was not captured");
            env.database.language(env.database.player(), "de");
            env.proxy.release();
            SyncHarness.check(awaitLanguage(coordinator, key, "de"), "reordered snapshot was not repaired");
            SyncHarness.check(coordinator.view(key).orElseThrow().revision() > firstRevision,
                    "revision did not increase");
            coordinator.close();
            SyncHarness.check(coordinator.awaitClosed(Duration.ofSeconds(2)), "freshness coordinator survived");
        }
    }

    private static void reconnect(SyncHarness.Environment env) throws Exception {
        SyncKey key = settings(env.database.player());
        long checkpoint;
        SyncConfig config = SyncHarness.config(env, 4, Duration.ofSeconds(2));
        SyncCoordinator first = new SyncCoordinator(config);
        first.subscribe(key);
        SyncHarness.check(awaitLanguage(first, key, "en"), "initial snapshot unavailable");
        SyncHarness.check(first.subscriptionCount() == 1, "duplicate subscription created");
        checkpoint = first.checkpoint();
        first.close();
        SyncHarness.check(first.awaitClosed(Duration.ofSeconds(2)), "initial coordinator survived");
        SyncCoordinator resumed = new SyncCoordinator(config, checkpoint);
        resumed.subscribe(key);
        for (int index = 0; index < 3; index++) {
            env.daemon.stop();
            String language = "r" + index;
            env.database.language(env.database.player(), language);
            for (int duplicate = 0; duplicate < 32; duplicate++) resumed.subscribe(key);
            env.daemon.start();
            SyncHarness.check(awaitLanguage(resumed, key, language), "restart was not repaired");
        }
        SyncHarness.check(resumed.subscriptionCount() == 1, "reconnect created duplicate pollers");
        SyncHarness.check(resumed.checkpoint() >= checkpoint, "cursor checkpoint was not reloaded");
        resumed.close();
        SyncHarness.check(resumed.awaitClosed(Duration.ofSeconds(2)), "resumed coordinator survived");
    }

    private static void budget(SyncHarness.Environment env) throws Exception {
        List<SyncKey> keys = new ArrayList<>();
        keys.add(settings(env.database.player()));
        for (int index = 0; index < 20; index++) {
            UUID player = UUID.randomUUID();
            env.database.createPlayer(player);
            keys.add(settings(player));
        }
        env.proxy.holdAllSnapshots(true);
        long started = System.nanoTime();
        SyncCoordinator coordinator = new SyncCoordinator(
                SyncHarness.config(env, 2, Duration.ofSeconds(2)));
        keys.forEach(coordinator::subscribe);
        Duration submitted = Duration.ofNanos(System.nanoTime() - started);
        SyncHarness.check(submitted.compareTo(Duration.ofMillis(250)) < 0, "enable path blocked");
        SyncHarness.check(SyncHarness.await(Duration.ofSeconds(2), () -> coordinator.requestCount() > 0),
                "bounded requests did not start");
        Thread.sleep(250);
        SyncHarness.check(coordinator.requestCount() <= 2 && env.proxy.maximumConcurrent() <= 2,
                "request budget exceeded");
        env.proxy.holdAllSnapshots(false);
        SyncHarness.check(awaitLanguage(coordinator, keys.get(keys.size() - 1), "en"),
                "request budget did not recover");
        started = System.nanoTime();
        coordinator.close();
        Duration disabled = Duration.ofNanos(System.nanoTime() - started);
        SyncHarness.check(disabled.compareTo(Duration.ofMillis(250)) < 0, "disable path blocked");
        SyncHarness.check(coordinator.awaitClosed(Duration.ofSeconds(2)), "budget coordinator survived");
    }

    private static void auth(SyncHarness.Environment env) throws Exception {
        SyncKey key = settings(env.database.player());
        SyncCoordinator coordinator = new SyncCoordinator(
                SyncHarness.config(env, 3, Duration.ofMillis(600)));
        coordinator.subscribe(key);
        SyncHarness.check(awaitLanguage(coordinator, key, "en"), "auth baseline unavailable");
        String second = "sync-rotated-" + UUID.randomUUID();
        env.database.createCredential(second, "velocity");
        env.database.language(env.database.player(), "ja");
        SyncHarness.check(awaitLanguage(coordinator, key, "ja"), "credential revision did not reconnect");
        coordinator.replaceCredential(second);
        env.database.language(env.database.player(), "de");
        SyncHarness.check(awaitLanguage(coordinator, key, "de"), "credential rotation failed");
        env.database.revoke(second);
        SyncHarness.check(SyncHarness.await(FRESHNESS, () -> coordinator.view(key).isEmpty()),
                "revoked credential retained current cache");
        coordinator.replaceCredential(env.database.token());
        SyncHarness.check(awaitLanguage(coordinator, key, "de"), "valid credential did not recover");
        try (Connection lock = env.database.lockCredentialRevision()) {
            coordinator.replaceCredential(env.database.token());
            SyncHarness.check(SyncHarness.await(Duration.ofSeconds(2), () -> coordinator.view(key).isEmpty()),
                    "database uncertainty did not fail closed");
            lock.rollback();
        }
        SyncHarness.check(awaitLanguage(coordinator, key, "de"), "database recovery failed");
        SyncHarness.check(!env.daemon.logText().contains(second), "rotated credential leaked");
        coordinator.close();
        SyncHarness.check(coordinator.awaitClosed(Duration.ofSeconds(2)), "auth coordinator survived");
    }

    private static void shutdown(SyncHarness.Environment env) throws Exception {
        env.proxy.holdAllSnapshots(true);
        SyncCoordinator coordinator = new SyncCoordinator(
                SyncHarness.config(env, 3, Duration.ofSeconds(2)));
        coordinator.subscribe(settings(env.database.player()));
        SyncHarness.check(SyncHarness.await(Duration.ofSeconds(2), () -> coordinator.requestCount() > 0),
                "cancellable request did not start");
        long started = System.nanoTime();
        coordinator.close();
        SyncHarness.check(Duration.ofNanos(System.nanoTime() - started).compareTo(Duration.ofMillis(250)) < 0,
                "close blocked caller");
        env.proxy.holdAllSnapshots(false);
        SyncHarness.check(coordinator.awaitClosed(Duration.ofSeconds(2)), "coordinator did not terminate");
        SyncHarness.check(SyncHarness.await(Duration.ofSeconds(1), () -> namedThreads() == 0),
                "sync thread survived close");
    }

    private static SyncKey settings(UUID player) { return new SyncKey("settings", player.toString()); }

    private static boolean awaitLanguage(SyncCoordinator coordinator, SyncKey key, String expected)
            throws Exception {
        return SyncHarness.await(FRESHNESS, () -> coordinator.view(key)
                .map(value -> value.payload().getAsJsonObject().get("language").getAsString().equals(expected))
                .orElse(false));
    }

    private static long namedThreads() {
        return Thread.getAllStackTraces().keySet().stream().filter(Thread::isAlive)
                .filter(thread -> thread.getName().startsWith("lkjmc-sync-")).count();
    }
}
