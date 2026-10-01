package com.lkjsxc.lkjmc.paper;

import com.google.gson.*;
import com.lkjsxc.lkjmc.common.*;
import java.util.*;
import org.bukkit.*;
import org.bukkit.entity.Player;
import org.bukkit.inventory.ItemStack;
import org.bukkit.persistence.PersistentDataType;

/** Inventory and the operation nonce are written in the same native player NBT file. */
public final class InventoryTransactions {
  private final Journal journal;
  private final NamespacedKey marker;
  private final PaperContext ctx;
  private final Map<UUID, JsonObject> recovering = new java.util.concurrent.ConcurrentHashMap<>();

  public InventoryTransactions(PaperContext ctx) throws Exception {
    this.ctx = ctx;
    journal = new Journal(ctx.plugin().getDataFolder().toPath().resolve("inventory-journal"));
    marker = new NamespacedKey(ctx.plugin(), "last-inventory-job");
    for (JsonObject row : journal.unfinished()) {
      UUID player = CoreClient.uuid(row, "native_uuid");
      if (recovering.put(player, row) != null)
        throw new IllegalStateException("Multiple pending inventory transactions for one player");
    }
  }

  public boolean requiresRecovery(UUID nativeId) {
    return recovering.containsKey(nativeId);
  }

  public boolean pending(UUID job) throws Exception {
    return journal
        .read(job)
        .filter(row -> !CoreClient.string(row, "phase", "").equals("committed"))
        .isPresent();
  }

  public Optional<JsonObject> receipt(UUID id) throws Exception {
    Optional<JsonObject> stored = journal.read(id);
    return stored
        .filter(v -> CoreClient.string(v, "phase", "").equals("committed"))
        .map(v -> v.getAsJsonObject("result").deepCopy());
  }

  public void recover(Player player) throws Exception {
    JsonObject row = recovering.get(player.getUniqueId());
    if (row == null) return;
    apply(player, row);
  }

  public JsonObject commit(Player player, UUID job, ItemStack[] after, JsonObject receipt)
      throws Exception {
    Optional<JsonObject> old = receipt(job);
    if (old.isPresent()) return old.get();
    if (requiresRecovery(player.getUniqueId()))
      throw new IllegalStateException("Inventory reconciliation is pending");
    JsonObject row =
        CoreClient.object(
            "id",
            job,
            "native_uuid",
            player.getUniqueId(),
            "phase",
            "prepared",
            "before",
            encode(player.getInventory().getStorageContents()),
            "after",
            encode(after),
            "result",
            receipt);
    // Once prepared, failures stay pending. They must never be acknowledged as effect=none.
    journal.write(job, row);
    Faults.hit(ctx, "inventory.prepared");
    recovering.put(player.getUniqueId(), row);
    try {
      apply(player, row);
      return receipt;
    } catch (Exception error) {
      player.kick(net.kyori.adventure.text.Component.text("持ち物を安全に保存できませんでした。復旧後に接続し直してください。"));
      throw error;
    }
  }

  private void apply(Player player, JsonObject row) throws Exception {
    String id = row.get("id").getAsString();
    if (!id.equals(player.getPersistentDataContainer().get(marker, PersistentDataType.STRING))) {
      player.closeInventory();
      player.getInventory().setStorageContents(decode(row.get("after").getAsString()));
      player.getPersistentDataContainer().set(marker, PersistentDataType.STRING, id);
    }
    WorldDurability.flush(List.of(), List.of(player));
    Faults.hit(ctx, "inventory.flushed");
    row.addProperty("phase", "committed");
    journal.write(UUID.fromString(id), row);
    Faults.hit(ctx, "inventory.committed");
    recovering.remove(player.getUniqueId());
  }

  public static String encode(ItemStack[] contents) {
    return Base64.getEncoder().encodeToString(ItemStack.serializeItemsAsBytes(contents));
  }

  public static ItemStack[] decode(String encoded) {
    return ItemStack.deserializeItemsFromBytes(Base64.getDecoder().decode(encoded));
  }

  public static ItemStack[] copy(ItemStack[] contents) {
    return Arrays.stream(contents).map(i -> i == null ? null : i.clone()).toArray(ItemStack[]::new);
  }

  public static void remove(ItemStack[] contents, Material material, int amount) {
    int remaining = amount;
    for (int i = 0; i < contents.length && remaining > 0; i++) {
      ItemStack item = contents[i];
      if (item == null || item.getType() != material || item.hasItemMeta()) continue;
      int take = Math.min(remaining, item.getAmount());
      remaining -= take;
      item.setAmount(item.getAmount() - take);
      if (item.getAmount() == 0) contents[i] = null;
    }
    if (remaining != 0) throw new IllegalArgumentException("対象の通常素材が足りません。名前付き・特殊アイテムは自動売却しません。");
  }

  public static void add(ItemStack[] contents, ItemStack incoming) {
    int remaining = incoming.getAmount();
    for (ItemStack item : contents)
      if (item != null && item.isSimilar(incoming)) {
        int n = Math.min(remaining, item.getMaxStackSize() - item.getAmount());
        item.setAmount(item.getAmount() + n);
        remaining -= n;
      }
    for (int i = 0; i < contents.length && remaining > 0; i++)
      if (contents[i] == null || contents[i].isEmpty()) {
        contents[i] = incoming.clone();
        contents[i].setAmount(Math.min(remaining, incoming.getMaxStackSize()));
        remaining -= contents[i].getAmount();
      }
    if (remaining != 0) throw new IllegalArgumentException("持ち物に空きがありません。資産は預かり中のままです。");
  }
}
