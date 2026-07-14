package com.lkjmc.common.scheduler;

import java.util.concurrent.CompletionStage;

public interface VelocityScheduler {
    CompletionStage<Void> event(Runnable action);
    CompletionStage<Void> async(Runnable submission);
}
