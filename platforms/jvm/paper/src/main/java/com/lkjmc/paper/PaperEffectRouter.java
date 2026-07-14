package com.lkjmc.paper;

import com.lkjmc.common.scheduler.PaperScheduler;
import java.util.UUID;
import java.util.concurrent.CompletionStage;

public final class PaperEffectRouter {
    private final PaperScheduler scheduler;

    public PaperEffectRouter(PaperScheduler scheduler) {
        this.scheduler = scheduler;
    }

    public CompletionStage<Void> global(Runnable action) {
        return scheduler.mainOrGlobal(action);
    }

    public CompletionStage<Void> entity(UUID playerId, Runnable action) {
        return scheduler.entity(playerId, action);
    }

    public CompletionStage<Void> region(String world, int chunkX, int chunkZ, Runnable action) {
        return scheduler.region(world, chunkX, chunkZ, action);
    }

    public CompletionStage<Void> asyncSubmission(Runnable action) {
        return scheduler.async(action);
    }
}
