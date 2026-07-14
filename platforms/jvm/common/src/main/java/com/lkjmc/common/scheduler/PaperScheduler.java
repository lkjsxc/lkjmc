package com.lkjmc.common.scheduler;

import java.util.UUID;
import java.util.concurrent.CompletionStage;

public interface PaperScheduler {
    CompletionStage<Void> mainOrGlobal(Runnable action);
    CompletionStage<Void> entity(UUID playerId, Runnable action);
    CompletionStage<Void> region(String world, int chunkX, int chunkZ, Runnable action);
    CompletionStage<Void> async(Runnable submission);
}
