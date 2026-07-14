package com.lkjmc.common.sync;

import java.nio.file.Path;
import java.time.Duration;
import java.util.List;

public final class SyncHarness {
    private static final List<String> PROBES = List.of(
            "freshness-bound-pass", "reconnect-storm-pass", "request-budget-pass",
            "auth-invalidation-pass", "shutdown-clean");

    private SyncHarness() {}

    public static void main(String[] args) throws Exception {
        String selected = args.length == 0 ? "all" : args[0];
        List<String> probes = "all".equals(selected) ? PROBES : List.of(selected);
        if (!PROBES.containsAll(probes)) {
            throw new IllegalArgumentException("unknown sync harness probe");
        }
        Path root = Path.of(System.getProperty("user.dir")).toAbsolutePath();
        for (String probe : probes) {
            try (Environment environment = new Environment(root)) {
                try {
                    SyncHarnessProbes.run(probe, environment);
                    String log = environment.daemon.logText();
                    check(!log.contains(environment.database.token()), "credential leaked to daemon log");
                    System.out.println("ok sync-harness probe=" + probe);
                } catch (Exception failure) {
                    System.err.println(environment.daemon.logText());
                    throw failure;
                }
            }
        }
    }

    static SyncConfig config(Environment environment, int inflight, Duration maxAge) {
        return new SyncConfig(environment.proxy.endpoint(), environment.database.token(),
                Duration.ofMillis(800), Duration.ofMillis(100), maxAge,
                64, inflight, 64, 2 * 1024 * 1024L, 1024 * 1024);
    }

    static void check(boolean condition, String message) {
        if (!condition) {
            throw new IllegalStateException(message);
        }
    }

    static boolean await(Duration timeout, CheckedBoolean condition) throws Exception {
        long deadline = System.nanoTime() + timeout.toNanos();
        while (System.nanoTime() < deadline) {
            if (condition.get()) {
                return true;
            }
            Thread.sleep(20);
        }
        return condition.get();
    }

    @FunctionalInterface
    interface CheckedBoolean { boolean get() throws Exception; }

    static final class Environment implements AutoCloseable {
        final HarnessDatabase database;
        final HarnessDaemon daemon;
        final HarnessProxy proxy;

        Environment(Path root) throws Exception {
            database = new HarnessDatabase(root);
            HarnessDaemon started = null;
            HarnessProxy forwarding = null;
            try {
                started = new HarnessDaemon(root, database.daemonUrl());
                forwarding = new HarnessProxy(started.endpoint());
                daemon = started;
                proxy = forwarding;
            } catch (Exception failure) {
                if (forwarding != null) forwarding.close();
                if (started != null) started.close();
                database.close();
                throw failure;
            }
        }

        @Override
        public void close() throws Exception {
            proxy.close();
            daemon.close();
            database.close();
        }
    }
}
