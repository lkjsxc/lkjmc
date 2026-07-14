package com.lkjmc.paper.harness;
import com.lkjmc.bindings.CommandCatalog;
import com.lkjmc.bindings.ProfileSnapshot;
import com.lkjmc.bindings.Route;
import com.lkjmc.bindings.RoutingSnapshot;
import com.lkjmc.bindings.SyncDomain;
import com.lkjmc.common.attestation.AttestationVerifier;
import com.lkjmc.common.effect.BoundedEffectExecutor;
import com.lkjmc.common.effect.EffectTask;
import com.lkjmc.common.runtime.JvmPluginRuntime;
import com.lkjmc.common.workflow.WorkflowDecision;
import com.lkjmc.common.workflow.WorkflowKey;
import com.lkjmc.common.workflow.WorkflowKind;
import com.lkjmc.common.workflow.WorkflowMachine;
import com.lkjmc.common.workflow.WorkflowPhase;
import com.lkjmc.common.workflow.WorkflowSignal;
import com.lkjmc.paper.PaperEffectRouter;
import com.lkjmc.paper.ProfileApplicationAdapter;
import com.lkjmc.velocity.VelocityRoutingAdapter;
import com.lkjmc.velocity.VelocityTransferAdapter;
import java.nio.file.Path;
import java.time.Duration;
import java.time.Instant;
import java.util.List;
import java.util.UUID;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.TimeUnit;
import java.util.function.Supplier;
import java.util.zip.ZipFile;
public final class JvmProbeRunner {
    private static final int REPEATS = 10;
    private static Path paperJar, velocityJar; private static final String SELECTED = System.getProperty("lkjmc.jvm.probe", "");
    public static void main(String[] args) throws Exception {
        check(args.length == 2, "two real plugin jars required");
        paperJar = Path.of(args[0]); velocityJar = Path.of(args[1]);
        probe("scheduler-blocks-zero", JvmProbeRunner::schedulerBlocksZero);
        probe("typed-bindings-all", JvmProbeRunner::typedBindingsAll);
        probe("folia-ownership-pass", JvmProbeRunner::foliaOwnership);
        probe("velocity-routing-pass", JvmProbeRunner::velocityRouting);
        probe("transfer-outcomes-pass", JvmProbeRunner::transferOutcomes);
        probe("workflow-ack-pass", JvmProbeRunner::workflowAck);
        probe("plugin-shutdown-pass", JvmProbeRunner::pluginShutdown);
        probe("duplicate-jvm-paths-absent", JvmProbeRunner::jarContainment);
    }
    private static void schedulerBlocksZero() throws Exception {
        var hops = new HarnessFakes.PaperHops(); hops.hold = true;
        try (var effects = new BoundedEffectExecutor("nonblock", 1, 2)) {
            WorkflowKey key = key(); var player = HarnessFakes.player(key.playerId());
            var adapter = new ProfileApplicationAdapter(hops, effects, trusted());
            long start = System.nanoTime();
            var result = adapter.apply(key, profile(key), player.player()).toCompletableFuture();
            check(Duration.ofNanos(System.nanoTime() - start).toMillis() < 100, "submission blocked");
            spin(() -> hops.held != null);
            check(!result.isDone(), "held ownership hop completed"); hops.release();
            check(result.get(2, TimeUnit.SECONDS) == ProfileApplicationAdapter.Result.APPLIED, "apply failed");
        }
        EffectExecutorStress.queueSaturation();
    }
    private static void typedBindingsAll() {
        check(CommandCatalog.ALL.size() >= 100, "canonical commands missing");
        check(CommandCatalog.JVM_CONSUMED == 0, "withdrawn commands generated as consumers");
        check(CommandCatalog.ALL.stream().map(item -> item.name()).distinct().count()
                == CommandCatalog.ALL.size(), "duplicate bindings");
        check(CommandCatalog.SOURCE_SHA256.matches("[0-9a-f]{64}"), "source digest missing");
        check(SyncDomain.values().length == 7, "typed sync domains missing");
    }
    private static void foliaOwnership() throws Exception {
        for (int repeat = 0; repeat < REPEATS; repeat++) {
            var hops = new HarnessFakes.PaperHops(); WorkflowKey key = key();
            var player = HarnessFakes.player(key.playerId());
            try (var effects = new BoundedEffectExecutor("folia", 1, 2)) {
                var denied = new ProfileApplicationAdapter(hops, effects, AttestationVerifier.unavailable());
                check(denied.apply(key, profile(key), player.player()).toCompletableFuture().get()
                        == ProfileApplicationAdapter.Result.UNATTESTED && player.clears().get() == 0
                        && hops.entity.get() == 0, "unattested profile mutated");
                var adapter = new ProfileApplicationAdapter(hops, effects, trusted());
                check(adapter.apply(key, profile(key), player.player()).toCompletableFuture()
                        .get(2, TimeUnit.SECONDS) == ProfileApplicationAdapter.Result.APPLIED, "unapplied");
                var router = new PaperEffectRouter(hops);
                router.global(() -> {}).toCompletableFuture().get();
                router.region("world", 2, 3, () -> {}).toCompletableFuture().get();
                router.asyncSubmission(() -> {}).toCompletableFuture().get();
                check(hops.entity.get() == 1 && hops.global.get() == 1 && hops.region.get() == 1
                        && hops.async.get() == 1, "wrong ownership hop");
                check(player.clears().get() == 1, "Bukkit inventory API not exercised");
            }
        }
    }
    private static void velocityRouting() throws Exception {
        for (int repeat = 0; repeat < REPEATS; repeat++) {
            var platform = new HarnessFakes.ProxyEffects(); platform.names.add("unrelated");
            platform.names.add(VelocityRoutingAdapter.owned("old"));
            var hops = new HarnessFakes.VelocityHops();
            var adapter = new VelocityRoutingAdapter(platform, hops);
            Instant now = Instant.now();
            var first = new RoutingSnapshot(now.plusSeconds(5), 1,
                    List.of(new Route("127.0.0.1", "lobby", 25566, true)));
            check(adapter.reconcile(first, 1, now).toCompletableFuture().get(), "reconcile failed");
            check(platform.names.contains("unrelated") && !platform.names.contains(
                    VelocityRoutingAdapter.owned("old")), "ownership violation");
            var restarted = new VelocityRoutingAdapter(platform, hops);
            var second = new RoutingSnapshot(now.plusSeconds(5), 2,
                    List.of(new Route("127.0.0.1", "survival", 25567, true)));
            check(restarted.reconcile(second, 2, now).toCompletableFuture().get(), "restart repair failed");
            check(!restarted.reconcile(second, 1, now).toCompletableFuture().get(), "stale route accepted");
        }
    }
    private static void transferOutcomes() throws Exception {
        WorkflowKey key = key(); var machine = new WorkflowMachine(WorkflowKind.TRANSFER, key);
        machine.apply(key, WorkflowSignal.TRANSFER_REQUESTED, false, "");
        machine.apply(key, WorkflowSignal.SAVE_ACKNOWLEDGED, true, "");
        var platform = new HarnessFakes.ProxyEffects();
        platform.names.add(VelocityRoutingAdapter.owned("lobby"));
        try (var effects = new BoundedEffectExecutor("transfer", 1, 4)) {
            var unavailable = new VelocityTransferAdapter(platform, effects, AttestationVerifier.unavailable());
            WorkflowDecision connected = unavailable.connect(machine, key, "lobby").toCompletableFuture().get();
            check(connected.view().phase() == WorkflowPhase.CONNECTED && !connected.view().succeeded(),
                    "connect claimed arrival");
            check(unavailable.attestArrival(machine, key).toCompletableFuture().get().outcome()
                    == WorkflowDecision.Outcome.DENIED, "unattested arrival accepted");
            var trusted = new VelocityTransferAdapter(platform, effects, trusted());
            check(trusted.attestArrival(machine, key).toCompletableFuture().get().view().succeeded(),
                    "attested arrival not correlated");
            var lost = new WorkflowMachine(WorkflowKind.TRANSFER, key);
            lost.apply(key, WorkflowSignal.TRANSFER_REQUESTED, false, "");
            lost.apply(key, WorkflowSignal.SAVE_ACKNOWLEDGED, true, ""); platform.connectionResult = false;
            check(trusted.connect(lost, key, "lobby").toCompletableFuture().get().view().phase()
                    == WorkflowPhase.FAILED, "connection loss not terminal");
        }
    }
    private static void workflowAck() {
        for (int repeat = 0; repeat < REPEATS; repeat++) {
            WorkflowKey key = key(); var machine = new WorkflowMachine(WorkflowKind.PROFILE, key);
            check(!machine.apply(key, WorkflowSignal.SAVE_REQUESTED, false, "").view().succeeded(),
                    "request became success");
            check(machine.apply(key, WorkflowSignal.PROFILE_APPLIED, true, "").outcome()
                    == WorkflowDecision.Outcome.DENIED, "reorder accepted");
            check(machine.apply(key, WorkflowSignal.SAVE_ACKNOWLEDGED, false, "").outcome()
                    == WorkflowDecision.Outcome.DENIED, "untrusted ack accepted");
            machine.apply(key, WorkflowSignal.SAVE_ACKNOWLEDGED, true, "");
            WorkflowDecision duplicate = machine.apply(key, WorkflowSignal.SAVE_ACKNOWLEDGED, true, "");
            check(duplicate.outcome() == WorkflowDecision.Outcome.DUPLICATE, "duplicate unstable");
            machine.apply(key, WorkflowSignal.LOAD_REQUESTED, false, "");
            check(machine.apply(key, WorkflowSignal.PROFILE_APPLIED, true, "").view().succeeded(), "no ack");
            WorkflowKey mismatch = new WorkflowKey(key.operationId(), key.sessionId(), key.playerId(),
                    key.profileRevision(), key.fence() + 1, key.correlationId());
            check(machine.apply(mismatch, WorkflowSignal.PROFILE_APPLIED, true, "").outcome()
                    == WorkflowDecision.Outcome.DENIED, "fence mutation survived");
        }
    }
    private static void pluginShutdown() throws Exception {
        for (int repeat = 0; repeat < REPEATS; repeat++) {
            var runtime = new JvmPluginRuntime(java.util.Optional.empty(), "shutdown");
            runtime.effects().submit(new EffectTask<>("held", 1, Duration.ofSeconds(10),
                    () -> new CompletableFuture<>()));
            check(runtime.closeAsync(Duration.ofSeconds(2)).get(3, TimeUnit.SECONDS), "shutdown not joined");
        }
        check(Thread.getAllStackTraces().keySet().stream().noneMatch(thread ->
                thread.isAlive() && thread.getName().startsWith("lkjmc-effect-")), "worker leaked");
    }
    private static void jarContainment() throws Exception {
        inspect(paperJar, "com/lkjmc/paper/LkjmcPaperPlugin.class");
        inspect(velocityJar, "com/lkjmc/velocity/LkjmcVelocityPlugin.class");
    }
    private static void inspect(Path jar, String pluginClass) throws Exception {
        check(java.nio.file.Files.isRegularFile(jar), "real plugin jar missing: " + jar);
        try (ZipFile zip = new ZipFile(jar.toFile())) {
            var names = zip.stream().map(entry -> entry.getName()).toList();
            check(names.size() == names.stream().distinct().count(), "duplicate jar paths");
            check(names.contains(pluginClass), "plugin class missing");
            check(names.stream().filter(name -> name.endsWith("/SyncCoordinator.class")).count() == 1,
                    "duplicate coordinator");
            check(names.stream().filter(name -> name.endsWith("/BoundedEffectExecutor.class")).count() == 1,
                    "duplicate effect executor");
        }
    }
    private static WorkflowKey key() {
        return new WorkflowKey(UUID.randomUUID(), UUID.randomUUID(), UUID.randomUUID(), 7, 9, UUID.randomUUID());
    }
    private static ProfileSnapshot profile(WorkflowKey key) {
        return new ProfileSnapshot(key.fence(), List.of(), key.playerId(), key.profileRevision(), "global");
    }
    private static AttestationVerifier trusted() {
        return key -> CompletableFuture.completedFuture(new AttestationVerifier.Attestation(key, true));
    }
    private static void spin(Supplier<Boolean> condition) {
        long limit = System.nanoTime() + Duration.ofSeconds(2).toNanos();
        while (!condition.get() && System.nanoTime() < limit) Thread.onSpinWait();
        check(condition.get(), "timed out waiting for harness hop");
    }
    private static void probe(String name, Checked action) throws Exception {
        if (!SELECTED.isEmpty() && !SELECTED.equals(name)) return;
        action.run(); System.out.println(name + ": PASS");
    }
    private static void check(boolean condition, String message) {
        if (!condition) throw new IllegalStateException(message);
    }
    @FunctionalInterface private interface Checked { void run() throws Exception; }
    private JvmProbeRunner() {}
}
