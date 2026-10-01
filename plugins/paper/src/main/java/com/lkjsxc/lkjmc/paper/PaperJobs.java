package com.lkjsxc.lkjmc.paper;

import com.google.gson.*;
import com.lkjsxc.lkjmc.common.*;
import java.time.Instant;
import java.util.*;
import org.bukkit.*;
import org.bukkit.entity.Player;
import org.bukkit.inventory.ItemStack;

public final class PaperJobs {
  private final PaperContext ctx;
  private final SpawnPolicy spawns;
  private final ClaimProtection claims;
  private final Provenance provenance;
  private final Journal receipts;
  private final InventoryTransactions inventory;

  public PaperJobs(
      PaperContext ctx, SpawnPolicy spawns, ClaimProtection claims, Provenance provenance)
      throws Exception {
    this.ctx = ctx;
    this.spawns = spawns;
    this.claims = claims;
    this.provenance = provenance;
    receipts = new Journal(ctx.plugin().getDataFolder().toPath().resolve("job-receipts"));
    inventory = new InventoryTransactions(ctx);
  }

  public void recover() throws Exception {
    // Physical inventory recovery is done on the isolated player's first join,
    // before another operation or interaction can run for that native UUID.
    for (JsonObject row : receipts.unfinished())
      throw new IllegalStateException("Unexpected unfinished receipt: " + row.get("id"));
  }

  public boolean needsRecovery(UUID id) {
    return inventory.requiresRecovery(id);
  }

  public void recoverPlayer(Player player) throws Exception {
    inventory.recover(player);
  }

  public void poll() {
    JsonObject job = null;
    try {
      JsonElement value = ctx.core().post("/internal/v1/poll", new JsonObject()).get("job");
      if (value.isJsonNull()) return;
      job = value.getAsJsonObject();
      UUID id = CoreClient.uuid(job, "id");
      Optional<JsonObject> stored = receipts.read(id);
      JsonObject result;
      if (stored.isPresent()) result = stored.get().getAsJsonObject("result");
      else if (inventory.receipt(id).isPresent()) result = inventory.receipt(id).orElseThrow();
      else if (inventory.pending(id))
        throw new IllegalStateException("Waiting for the isolated player's inventory recovery");
      else {
        ctx.refreshProjection();
        JsonObject current = job;
        result = ctx.main(() -> execute(current));
      }
      receipts.write(id, CoreClient.object("id", id, "phase", "committed", "result", result));
      ctx.core().ack(job, "succeeded", result, null, null);
    } catch (IllegalArgumentException e) {
      if (job != null)
        try {
          if (inventory.pending(CoreClient.uuid(job, "id")))
            throw new IllegalStateException("Prepared inventory operation requires reconciliation");
          ctx.core().ack(job, "failed", CoreClient.object("effect", "none"), null, e.getMessage());
        } catch (Exception failure) {
          ctx.plugin()
              .getLogger()
              .warning("Cannot report safe job failure: " + failure.getMessage());
        }
    } catch (Exception e) {
      // Persistence/network errors may have happened after a mutation. Leave the
      // operation leased for replay from its durable receipt, never compensate blindly.
      ctx.plugin().getLogger().warning("World job awaits reconciliation: " + e.getMessage());
    }
  }

  private Player actor(JsonObject job) throws Exception {
    String account = job.get("actor").getAsString();
    for (Player online : Bukkit.getOnlinePlayers()) {
      JsonObject session = ctx.session(online.getUniqueId());
      if (session.get("account_id").getAsString().equals(account)) {
        Player player = online;
        if (player != null && !player.isDead() && !player.getWorld().getName().equals("holding")) {
          if (inventory.requiresRecovery(player.getUniqueId()))
            throw new IllegalStateException("Inventory is quarantined");
          return player;
        }
      }
    }
    throw new IllegalArgumentException("公式SMPでプレイ中に操作してください。");
  }

  private void combat(Player player) throws Exception {
    if (ctx.inCombat(player.getUniqueId()))
      throw new IllegalArgumentException("PvP直後は30秒間移動できません。");
    JsonObject session = ctx.session(player.getUniqueId());
    if (session.has("combat_until")
        && !session.get("combat_until").isJsonNull()
        && Instant.parse(session.get("combat_until").getAsString()).isAfter(Instant.now()))
      throw new IllegalArgumentException("PvP直後は30秒間移動できません。");
  }

  private JsonObject execute(JsonObject job) throws Exception {
    String kind = job.get("kind").getAsString();
    JsonObject payload = job.getAsJsonObject("payload");
    UUID id = CoreClient.uuid(job, "id");
    switch (kind) {
      case "claim.sync":
        claims.apply();
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
            throw new IllegalArgumentException("この場所にホームを登録できません。");
          JsonObject location = SpawnPolicy.location(point);
          location.addProperty("world_id", worldId);
          return CoreClient.object("effect", "committed", "location", location);
        }
      case "home.travel":
        {
          Player player = actor(job);
          combat(player);
          Location target = spawns.decode(payload.getAsJsonObject("location"));
          if (target == null) throw new IllegalArgumentException("ホームのワールドがありません。");
          target.getChunk().load();
          if (!target.getBlock().isPassable()
              || !target.clone().add(0, 1, 0).getBlock().isPassable())
            throw new IllegalArgumentException("ホームが塞がれています。");
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
            throw new IllegalArgumentException("移動先のプレイヤーが見つかりません。");
          combat(target);
          spawns.teleport(player, target.getLocation());
          return CoreClient.object("effect", "committed");
        }
      case "npc.sell":
        {
          Player player = actor(job);
          ItemStack[] after =
              InventoryTransactions.copy(player.getInventory().getStorageContents());
          Material material = Material.matchMaterial(payload.get("material").getAsString());
          if (material == null) throw new IllegalArgumentException("この素材は扱えません。");
          int amount = payload.get("amount").getAsInt();
          InventoryTransactions.remove(after, material, amount);
          return inventory.commit(
              player, id, after, CoreClient.object("effect", "committed", "removed", amount));
        }
      case "asset.capture":
        {
          if (!payload.get("kind").getAsString().equals("items"))
            throw new IllegalStateException("Building capture handler is not installed");
          Player player = actor(job);
          ItemStack[] after =
              InventoryTransactions.copy(player.getInventory().getStorageContents());
          JsonObject selection = payload.getAsJsonObject("selection");
          int slot =
              selection.has("slot")
                  ? selection.get("slot").getAsInt()
                  : player.getInventory().getHeldItemSlot();
          if (slot < 0 || slot >= after.length || after[slot] == null || after[slot].isEmpty())
            throw new IllegalArgumentException("預けるアイテムを手に持ってください。");
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
