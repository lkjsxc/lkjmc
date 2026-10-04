package com.lkjsxc.lkjmc.paper;

import com.google.gson.*;
import com.lkjsxc.lkjmc.common.*;
import com.sk89q.worldedit.WorldEdit;
import com.sk89q.worldedit.bukkit.BukkitAdapter;
import com.sk89q.worldedit.extent.clipboard.*;
import com.sk89q.worldedit.extent.clipboard.io.BuiltInClipboardFormat;
import com.sk89q.worldedit.extent.transform.BlockTransformExtent;
import com.sk89q.worldedit.math.*;
import com.sk89q.worldedit.math.transform.AffineTransform;
import com.sk89q.worldedit.regions.*;
import com.sk89q.worldedit.util.SideEffect;
import com.sk89q.worldedit.util.SideEffectSet;
import com.sk89q.worldedit.world.block.BlockTypes;
import java.io.*;
import java.nio.file.*;
import java.security.MessageDigest;
import java.util.*;
import org.bukkit.*;
import org.bukkit.block.*;
import org.bukkit.block.data.Bisected;
import org.bukkit.block.data.type.Bed;
import org.bukkit.entity.*;
import org.bukkit.inventory.*;
import org.bukkit.persistence.PersistentDataType;

/** Immutable native snapshots. Only physically escrowed assets may be replayed into a world. */
public final class BuildingStore {
  private static final SideEffectSet EFFECTS =
      SideEffectSet.none()
          .with(SideEffect.LIGHTING, SideEffect.State.ON)
          .with(SideEffect.NETWORK, SideEffect.State.ON)
          .with(SideEffect.POI_UPDATE, SideEffect.State.ON);
  private final PaperContext ctx;
  private final ClaimProtection claims;
  private final Provenance provenance;
  private final Path root;
  private final Journal records;
  private final NamespacedKey placementKey;

  public BuildingStore(PaperContext ctx, ClaimProtection claims, Provenance provenance)
      throws Exception {
    this.ctx = ctx;
    this.claims = claims;
    this.provenance = provenance;
    root = ctx.plugin().getDataFolder().toPath().resolve("building-assets");
    records = new Journal(root);
    placementKey = new NamespacedKey(ctx.plugin(), "building-entity-slot");
  }

  public JsonObject read(UUID id) throws Exception {
    JsonObject result =
        records
            .read(id)
            .orElseThrow(
                () ->
                    new IllegalStateException(
                        com.lkjsxc.lkjmc.common.SystemMessage.of("text.the_building_save_record_is_missing_quarantine_the_asse_260e532265").toString()));
    byte[] bytes = Files.readAllBytes(root.resolve(id + ".schem"));
    if (!sha(bytes).equals(result.get("schematic_sha256").getAsString()))
      throw new IllegalStateException("Building snapshot checksum mismatch");
    if (!Journal.digest(result.get("entities")).equals(result.get("entities_sha256").getAsString()))
      throw new IllegalStateException("Entity snapshot checksum mismatch");
    return result;
  }

  public Optional<JsonObject> existing(UUID id) throws Exception {
    return records.read(id).isPresent() ? Optional.of(read(id)) : Optional.empty();
  }

  public JsonObject claim(UUID id) {
    for (JsonElement e : ctx.projection().getAsJsonArray("claims"))
      if (e.getAsJsonObject().get("id").getAsString().equals(id.toString()))
        return e.getAsJsonObject();
    throw new IllegalArgumentException(com.lkjsxc.lkjmc.common.SystemMessage.of("text.no_protected_claim_was_found").toString());
  }

  private World world(JsonObject claim) {
    for (World world : Bukkit.getWorlds())
      if (claim.get("world_id").getAsString().equals(claims.worldId(world))) return world;
    throw new IllegalArgumentException(com.lkjsxc.lkjmc.common.SystemMessage.of("text.the_claim_s_world_is_not_loaded").toString());
  }

  public WorldLocks.Box source(JsonObject job, Player player) throws Exception {
    JsonObject p = job.getAsJsonObject("payload"),
        c = claim(CoreClient.uuid(p.getAsJsonObject("selection"), "claim_id"));
    if (!c.get("state").getAsString().equals("active") || !c.get("owner").equals(p.get("owner")))
      throw new IllegalArgumentException(com.lkjsxc.lkjmc.common.SystemMessage.of("text.the_claim_s_owner_or_state_has_changed").toString());
    World world = world(c);
    if (p.get("kind").getAsString().equals("land"))
      return new WorldLocks.Box(
          world.getName(),
          c.get("min_x").getAsInt() * 16,
          world.getMinHeight(),
          c.get("min_z").getAsInt() * 16,
          c.get("max_x").getAsInt() * 16 + 15,
          world.getMaxHeight() - 1,
          c.get("max_z").getAsInt() * 16 + 15);
    var local = WorldEdit.getInstance().getSessionManager().get(BukkitAdapter.adapt(player));
    Region region;
    try {
      region = local.getSelection(BukkitAdapter.adapt(world));
    } catch (com.sk89q.worldedit.IncompleteRegionException e) {
      throw new IllegalArgumentException(
          com.lkjsxc.lkjmc.common.SystemMessage.of("text.look_at_opposite_building_corners_and_use_lkjmc_pos1_an_9778a4c9fa").toString());
    }
    if (!(region instanceof CuboidRegion))
      throw new IllegalArgumentException(com.lkjsxc.lkjmc.common.SystemMessage.of("text.select_a_cuboid_building_area").toString());
    var min = region.getMinimumPoint();
    var max = region.getMaximumPoint();
    long volume =
        ((long) max.x() - min.x() + 1)
            * ((long) max.y() - min.y() + 1)
            * ((long) max.z() - min.z() + 1);
    if (volume > ctx.plugin().getConfig().getLong("building-max-volume", 1048576))
      throw new IllegalArgumentException(
          com.lkjsxc.lkjmc.common.SystemMessage.of("text.the_area_is_too_large_to_pack_at_once_split_it_or_ask_a_282d86a8c3").toString());
    if (min.x() < c.get("min_x").getAsInt() * 16
        || max.x() > c.get("max_x").getAsInt() * 16 + 15
        || min.z() < c.get("min_z").getAsInt() * 16
        || max.z() > c.get("max_z").getAsInt() * 16 + 15
        || min.y() < world.getMinHeight()
        || max.y() >= world.getMaxHeight())
      throw new IllegalArgumentException(com.lkjsxc.lkjmc.common.SystemMessage.of("text.the_entire_building_must_fit_inside_the_selected_claim").toString());
    return new WorldLocks.Box(
        world.getName(), min.x(), min.y(), min.z(), max.x(), max.y(), max.z());
  }

  public void noPlayers(WorldLocks.Box box) {
    World world = Objects.requireNonNull(Bukkit.getWorld(box.world()));
    for (Player p : world.getPlayers())
      if (box.contains(p.getLocation()) || box.contains(p.getEyeLocation()))
        throw new IllegalArgumentException(com.lkjsxc.lkjmc.common.SystemMessage.of("text.everyone_must_leave_the_building_area_first").toString());
  }

  public JsonObject capture(JsonObject job, WorldLocks.Box box) throws Exception {
    UUID asset = CoreClient.uuid(job.getAsJsonObject("payload"), "asset_id");
    if (records.read(asset).isPresent()) return read(asset);
    JsonObject payload = job.getAsJsonObject("payload");
    boolean land = payload.get("kind").getAsString().equals("land");
    boolean contents = payload.get("include_contents").getAsBoolean();
    World world = Objects.requireNonNull(Bukkit.getWorld(box.world()));
    noPlayers(box);
    var min = BlockVector3.at(box.minX(), box.minY(), box.minZ());
    var dimensions =
        BlockVector3.at(
            box.maxX() - box.minX() + 1, box.maxY() - box.minY() + 1, box.maxZ() - box.minZ() + 1);
    // An in-place land transfer keeps terrain and blocks. It snapshots ownership-bearing entities.
    var clipboard =
        new BlockArrayClipboard(
            new CuboidRegion(
                BlockVector3.ZERO, land ? BlockVector3.ZERO : dimensions.subtract(1, 1, 1)));
    clipboard.setOrigin(BlockVector3.ZERO);
    JsonArray containers = new JsonArray();
    TreeMap<String, Integer> materials = new TreeMap<>();
    long blocks = 0;
    if (!land)
      for (Chunk chunk : box.chunks())
        for (Block block : provenance.builtBlocks(chunk)) {
          if (!box.contains(block.getLocation()) || block.getType().isAir()) continue;
          BlockVector3 local =
              BlockVector3.at(block.getX(), block.getY(), block.getZ()).subtract(min);
          connected(block, box);
          BlockState state = block.getState();
          if (state instanceof InventoryHolder holder)
            inventory(holder.getInventory(), contents, containers, local.toString());
          if (state instanceof org.bukkit.block.Jukebox jukebox
              && jukebox.getRecord() != null
              && !jukebox.getRecord().isEmpty())
            inventory(
                new ItemStack[] {jukebox.getRecord()}, contents, containers, local.toString());
          clipboard.setBlock(local, BukkitAdapter.adapt(world).getFullBlock(min.add(local)));
          materials.merge(block.getType().name(), 1, Integer::sum);
          blocks++;
        }
    if (land)
      for (Chunk chunk : box.chunks()) {
        for (Block block : provenance.builtBlocks(chunk))
          if (box.contains(block.getLocation()) && !block.getType().isAir()) {
            materials.merge(block.getType().name(), 1, Integer::sum);
            blocks++;
          }
        for (BlockState state : chunk.getTileEntities())
          if (box.contains(state.getLocation()) && state instanceof InventoryHolder holder)
            inventory(
                holder.getInventory(),
                contents,
                containers,
                state.getLocation().toVector().toString());
      }
    JsonArray entities = new JsonArray();
    TreeSet<String> consent = new TreeSet<>();
    List<Entity> selected = new ArrayList<>();
    for (Chunk chunk : box.chunks())
      for (Entity entity : chunk.getEntities())
        if (box.contains(entity.getLocation()) && eligible(entity) && !selected.contains(entity))
          selected.add(entity);
    Set<UUID> ids = new HashSet<>();
    selected.forEach(e -> ids.add(e.getUniqueId()));
    for (Entity entity : selected) {
      if (!entity.getPassengers().stream().allMatch(e -> ids.contains(e.getUniqueId()))
          || entity.getVehicle() != null && !ids.contains(entity.getVehicle().getUniqueId()))
        throw new IllegalArgumentException(
            com.lkjsxc.lkjmc.common.SystemMessage.of("text.dismount_riders_and_vehicles_outside_the_selection_before_packing").toString());
      if (entity instanceof LivingEntity living
          && living.isLeashed()
          && !ids.contains(living.getLeashHolder().getUniqueId()))
        throw new IllegalArgumentException(com.lkjsxc.lkjmc.common.SystemMessage.of("text.include_the_leash_anchor_or_remove_the_leash_first").toString());
      if (entity instanceof InventoryHolder holder)
        inventory(holder.getInventory(), contents, containers, entity.getType().name());
      // Equipment and frames are part of the visible building, even when container contents are
      // excluded.
      JsonObject row =
          CoreClient.object(
              "uuid",
              entity.getUniqueId(),
              "type",
              entity.getType().name(),
              "x",
              Double.toString(entity.getX() - box.minX()),
              "y",
              Double.toString(entity.getY() - box.minY()),
              "z",
              Double.toString(entity.getZ() - box.minZ()),
              "yaw",
              Float.toString(entity.getYaw()),
              "pitch",
              Float.toString(entity.getPitch()),
              "data",
              Base64.getEncoder().encodeToString(Bukkit.getUnsafe().serializeEntity(entity)),
              "passengers",
              entity.getPassengers().stream().map(e -> e.getUniqueId().toString()).toList());
      if (entity instanceof Hanging hanging) row.addProperty("facing", hanging.getFacing().name());
      if (entity instanceof LivingEntity living && living.isLeashed())
        row.addProperty("leash", living.getLeashHolder().getUniqueId().toString());
      if (entity instanceof Tameable pet && pet.getOwner() != null) {
        UUID owner = owner(pet.getOwner().getUniqueId());
        row.addProperty("owner", owner.toString());
        if (!owner.toString().equals(job.get("actor").getAsString())) consent.add(owner.toString());
      }
      entities.add(row);
    }
    if (!land && blocks == 0 && entities.isEmpty())
      throw new IllegalArgumentException(
          com.lkjsxc.lkjmc.common.SystemMessage.of("text.the_selection_contains_no_player_placed_building_or_tra_9f6dbc76e6").toString());
    ByteArrayOutputStream buffer = new ByteArrayOutputStream();
    try (var writer = BuiltInClipboardFormat.SPONGE_V3_SCHEMATIC.getWriter(buffer)) {
      writer.write(clipboard);
    }
    byte[] bytes = buffer.toByteArray();
    JsonArray entitySummary = new JsonArray();
    for (JsonElement e : entities) {
      JsonObject row = e.getAsJsonObject();
      Entity actual = Bukkit.getEntity(CoreClient.uuid(row, "uuid"));
      JsonObject summary =
          CoreClient.object(
              "type",
              row.get("type"),
              "name",
              actual == null
                  ? ""
                  : net.kyori.adventure.text.serializer.plain.PlainTextComponentSerializer
                      .plainText()
                      .serialize(actual.name()),
              "owner",
              row.get("owner"));
      if (actual instanceof AbstractVillager villager) {
        JsonArray trades = new JsonArray();
        for (MerchantRecipe recipe : villager.getRecipes())
          trades.add(
              CoreClient.object(
                  "result",
                  itemSummary(recipe.getResult()),
                  "ingredients",
                  recipe.getIngredients().stream().map(this::itemSummary).toList(),
                  "uses",
                  recipe.getUses(),
                  "max_uses",
                  recipe.getMaxUses()));
        summary.add("trades", trades);
      }
      if (actual instanceof ItemFrame frame && !frame.getItem().isEmpty())
        summary.add("item", itemSummary(frame.getItem()));
      if (actual instanceof ArmorStand armor)
        summary.add(
            "equipment",
            CoreClient.JSON.toJsonTree(
                Arrays.stream(armor.getEquipment().getArmorContents())
                    .filter(i -> i != null && !i.isEmpty())
                    .map(this::itemSummary)
                    .toList()));
      entitySummary.add(summary);
    }
    JsonObject manifest =
        CoreClient.object(
            "asset_id",
            asset,
            "version",
            1,
            "kind",
            land ? "land" : "building",
            "source",
            box.json(),
            "dimensions",
            List.of(dimensions.x(), dimensions.y(), dimensions.z()),
            "blocks",
            blocks,
            "materials",
            materials,
            "containers",
            containers,
            "entities",
            entitySummary,
            "required_consents",
            consent,
            "schematic_sha256",
            sha(bytes),
            "entities_sha256",
            Journal.digest(entities),
            "minecraft_version",
            Bukkit.getMinecraftVersion());
    manifest.addProperty("contents_included", contents);
    if (CoreClient.JSON.toJson(manifest).getBytes(java.nio.charset.StandardCharsets.UTF_8).length
        > 180000)
      throw new IllegalArgumentException(
          com.lkjsxc.lkjmc.common.SystemMessage.of("text.the_container_inventory_description_is_too_large_split_80c0eb243d").toString());
    JsonObject record =
        CoreClient.object(
            "id",
            asset,
            "phase",
            "committed",
            "source",
            box.json(),
            "manifest",
            manifest,
            "entities",
            entities,
            "schematic_sha256",
            sha(bytes),
            "entities_sha256",
            Journal.digest(entities));
    Journal.atomic(root.resolve(asset + ".schem"), bytes);
    records.write(asset, record);
    return record;
  }

  private boolean eligible(Entity e) {
    return e instanceof Animals
        || e instanceof WaterMob
        || e instanceof AbstractVillager
        || e instanceof Golem
        || e instanceof Allay
        || e instanceof ItemFrame
        || e instanceof Painting
        || e instanceof ArmorStand
        || e instanceof LeashHitch;
  }

  private UUID owner(UUID nativeId) {
    for (JsonElement e : ctx.projection().getAsJsonArray("native_owners")) {
      JsonObject row = e.getAsJsonObject();
      if (row.get("status").getAsString().equals("active")
          && row.get("native_uuid").getAsString().equals(nativeId.toString()))
        return CoreClient.uuid(row, "account_id");
    }
    throw new IllegalArgumentException(
        com.lkjsxc.lkjmc.common.SystemMessage.of("text.the_pet_owner_s_game_identity_could_not_be_verified_ask_4bbb71e86b").toString());
  }

  public UUID nativeOwner(UUID account) {
    for (JsonElement e : ctx.projection().getAsJsonArray("native_owners")) {
      JsonObject row = e.getAsJsonObject();
      if (row.get("account_id").getAsString().equals(account.toString())
          && row.get("status").getAsString().equals("active"))
        return CoreClient.uuid(row, "native_uuid");
    }
    throw new IllegalArgumentException(com.lkjsxc.lkjmc.common.SystemMessage.of("text.the_recipient_s_game_data_could_not_be_verified").toString());
  }

  private void inventory(Inventory inventory, boolean include, JsonArray summary, String at) {
    inventory(inventory.getContents(), include, summary, at);
  }

  private void inventory(ItemStack[] inventory, boolean include, JsonArray summary, String at) {
    for (ItemStack item : inventory)
      if (item != null && !item.isEmpty()) {
        if (!include)
          throw new IllegalArgumentException(
              com.lkjsxc.lkjmc.common.SystemMessage.of("text.empty_containers_before_depositing_if_their_contents_are_excluded").toString());
        JsonObject info = itemSummary(item);
        info.addProperty("at", at);
        info.add("description", Bukkit.getUnsafe().serializeItemAsJson(item));
        summary.add(info);
      }
  }

  private JsonObject itemSummary(ItemStack item) {
    TreeMap<String, Integer> enchantments = new TreeMap<>();
    item.getEnchantments().forEach((e, level) -> enchantments.put(e.getKey().toString(), level));
    String name =
        item.hasItemMeta() && item.getItemMeta().hasDisplayName()
            ? net.kyori.adventure.text.serializer.plain.PlainTextComponentSerializer.plainText()
                .serialize(item.getItemMeta().displayName())
            : item.getType().name();
    return CoreClient.object(
        "material",
        item.getType().name(),
        "name",
        name,
        "amount",
        item.getAmount(),
        "enchantments",
        enchantments);
  }

  private void connected(Block b, WorldLocks.Box box) {
    List<Block> mates = new ArrayList<>();
    if (b.getBlockData() instanceof Bed bed)
      mates.add(
          b.getRelative(
              bed.getPart() == Bed.Part.FOOT
                  ? bed.getFacing()
                  : bed.getFacing().getOppositeFace()));
    else if (b.getBlockData() instanceof Bisected bisected
        && !(b.getBlockData() instanceof org.bukkit.block.data.type.Stairs)
        && !(b.getBlockData() instanceof org.bukkit.block.data.type.TrapDoor))
      mates.add(
          b.getRelative(
              bisected.getHalf() == Bisected.Half.BOTTOM ? BlockFace.UP : BlockFace.DOWN));
    if (b.getState() instanceof org.bukkit.block.Chest chest
        && chest.getInventory().getHolder() instanceof DoubleChest pair) {
      if (pair.getLeftSide() instanceof org.bukkit.block.Chest left) mates.add(left.getBlock());
      if (pair.getRightSide() instanceof org.bukkit.block.Chest right) mates.add(right.getBlock());
    }
    for (Block mate : mates)
      if (!box.contains(mate.getLocation()) || !provenance.built(mate))
        throw new IllegalArgumentException(
            com.lkjsxc.lkjmc.common.SystemMessage.of("text.include_all_parts_of_beds_doors_and_connected_containers").toString());
  }

  public Clipboard clipboard(UUID id) throws Exception {
    read(id);
    try (var reader =
        BuiltInClipboardFormat.SPONGE_V3_SCHEMATIC.getReader(
            Files.newInputStream(root.resolve(id + ".schem")))) {
      return reader.read();
    }
  }

  public static String sha(byte[] bytes) throws Exception {
    return HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(bytes));
  }

  public record Placement(
      WorldLocks.Box box, BlockVector3 origin, AffineTransform rotation, int degrees) {}

  public Placement placement(JsonObject p, JsonObject record, UUID actor) {
    JsonObject c = claim(CoreClient.uuid(p, "claim_id"));
    World world = world(c);
    if (!c.get("state").getAsString().equals("active"))
      throw new IllegalArgumentException(com.lkjsxc.lkjmc.common.SystemMessage.of("text.the_destination_claim_still_has_work_in_progress").toString());
    int rotation = p.has("rotation") ? p.get("rotation").getAsInt() : 0;
    if (!Set.of(0, 90, 180, 270).contains(rotation))
      throw new IllegalArgumentException(com.lkjsxc.lkjmc.common.SystemMessage.of("text.choose_a_rotation_of_0_90_180_or_270_degrees").toString());
    int x = p.get("x").getAsInt(), y = p.get("y").getAsInt(), z = p.get("z").getAsInt();
    if (Math.abs((long) x) > 29999872
        || Math.abs((long) z) > 29999872
        || y < world.getMinHeight()
        || y >= world.getMaxHeight())
      throw new IllegalArgumentException(com.lkjsxc.lkjmc.common.SystemMessage.of("text.the_placement_origin_is_outside_the_world_bounds").toString());
    JsonArray d = record.getAsJsonObject("manifest").getAsJsonArray("dimensions");
    int w = d.get(0).getAsInt(), h = d.get(1).getAsInt(), l = d.get(2).getAsInt();
    // Clockwise, with the entered origin always being the minimum corner of the rotated footprint.
    AffineTransform transform = new AffineTransform().rotateY(-rotation);
    var far = rounded(transform.apply(Vector3.at(w - 1, 0, l - 1)));
    BlockVector3 origin = BlockVector3.at(x - Math.min(0, far.x()), y, z - Math.min(0, far.z()));
    WorldLocks.Box box =
        new WorldLocks.Box(
            world.getName(),
            x,
            y,
            z,
            x + (rotation % 180 == 0 ? w : l) - 1,
            y + h - 1,
            z + (rotation % 180 == 0 ? l : w) - 1);
    if (box.maxY() >= world.getMaxHeight()
        || !world.getWorldBorder().isInside(new Location(world, box.minX(), y, box.minZ()))
        || !world.getWorldBorder().isInside(new Location(world, box.maxX(), y, box.maxZ())))
      throw new IllegalArgumentException(com.lkjsxc.lkjmc.common.SystemMessage.of("text.the_placement_area_extends_beyond_the_world_bounds").toString());
    for (int cx = box.minX() >> 4; cx <= box.maxX() >> 4; cx++)
      for (int cz = box.minZ() >> 4; cz <= box.maxZ() >> 4; cz++) {
        Block block = world.getBlockAt(cx * 16, y, cz * 16);
        JsonObject plot = claims.claim(block);
        if (plot == null || !plot.get("id").equals(c.get("id")) || !claims.canBuild(actor, block))
          throw new IllegalArgumentException(
              com.lkjsxc.lkjmc.common.SystemMessage.of("text.the_entire_building_must_fit_inside_a_destination_claim_30004dadcf").toString());
      }
    return new Placement(box, origin, transform, rotation);
  }

  public boolean clear(Placement placement) {
    WorldLocks.Box box = placement.box();
    World world = Objects.requireNonNull(Bukkit.getWorld(box.world()));
    for (int x = box.minX(); x <= box.maxX(); x++)
      for (int z = box.minZ(); z <= box.maxZ(); z++)
        for (int y = box.minY(); y <= box.maxY(); y++)
          if (!world.getBlockAt(x, y, z).getType().isAir()) return false;
    for (Chunk chunk : box.chunks())
      for (Entity e : chunk.getEntities()) if (box.contains(e.getLocation())) return false;
    return true;
  }

  public void remove(JsonObject record, Clipboard clipboard) throws Exception {
    WorldLocks.Box box = WorldLocks.Box.read(record.getAsJsonObject("source"));
    World world = Objects.requireNonNull(Bukkit.getWorld(box.world()));
    var edit = BukkitAdapter.adapt(world);
    for (BlockVector3 local : clipboard.getRegion())
      if (!clipboard.getBlock(local).getBlockType().getMaterial().isAir()) {
        var at = local.add(box.minX(), box.minY(), box.minZ());
        edit.setBlock(at, BlockTypes.AIR.getDefaultState(), EFFECTS);
        provenance.mark(world.getBlockAt(at.x(), at.y(), at.z()), false);
      }
    box.chunks(); // All source entities are loaded before UUID tombstones are applied.
    for (JsonElement e : record.getAsJsonArray("entities")) {
      Entity entity = Bukkit.getEntity(CoreClient.uuid(e.getAsJsonObject(), "uuid"));
      if (entity != null) entity.remove();
    }
  }

  public void place(
      JsonObject record, Clipboard clipboard, Placement placement, UUID asset, UUID actor)
      throws Exception {
    World world = Objects.requireNonNull(Bukkit.getWorld(placement.box().world()));
    var edit = BukkitAdapter.adapt(world);
    for (BlockVector3 local : clipboard.getRegion())
      if (!clipboard.getBlock(local).getBlockType().getMaterial().isAir()) {
        var at = placement.origin().add(rounded(placement.rotation().apply(local.toVector3())));
        edit.setBlock(
            at,
            BlockTransformExtent.transform(clipboard.getFullBlock(local), placement.rotation()),
            EFFECTS);
        provenance.mark(world.getBlockAt(at.x(), at.y(), at.z()), true);
      }
    Map<String, Entity> restored = new LinkedHashMap<>();
    Map<String, Entity> existing = new HashMap<>();
    for (Chunk chunk : placement.box().chunks())
      for (Entity e : chunk.getEntities()) {
        String mark = e.getPersistentDataContainer().get(placementKey, PersistentDataType.STRING);
        if (mark != null) {
          if (existing.put(mark, e) != null)
            throw new IllegalStateException("Duplicate placed entity marker");
        }
      }
    for (JsonElement value : record.getAsJsonArray("entities")) {
      JsonObject row = value.getAsJsonObject();
      String original = row.get("uuid").getAsString(), marker = asset + ":" + original;
      Entity entity = existing.get(marker);
      if (entity == null) {
        entity =
            Bukkit.getUnsafe()
                .deserializeEntity(
                    Base64.getDecoder().decode(row.get("data").getAsString()), world, false, false);
        if (entity.isInWorld())
          throw new IllegalStateException(
              "Deserializer spawned before an idempotency marker was set");
        entity.getPersistentDataContainer().set(placementKey, PersistentDataType.STRING, marker);
        if (entity instanceof Tameable pet && row.has("owner"))
          ctx.assignPetOwner(pet, nativeOwner(actor));
        if (entity instanceof Hanging hanging && row.has("facing")) {
          BlockFace face = BlockFace.valueOf(row.get("facing").getAsString());
          var direction =
              placement
                  .rotation()
                  .apply(Vector3.at(face.getModX(), face.getModY(), face.getModZ()))
                  .toBlockPoint();
          for (BlockFace candidate :
              List.of(
                  BlockFace.NORTH,
                  BlockFace.SOUTH,
                  BlockFace.EAST,
                  BlockFace.WEST,
                  BlockFace.UP,
                  BlockFace.DOWN))
            if (candidate.getModX() == direction.x()
                && candidate.getModY() == direction.y()
                && candidate.getModZ() == direction.z())
              hanging.setFacingDirection(candidate, true);
        }
        var relative =
            placement
                .rotation()
                .apply(
                    Vector3.at(
                        row.get("x").getAsDouble() - .5,
                        row.get("y").getAsDouble(),
                        row.get("z").getAsDouble() - .5));
        Location at =
            new Location(
                world,
                placement.origin().x() + relative.x() + .5,
                placement.origin().y() + relative.y(),
                placement.origin().z() + relative.z() + .5,
                row.get("yaw").getAsFloat() + placement.degrees(),
                row.get("pitch").getAsFloat());
        if (!entity.spawnAt(at, org.bukkit.event.entity.CreatureSpawnEvent.SpawnReason.CUSTOM))
          throw new IllegalStateException("Could not restore entity slot " + original);
      }
      restored.put(original, entity);
    }
    for (JsonElement value : record.getAsJsonArray("entities")) {
      JsonObject row = value.getAsJsonObject();
      Entity entity = restored.get(row.get("uuid").getAsString());
      for (JsonElement passenger : row.getAsJsonArray("passengers")) {
        Entity p = restored.get(passenger.getAsString());
        if (p.getVehicle() != entity && !entity.addPassenger(p))
          throw new IllegalStateException("Could not restore passenger");
      }
      if (row.has("leash")
          && entity instanceof LivingEntity living
          && !living.setLeashHolder(restored.get(row.get("leash").getAsString())))
        throw new IllegalStateException("Could not restore leash");
    }
  }

  public void transferPets(UUID asset, UUID buyer) throws Exception {
    JsonObject record = read(asset);
    WorldLocks.Box box = WorldLocks.Box.read(record.getAsJsonObject("source"));
    box.chunks();
    for (JsonElement value : record.getAsJsonArray("entities")) {
      JsonObject row = value.getAsJsonObject();
      if (!row.has("owner")) continue;
      Entity entity = Bukkit.getEntity(CoreClient.uuid(row, "uuid"));
      if (!(entity instanceof Tameable pet))
        throw new IllegalStateException("Sale pet is missing from quarantined land");
      ctx.assignPetOwner(pet, nativeOwner(buyer));
    }
    WorldDurability.flush(box.chunks(), List.of());
  }

  private static BlockVector3 rounded(Vector3 v) {
    return BlockVector3.at(Math.round(v.x()), Math.round(v.y()), Math.round(v.z()));
  }
}
