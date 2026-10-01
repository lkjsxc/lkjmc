package com.lkjsxc.lkjmc.paper;

import com.google.gson.*;
import com.lkjsxc.lkjmc.common.*;
import java.nio.channels.FileChannel;
import java.nio.charset.StandardCharsets;
import java.nio.file.*;
import java.time.Instant;
import java.util.*;
import java.util.concurrent.Callable;
import java.util.concurrent.TimeUnit;
import net.kyori.adventure.text.Component;
import org.bukkit.*;
import org.bukkit.entity.Player;
import org.bukkit.inventory.ItemStack;

/** A private End and its physical entry cost have separate durable subtransactions. */
final class AdventureTransactions {
  private final PaperContext ctx;
  private final SpawnPolicy spawns;
  private final InventoryTransactions inventory;
  private final Journal worlds;
  private final Map<UUID, UUID> started = new java.util.concurrent.ConcurrentHashMap<>();
  private final Map<String, Long> warned = new HashMap<>();

  AdventureTransactions(PaperContext ctx, SpawnPolicy spawns, InventoryTransactions inventory)
      throws Exception {
    this.ctx = ctx;
    this.spawns = spawns;
    this.inventory = inventory;
    worlds = new Journal(ctx.plugin().getDataFolder().toPath().resolve("adventure-worlds"));
    for (JsonObject row : worlds.unfinished())
      started.put(CoreClient.uuid(row, "prepare_job"), CoreClient.uuid(row, "id"));
  }

  boolean pending(UUID job) {
    return started.containsKey(job);
  }

  static boolean handles(JsonObject job) {
    return job.get("kind").getAsString().startsWith("adventure.");
  }

  static UUID eyesJob(UUID prepare) {
    return UUID.nameUUIDFromBytes(
        ("lkjmc:adventure-eyes:" + prepare).getBytes(StandardCharsets.UTF_8));
  }

  private JsonObject current(UUID id) {
    for (JsonElement item : ctx.projection().getAsJsonArray("adventures"))
      if (item.getAsJsonObject().get("id").getAsString().equals(id.toString()))
        return item.getAsJsonObject();
    return null;
  }

  static boolean permits(PaperContext ctx, UUID nativeId, World world) {
    if (!world.getName().startsWith("adventure_")) return true;
    try {
      String id = world.getName().substring("adventure_".length());
      String account = ctx.session(nativeId).get("account_id").getAsString();
      for (JsonElement item : ctx.projection().getAsJsonArray("adventures")) {
        JsonObject a = item.getAsJsonObject();
        if (!a.get("id").getAsString().equals(id)
            || !a.get("state").getAsString().equals("active")
            || !Instant.parse(a.get("expires_at").getAsString()).isAfter(Instant.now())) continue;
        for (JsonElement p : a.getAsJsonArray("participants"))
          if (p.getAsJsonObject().get("account_id").getAsString().equals(account)
              && p.getAsJsonObject().get("ready").getAsBoolean()) return true;
      }
    } catch (Exception ignored) {
    }
    return false;
  }

  JsonObject execute(JsonObject job, Callable<Player> actor) throws Exception {
    UUID id = CoreClient.uuid(job.getAsJsonObject("payload"), "adventure_id");
    return switch (job.get("kind").getAsString()) {
      case "adventure.prepare" -> prepare(job, id, actor);
      case "adventure.cancel" -> cancel(job, id);
      case "adventure.close" -> close(id);
      case "adventure.join" -> join(id, actor);
      default -> throw new IllegalStateException("Unknown adventure operation");
    };
  }

  private JsonObject prepare(JsonObject job, UUID id, Callable<Player> actor) throws Exception {
    UUID prepare = CoreClient.uuid(job, "id"), eyes = eyesJob(prepare);
    JsonObject a = current(id);
    if (a == null || !Set.of("preparing", "activating").contains(a.get("state").getAsString()))
      throw new IllegalStateException("冒険は取り消し中です。返却処理を待っています。");
    JsonObject saved = worlds.read(id).orElse(null);
    if (saved == null) {
      ctx.main(
          () -> {
            Player p = actor.call();
            if (ctx.inCombat(p.getUniqueId()))
              throw new IllegalArgumentException("PvP直後は冒険を準備できません。");
            for (JsonElement member : a.getAsJsonArray("participants")) {
              JsonObject m = member.getAsJsonObject();
              boolean online = false;
              for (Player other : Bukkit.getOnlinePlayers())
                if (!other.isDead()
                    && !other.getWorld().getName().equals("holding")
                    && !ctx.inCombat(other.getUniqueId())
                    && ctx.session(other.getUniqueId())
                        .get("account_id")
                        .equals(m.get("account_id"))) online = true;
              if (!m.get("ready").getAsBoolean() || !online)
                throw new IllegalArgumentException("冒険の参加者全員がSMPで準備完了にしてください。");
            }
            ItemStack[] after = InventoryTransactions.copy(p.getInventory().getStorageContents());
            InventoryTransactions.remove(after, Material.ENDER_EYE, 12);
            Path expected = path(id);
            if (Files.exists(expected)) throw new IllegalStateException("新しい冒険の保存先がすでに存在します。");
            worlds.write(
                id,
                CoreClient.object(
                    "id",
                    id,
                    "prepare_job",
                    prepare,
                    "phase",
                    "prepared",
                    "seed",
                    new java.security.SecureRandom().nextLong()));
            started.put(prepare, id);
            inventory.commit(
                p, eyes, after, CoreClient.object("effect", "committed", "removed", 12));
            return null;
          });
      saved = worlds.read(id).orElseThrow();
    }
    if (inventory.pending(eyes)) throw new IllegalStateException("準備アイテムの保存を回復中です。本人の再接続を待っています。");
    if (inventory.receipt(eyes).isEmpty()) {
      ctx.main(
          () -> {
            Player p = actor.call();
            ItemStack[] after = InventoryTransactions.copy(p.getInventory().getStorageContents());
            InventoryTransactions.remove(after, Material.ENDER_EYE, 12);
            inventory.commit(
                p, eyes, after, CoreClient.object("effect", "committed", "removed", 12));
            return null;
          });
    }
    Faults.hit(ctx, "adventure.eyes_removed");
    JsonObject row = saved;
    World world = ctx.main(() -> load(id, row));
    if (!CoreClient.string(row, "phase", "").equals("ready")) {
      world.getChunkAtAsync(6, 0, true).get(90, TimeUnit.SECONDS);
      world.getChunkAtAsync(6, -1, true).get(90, TimeUnit.SECONDS);
      ctx.main(
          () -> {
            for (int x = 98; x <= 102; x++)
              for (int z = -2; z <= 2; z++)
                for (int y = 49; y <= 52; y++)
                  world
                      .getBlockAt(x, y, z)
                      .setType(y == 49 ? Material.OBSIDIAN : Material.AIR, false);
            world.setSpawnLocation(100, 50, 0);
            WorldDurability.flush(
                List.of(world.getChunkAt(6, 0), world.getChunkAt(6, -1)), List.of());
            return null;
          });
      row.addProperty("native_uuid", world.getUID().toString());
      row.addProperty("phase", "ready");
      worlds.write(id, row);
    }
    Faults.hit(ctx, "adventure.world_ready");
    return CoreClient.object(
        "effect",
        "committed",
        "world_ready",
        true,
        "eyes_removed",
        12,
        "native_world_id",
        world.getUID(),
        "world_name",
        world.getName());
  }

  private World load(UUID id, JsonObject row) throws Exception {
    String name = "adventure_" + id;
    World world = Bukkit.getWorld(name);
    if (world == null)
      world =
          new WorldCreator(name)
              .environment(World.Environment.THE_END)
              .seed(row.get("seed").getAsLong())
              .createWorld();
    if (world == null
        || world.getEnvironment() != World.Environment.THE_END
        || !world.getWorldPath().toAbsolutePath().normalize().equals(path(id)))
      throw new IllegalStateException("冒険ワールドの保存先を確認できません。");
    if (row.has("native_uuid")
        && !world.getUID().toString().equals(row.get("native_uuid").getAsString()))
      throw new IllegalStateException("冒険ワールドのIDが変わっています。");
    return world;
  }

  private Path path(UUID id) {
    return Bukkit.getServer()
        .getLevelDirectory()
        .toAbsolutePath()
        .normalize()
        .resolve("dimensions/minecraft/adventure_" + id);
  }

  private JsonObject cancel(JsonObject job, UUID id) throws Exception {
    JsonObject a = current(id);
    if (a != null && !a.get("state").getAsString().equals("refunding"))
      throw new IllegalStateException("開始済みの冒険は払い戻せません。");
    JsonObject payload = job.getAsJsonObject("payload");
    UUID eyes = eyesJob(CoreClient.uuid(payload, "prepare_job_id"));
    if (inventory.pending(eyes)) throw new IllegalStateException("準備アイテムの保存状態を本人の再接続時に回復します。");
    boolean removed = inventory.receipt(eyes).isPresent();
    retire(id);
    JsonObject result =
        CoreClient.object(
            "effect", "committed", "materials_returned", true, "eyes_removed", removed ? 12 : 0);
    if (removed)
      result.add(
          "refund_manifest",
          CoreClient.object(
              "version",
              1,
              "kind",
              "items",
              "items",
              InventoryTransactions.encode(new ItemStack[] {new ItemStack(Material.ENDER_EYE, 12)}),
              "summary",
              CoreClient.object("material", "ENDER_EYE", "amount", 12),
              "required_consents",
              List.of()));
    return result;
  }

  private JsonObject close(UUID id) throws Exception {
    JsonObject a = current(id);
    if (a != null
        && a.get("state").getAsString().equals("active")
        && Instant.parse(a.get("expires_at").getAsString()).isAfter(Instant.now()))
      throw new IllegalStateException("冒険の終了時刻になっていません。");
    retire(id);
    return CoreClient.object("effect", "committed", "players_evacuated", true);
  }

  private void retire(UUID id) throws Exception {
    Optional<JsonObject> found = worlds.read(id);
    if (found.isEmpty()) {
      if (Files.exists(path(id))) throw new IllegalStateException("所有を証明できない冒険ワールドは削除しません。");
      return;
    }
    JsonObject row = found.get();
    if (CoreClient.string(row, "phase", "").equals("committed")) return;
    ctx.main(
        () -> {
          spawns.invalidateWorld("adventure_" + id);
          World world = Bukkit.getWorld("adventure_" + id);
          if (world != null) {
            for (Player p : List.copyOf(world.getPlayers())) {
              if (p.isDead()) p.spigot().respawn();
              if (p.isDead()) p.kick(Component.text("冒険が終了しました。再接続すると生活ワールドへ戻ります。"));
              else if (p.getWorld().equals(world)) spawns.returnFromEnd(p, "adventure_closed");
            }
            if (!world.getPlayers().isEmpty()) throw new IllegalStateException("冒険の退出完了を待っています。");
            if (!Bukkit.unloadWorld(world, false))
              throw new IllegalStateException("冒険ワールドの停止を待っています。");
          }
          return null;
        });
    Path directory = path(id);
    if (Files.exists(directory)) {
      Path root =
          Bukkit.getServer().getLevelDirectory().toRealPath().resolve("dimensions/minecraft");
      if (!directory.toRealPath().getParent().equals(root) || Files.isSymbolicLink(directory))
        throw new IllegalStateException("冒険の削除対象が管理範囲外です。");
      try (var files = Files.walk(directory)) {
        for (Path file : files.sorted(Comparator.reverseOrder()).toList()) Files.delete(file);
      }
      try (FileChannel parent = FileChannel.open(directory.getParent(), StandardOpenOption.READ)) {
        parent.force(true);
      }
    }
    row.addProperty("phase", "committed");
    worlds.write(id, row);
    started.remove(CoreClient.uuid(row, "prepare_job"));
  }

  private JsonObject join(UUID id, Callable<Player> actor) throws Exception {
    JsonObject a = current(id);
    if (a == null
        || !a.get("state").getAsString().equals("active")
        || !Instant.parse(a.get("expires_at").getAsString()).isAfter(Instant.now()))
      throw new IllegalArgumentException("この冒険は終了しています。");
    JsonObject row = worlds.read(id).orElseThrow(() -> new IllegalStateException("冒険の保存記録がありません。"));
    World world = ctx.main(() -> load(id, row));
    return ctx.main(
        () -> {
          Player p = actor.call();
          if (ctx.inCombat(p.getUniqueId()))
            throw new IllegalArgumentException("PvP直後は30秒間移動できません。");
          if (!permits(ctx, p.getUniqueId(), world))
            throw new IllegalArgumentException("参加登録と準備完了を確認してください。");
          Location at = new Location(world, 100.5, 50, .5);
          if (!at.getBlock().isPassable()
              || !at.clone().add(0, 1, 0).getBlock().isPassable()
              || !at.clone().add(0, -1, 0).getBlock().getType().isSolid())
            throw new IllegalArgumentException("エンドの入場地点が塞がれているか壊れています。先に参加した人に修復を依頼してください。");
          spawns.teleport(p, at);
          WorldDurability.flush(List.of(), List.of(p));
          return CoreClient.object("effect", "committed");
        });
  }

  void tick() {
    for (Player p : Bukkit.getOnlinePlayers()) {
      if (!p.getWorld().getName().startsWith("adventure_")) continue;
      try {
        if (!permits(ctx, p.getUniqueId(), p.getWorld())) {
          if (!p.isDead()) spawns.returnFromEnd(p, "adventure_closed");
          continue;
        }
        UUID id = UUID.fromString(p.getWorld().getName().substring("adventure_".length()));
        long seconds =
            java.time.Duration.between(
                    Instant.now(), Instant.parse(current(id).get("expires_at").getAsString()))
                .getSeconds();
        long bucket =
            seconds <= 60 ? 60 : seconds <= 300 ? 300 : seconds <= 600 ? 600 : Long.MAX_VALUE;
        String key = id + "/" + p.getUniqueId();
        if (bucket < warned.getOrDefault(key, Long.MAX_VALUE)) {
          warned.put(key, bucket);
          p.sendMessage(
              Component.text(
                  "冒険はあと約"
                      + Math.max(1, (seconds + 59) / 60)
                      + "分で終了します。地面に残したアイテムは消えるので回収してください。"));
        }
      } catch (Exception e) {
        p.kick(Component.text("冒険の状態を確認できません。生活ワールドへの帰還を回復中です。"));
      }
    }
  }
}
