package com.lkjmc.paper;

import com.lkjmc.common.scheduler.PaperScheduler;
import java.time.Instant;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.CompletionStage;
import net.kyori.adventure.text.Component;
import org.bukkit.entity.Player;

public final class ActionbarSnapshotAdapter {
    public record Snapshot(Instant expiresAt, String message, long revision) {}
    private final PaperScheduler scheduler;

    public ActionbarSnapshotAdapter(PaperScheduler scheduler) {
        this.scheduler = scheduler;
    }

    public CompletionStage<Boolean> render(
            Player player,
            Snapshot snapshot,
            long requiredRevision,
            Instant now) {
        if (player == null || snapshot == null || now == null
                || snapshot.revision() != requiredRevision || !snapshot.expiresAt().isAfter(now)) {
            return CompletableFuture.completedFuture(false);
        }
        return scheduler.entity(player.getUniqueId(),
                () -> player.sendActionBar(Component.text(snapshot.message())))
                .handle((unused, failure) -> failure == null);
    }
}
