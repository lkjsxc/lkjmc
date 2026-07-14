package com.lkjmc.paper;

import com.lkjmc.bindings.ItemSlot;
import com.lkjmc.bindings.ProfileSnapshot;
import com.lkjmc.common.attestation.AttestationVerifier;
import com.lkjmc.common.effect.BoundedEffectExecutor;
import com.lkjmc.common.effect.EffectTask;
import com.lkjmc.common.scheduler.PaperScheduler;
import com.lkjmc.common.workflow.WorkflowKey;
import java.time.Duration;
import java.util.ArrayList;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.CompletionStage;
import org.bukkit.Material;
import org.bukkit.entity.Player;
import org.bukkit.inventory.ItemStack;

public final class ProfileApplicationAdapter {
    public enum Result { APPLIED, MALFORMED, UNATTESTED, UNAVAILABLE }

    private final PaperScheduler scheduler;
    private final BoundedEffectExecutor effects;
    private final AttestationVerifier verifier;

    public ProfileApplicationAdapter(
            PaperScheduler scheduler,
            BoundedEffectExecutor effects,
            AttestationVerifier verifier) {
        this.scheduler = scheduler;
        this.effects = effects;
        this.verifier = verifier;
    }

    public CompletionStage<Result> apply(
            WorkflowKey key,
            ProfileSnapshot snapshot,
            Player player) {
        if (!exact(key, snapshot, player)) return CompletableFuture.completedFuture(Result.MALFORMED);
        EffectTask<AttestationVerifier.Attestation> verify = new EffectTask<>(
                "profile-attestation", 1, Duration.ofSeconds(2), () -> verifier.verify(key));
        return effects.submit(verify).handle((attestation, failure) -> {
            if (failure != null) return CompletableFuture.completedFuture(Result.UNAVAILABLE);
            if (!attestation.trusted() || !key.equals(attestation.key())) {
                return CompletableFuture.completedFuture(Result.UNATTESTED);
            }
            return scheduler.entity(player.getUniqueId(), () -> applyBukkit(snapshot, player))
                    .handle((unused, schedulerFailure) -> schedulerFailure == null
                            ? Result.APPLIED : Result.UNAVAILABLE).toCompletableFuture();
        }).thenCompose(value -> value);
    }

    private boolean exact(WorkflowKey key, ProfileSnapshot snapshot, Player player) {
        return key != null && snapshot != null && player != null
                && key.playerId().equals(player.getUniqueId())
                && key.playerId().equals(snapshot.playerUuid())
                && key.profileRevision() == snapshot.profileRevision()
                && key.fence() == snapshot.fence();
    }

    private void applyBukkit(ProfileSnapshot snapshot, Player player) {
        var prepared = new ArrayList<Prepared>();
        int size = player.getInventory().getSize();
        for (ItemSlot slot : snapshot.inventory()) {
            Material material = Material.matchMaterial(slot.material());
            if (material == null || !material.isItem() || slot.amount() < 1
                    || slot.amount() > material.getMaxStackSize() || slot.slot() < 0 || slot.slot() >= size) {
                throw new IllegalArgumentException("invalid typed inventory slot");
            }
            prepared.add(new Prepared(slot.slot(), new ItemStack(material, slot.amount())));
        }
        player.getInventory().clear();
        prepared.forEach(item -> player.getInventory().setItem(item.slot(), item.item()));
        player.updateInventory();
    }

    private record Prepared(int slot, ItemStack item) {}
}
