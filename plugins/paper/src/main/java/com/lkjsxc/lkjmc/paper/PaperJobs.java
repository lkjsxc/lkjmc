package com.lkjsxc.lkjmc.paper;

import com.google.gson.*;
import com.lkjsxc.lkjmc.common.*;
import java.time.Instant;
import java.util.*;
import org.bukkit.*;
import org.bukkit.entity.Player;
import org.bukkit.inventory.ItemStack;

public final class PaperJobs implements AutoCloseable {
  private final PaperContext ctx;
  private final SpawnPolicy spawns;
  private final ClaimProtection claims;
  private final Provenance provenance;
  private final Journal receipts;
  private final InventoryTransactions inventory;
  private final BuildingTransactions buildings;
  private final AdventureTransactions adventures;
  private final IdentityTransactions identities;
  private final java.util.concurrent.ScheduledExecutorService leaseKeeper =
      java.util.concurrent.Executors.newSingleThreadScheduledExecutor(
          r -> {
            Thread t = new Thread(r, "lkjmc-world-lease");
            t.setDaemon(true);
            return t;
          });

  public PaperJobs(
      PaperContext ctx,
      SpawnPolicy spawns,
      ClaimProtection claims,
      Provenance provenance,
      WorldLocks locks,
      IdentityOwnership ownership,
      GameEvents events)
      throws Exception {
    this.ctx = ctx;
    this.spawns = spawns;
    this.claims = claims;
    this.provenance = provenance;
    receipts = new Journal(ctx.plugin().getDataFolder().toPath().resolve("job-receipts"));
    inventory = new InventoryTransactions(ctx);
    buildings = new BuildingTransactions(ctx, claims, provenance, locks, spawns);
    adventures = new AdventureTransactions(ctx, spawns, inventory);
    identities = new IdentityTransactions(ctx, spawns, ownership, events);
    Bukkit.getScheduler().runTaskTimer(ctx.plugin(), adventures::tick, 20, 20);
  }

  public void recover() throws Exception {
    // Physical inventory recovery is done on the isolated player's first join,
    // before another operation or interaction can run for that native UUID.
    for (JsonObject row : receipts.unfinished())
      throw new IllegalStateException("Unexpected unfinished receipt: " + row.get("id"));
    buildings.recover();
  }

  public boolean needsRecovery(UUID id) {
    return inventory.requiresRecovery(id);
  }

  public boolean identityBlocked(UUID id) {
    return identities.blocked(id);
  }

  public void recoverIdentityReceipts() throws Exception {
    for (UUID id : identities.unpublishedJobs()) {
      JsonObject remote = ctx.core().get("/internal/v1/jobs/" + id + "/identity-state");
      if (remote.get("state").getAsString().equals("succeeded")) {
        ctx.refreshProjection();
        ctx.main(
            () -> {
              claims.apply();
              return null;
            });
        identities.acknowledged(id, remote.getAsJsonObject("result"));
      }
    }
  }

  public void recoverPlayer(Player player) throws Exception {
    inventory.recover(player);
  }

  public void poll() {
    JsonObject job = null;
    java.util.concurrent.ScheduledFuture<?> heartbeat = null;
    java.util.concurrent.atomic.AtomicReference<Exception> leaseFailure =
        new java.util.concurrent.atomic.AtomicReference<>();
    try {
      recoverIdentityReceipts();
      JsonElement value = ctx.core().post("/internal/v1/poll", new JsonObject()).get("job");
      if (value.isJsonNull()) return;
      job = value.getAsJsonObject();
      JsonObject leased = job;
      heartbeat =
          leaseKeeper.scheduleAtFixedRate(
              () -> {
                try {
                  ctx.core().ack(leased, "leased", null, null, null);
                  leaseFailure.set(null);
                } catch (Exception e) {
                  leaseFailure.set(e);
                }
              },
              30,
              30,
              java.util.concurrent.TimeUnit.SECONDS);
      UUID id = CoreClient.uuid(job, "id");
      Optional<JsonObject> stored = receipts.read(id);
      JsonObject result;
      if (stored.isPresent()) result = stored.get().getAsJsonObject("result");
      else if (inventory.receipt(id).isPresent()) result = inventory.receipt(id).orElseThrow();
      else if (buildings.receipt(id).isPresent()) result = buildings.receipt(id).orElseThrow();
      else if (inventory.pending(id))
        throw new IllegalStateException("Waiting for the isolated player's inventory recovery");
      else {
        ctx.refreshProjection();
        if (leaseFailure.get() != null)
          throw new IllegalStateException(
              "World job lease could not be renewed", leaseFailure.get());
        JsonObject current = job;
        if (job.get("kind").getAsString().equals("identity.migrate")) {
          result = identities.execute(job);
        } else if (AdventureTransactions.handles(job)) {
          result = adventures.execute(job, () -> actor(current));
        } else if (BuildingTransactions.handles(job)) {
          Player player =
              buildings.pending(id) || !job.get("kind").getAsString().equals("asset.capture")
                  ? null
                  : ctx.main(() -> actor(current));
          result = buildings.execute(job, player);
        } else result = ctx.main(() -> execute(current));
      }
      receipts.write(id, CoreClient.object("id", id, "phase", "committed", "result", result));
      ctx.core().ack(job, "succeeded", result, null, null);
      teleportResult(job, null);
      if (job.get("kind").getAsString().equals("identity.migrate"))
        Faults.hit(ctx, "identity.core_ack");
      ctx.refreshProjection();
      if (job.get("kind").getAsString().equals("identity.migrate")) {
        ctx.main(
            () -> {
              claims.apply();
              return null;
            });
        identities.acknowledged(id, result);
      }
      buildings.acknowledged(id);
    } catch (BuildingTransactions.Waiting | IdentityTransactions.Waiting e) {
      try {
        ctx.core()
            .ack(
                job,
                "waiting",
                null,
                CoreClient.object(
                    "phase",
                    e instanceof IdentityTransactions.Waiting
                        ? "awaiting_disconnect"
                        : "awaiting_consent",
                    "message",
                    e.getMessage()),
                null);
      } catch (Exception failure) {
        ctx.plugin().getLogger().warning("Unable to persist consent wait: " + failure.getMessage());
      }
    } catch (IllegalArgumentException e) {
      if (job != null)
        try {
          if (inventory.pending(CoreClient.uuid(job, "id"))
              || buildings.pending(CoreClient.uuid(job, "id"))
              || identities.pending(CoreClient.uuid(job, "id"))
              || adventures.pending(CoreClient.uuid(job, "id")))
            throw new IllegalStateException("Prepared physical operation requires reconciliation");
          ctx.core().ack(job, "failed", CoreClient.object("effect", "none"), null, e.getMessage());
          teleportResult(job, e.getMessage());
          if (job.get("kind").getAsString().equals("identity.migrate"))
            identities.failedBeforePreparation(CoreClient.uuid(job, "id"));
        } catch (Exception failure) {
          ctx.plugin()
              .getLogger()
              .warning("Cannot report safe job failure: " + failure.getMessage());
        }
    } catch (Exception e) {
      // Persistence/network errors may have happened after a mutation. Leave the
      // operation leased for replay from its durable receipt, never compensate blindly.
      ctx.plugin().getLogger().warning("World job awaits reconciliation: " + e.getMessage());
      if (job != null)
        try {
          ctx.core()
              .ack(
                  job,
                  "leased",
                  null,
                  CoreClient.object("phase", "reconciling", "message", e.getMessage()),
                  null);
        } catch (Exception ignored) {
        }
    } finally {
      if (heartbeat != null) heartbeat.cancel(false);
    }
  }

  private void teleportResult(JsonObject job, String error) {
    if (!CoreClient.string(job, "kind", "").equals("player.teleport")) return;
    JsonObject payload = job.getAsJsonObject("payload");
    try {
      ctx.main(
          () -> {
            for (Player player : Bukkit.getOnlinePlayers()) {
              JsonObject session;
              try {
                session = ctx.session(player.getUniqueId());
              } catch (Exception ignored) {
                continue;
              }
              JsonElement account = session.get("account_id");
              if (!account.equals(job.get("actor")) && !account.equals(payload.get("target")))
                continue;
              SystemMessage message =
                  error == null
                      ? SystemMessage.of("teleport.completed")
                      : SystemMessage.of(
                          "teleport.failed",
                          Messages.error(CoreClient.string(session, "language", "en"), error));
              var text = ctx.text(player.getUniqueId(), message.toString());
              player.sendMessage(text);
              player.sendActionBar(text);
            }
            return null;
          });
    } catch (Exception noticeError) {
      ctx.plugin()
          .getLogger()
          .warning(
              "Teleport result remains in durable notification history: "
                  + noticeError.getMessage());
    }
  }

  @Override
  public void close() {
    leaseKeeper.shutdownNow();
  }

  private Player actor(JsonObject job) throws Exception {
    String account = job.get("actor").getAsString();
    for (Player online : Bukkit.getOnlinePlayers()) {
      JsonObject session = ctx.session(online.getUniqueId());
      if (session.get("account_id").getAsString().equals(account)) {
        Player player = online;
        if (player != null && !player.isDead() && !player.getWorld().getName().equals("holding")) {
          if (ctx.departing(player.getUniqueId()))
            throw new IllegalArgumentException(
                com.lkjsxc.lkjmc.common.SystemMessage.of(
                        "text.you_are_changing_servers_wait_until_you_arrive")
                    .toString());
          if (inventory.requiresRecovery(player.getUniqueId()))
            throw new IllegalStateException("Inventory is quarantined");
          return player;
        }
      }
    }
    throw new IllegalArgumentException(
        com.lkjsxc.lkjmc.common.SystemMessage.of("text.use_this_while_playing_in_the_official_smp")
            .toString());
  }

  private void combat(Player player) throws Exception {
    if (ctx.inCombat(player.getUniqueId()))
      throw new IllegalArgumentException(
          com.lkjsxc.lkjmc.common.SystemMessage.of(
                  "text.you_cannot_travel_for_30_seconds_after_pvp")
              .toString());
    JsonObject session = ctx.session(player.getUniqueId());
    if (session.has("combat_until")
        && !session.get("combat_until").isJsonNull()
        && Instant.parse(session.get("combat_until").getAsString()).isAfter(Instant.now()))
      throw new IllegalArgumentException(
          com.lkjsxc.lkjmc.common.SystemMessage.of(
                  "text.you_cannot_travel_for_30_seconds_after_pvp")
              .toString());
  }

  private JsonObject execute(JsonObject job) throws Exception {
    String kind = job.get("kind").getAsString();
    JsonObject payload = job.getAsJsonObject("payload");
    UUID id = CoreClient.uuid(job, "id");
    switch (kind) {
      case "claim.sync":
        claims.apply();
        buildings.transferPets(payload);
        return CoreClient.object("effect", "committed");
      case "claim.release":
        claims.release(CoreClient.uuid(payload, "claim_id"));
        return CoreClient.object("effect", "committed");
      case "home.set":
        {
          Player player = actor(job);
          Location point = player.getLocation();
          String worldId = claims.worldId(point.getWorld());
          if (worldId.isEmpty() || point.getWorld().getName().startsWith("adventure_"))
            throw new IllegalArgumentException(
                com.lkjsxc.lkjmc.common.SystemMessage.of("text.you_cannot_set_a_home_here")
                    .toString());
          JsonObject location = SpawnPolicy.location(point);
          location.addProperty("world_id", worldId);
          return CoreClient.object("effect", "committed", "location", location);
        }
      case "home.travel":
        {
          Player player = actor(job);
          combat(player);
          Location target = spawns.decode(payload.getAsJsonObject("location"));
          if (target == null)
            throw new IllegalArgumentException(
                com.lkjsxc.lkjmc.common.SystemMessage.of("text.the_home_world_is_missing")
                    .toString());
          target.getChunk().load();
          if (!target.getBlock().isPassable()
              || !target.clone().add(0, 1, 0).getBlock().isPassable())
            throw new IllegalArgumentException(
                com.lkjsxc.lkjmc.common.SystemMessage.of("text.the_home_is_obstructed").toString());
          spawns.teleport(player, target);
          return CoreClient.object("effect", "committed");
        }
      case "player.teleport":
        {
          Player player = actor(job);
          combat(player);
          Player target = null;
          for (Player online : Bukkit.getOnlinePlayers())
            if (ctx.session(online.getUniqueId()).get("account_id").equals(payload.get("target")))
              target = online;
          if (target == null || target.getWorld().getName().equals("holding"))
            throw new IllegalArgumentException(
                com.lkjsxc.lkjmc.common.SystemMessage.of(
                        "text.the_destination_player_could_not_be_found")
                    .toString());
          combat(target);
          if ((payload.has("session_id")
                  && !payload
                      .get("session_id")
                      .equals(ctx.session(player.getUniqueId()).get("session_id")))
              || (payload.has("target_session_id")
                  && !payload
                      .get("target_session_id")
                      .equals(ctx.session(target.getUniqueId()).get("session_id"))))
            throw new IllegalArgumentException(
                SystemMessage.of("text.this_travel_request_has_ended_choose_a_destination_again")
                    .toString());
          spawns.teleport(player, target.getLocation());
          return CoreClient.object("effect", "committed");
        }
      case "npc.sell":
        {
          Player player = actor(job);
          ItemStack[] after =
              InventoryTransactions.copy(player.getInventory().getStorageContents());
          Material material = Material.matchMaterial(payload.get("material").getAsString());
          if (material == null)
            throw new IllegalArgumentException(
                com.lkjsxc.lkjmc.common.SystemMessage.of("text.this_material_is_not_supported")
                    .toString());
          int amount = payload.get("amount").getAsInt();
          InventoryTransactions.remove(after, material, amount);
          return inventory.commit(
              player, id, after, CoreClient.object("effect", "committed", "removed", amount));
        }
      case "asset.capture":
        {
          if (!payload.get("kind").getAsString().equals("items"))
            throw new IllegalStateException("Building operation must use its physical journal");
          Player player = actor(job);
          ItemStack[] after =
              InventoryTransactions.copy(player.getInventory().getStorageContents());
          JsonObject selection = payload.getAsJsonObject("selection");
          int slot =
              selection.has("slot")
                  ? selection.get("slot").getAsInt()
                  : player.getInventory().getHeldItemSlot();
          if (slot < 0 || slot >= after.length || after[slot] == null || after[slot].isEmpty())
            throw new IllegalArgumentException(
                com.lkjsxc.lkjmc.common.SystemMessage.of("text.hold_the_item_you_want_to_deposit")
                    .toString());
          ItemStack item = after[slot];
          after[slot] = null;
          JsonObject manifest =
              CoreClient.object(
                  "asset_id",
                  payload.get("asset_id").getAsString(),
                  "version",
                  1,
                  "kind",
                  "items",
                  "items",
                  InventoryTransactions.encode(new ItemStack[] {item}),
                  "summary",
                  CoreClient.object(
                      "material",
                      item.getType().name(),
                      "amount",
                      item.getAmount(),
                      "name",
                      item.getType().translationKey()),
                  "required_consents",
                  List.of());
          if (CoreClient.JSON
                  .toJson(manifest)
                  .getBytes(java.nio.charset.StandardCharsets.UTF_8)
                  .length
              > 15 * 1024 * 1024)
            throw new IllegalArgumentException(
                com.lkjsxc.lkjmc.common.SystemMessage.of(
                        "text.this_item_contains_too_much_stored_data_split_its_conte_60b247184c")
                    .toString());
          return inventory.commit(
              player,
              id,
              after,
              CoreClient.object(
                  "effect", "committed", "original_removed", true, "manifest", manifest));
        }
      case "asset.receive":
        {
          Player player = actor(job);
          ItemStack[] after =
              InventoryTransactions.copy(player.getInventory().getStorageContents());
          for (ItemStack item :
              InventoryTransactions.decode(
                  payload.getAsJsonObject("manifest").get("items").getAsString()))
            if (item != null) InventoryTransactions.add(after, item);
          return inventory.commit(
              player,
              id,
              after,
              CoreClient.object(
                  "effect", "committed", "asset_id", payload.get("asset_id").getAsString()));
        }
      default:
        throw new IllegalStateException("Handler unavailable for world operation: " + kind);
    }
  }
}
