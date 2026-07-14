package com.lkjmc.common.effect;

import java.time.Duration;
import java.util.concurrent.CompletionStage;
import java.util.function.Supplier;

public record EffectTask<T>(
        String name,
        int maxAttempts,
        Duration timeout,
        Supplier<CompletionStage<T>> operation) {
    public EffectTask {
        if (name == null || name.isBlank() || maxAttempts < 1 || maxAttempts > 8
                || timeout == null || timeout.isNegative() || timeout.isZero() || operation == null) {
            throw new IllegalArgumentException("invalid bounded effect");
        }
    }
}
