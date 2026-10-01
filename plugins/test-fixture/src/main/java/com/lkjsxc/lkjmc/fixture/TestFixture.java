package com.lkjsxc.lkjmc.fixture;

import com.google.gson.*;
import com.lkjsxc.lkjmc.common.CoreClient;
import java.util.*;
import org.bukkit.*;
import org.bukkit.block.*;
import org.bukkit.command.*;
import org.bukkit.entity.*;
import org.bukkit.inventory.*;
import org.bukkit.persistence.PersistentDataType;
import org.bukkit.plugin.java.JavaPlugin;

/**
 * A separate artifact. Never installed by production deployment. Does not acknowledge world jobs.
 */
public final class TestFixture extends JavaPlugin {
  @Override
  public void onEnable() {
    if (!Boolean.getBoolean("lkjmc.testFaults")
        || !Bukkit.getIp().equals("127.0.0.1")
        || Bukkit.getOnlineMode())
      throw new IllegalStateException("Fixtures require the explicit private offline loopback rig");
    getCommand("lkjmcfixture").setExecutor(this);
  }

  @Override
  public boolean onCommand(CommandSender sender, Command command, String label, String[] a) {
    if (!(sender instanceof ConsoleCommandSender)) return false;
    try {
      World world = Objects.requireNonNull(Bukkit.getWorld("living"));
      int x = Integer.parseInt(a[1]), y = Integer.parseInt(a[2]), z = Integer.parseInt(a[3]);
      if (a[0].equals("entities")) {
        Villager villager =
            world.spawn(
                new Location(world, x + .5, y, z + 1.5),
                Villager.class,
                e -> {
                  e.setAI(false);
                  e.setProfession(Villager.Profession.LIBRARIAN);
                  e.setVillagerLevel(3);
                  e.setVillagerExperience(80);
                  e.setPersistent(true);
                });
        MerchantRecipe recipe =
            new MerchantRecipe(new ItemStack(Material.DIAMOND, 2), 3, 12, true, 5, 0.05f);
        recipe.addIngredient(new ItemStack(Material.EMERALD, 7));
        villager.setRecipes(List.of(recipe));
        world.spawn(
            new Location(world, x + 1.5, y, z + 1.5),
            Wolf.class,
            e -> {
              e.setAI(false);
              e.setOwner(Bukkit.getOfflinePlayer(UUID.fromString(a[4])));
              e.setSitting(true);
              e.setCollarColor(DyeColor.PURPLE);
              e.setPersistent(true);
            });
        world.spawn(
            new Location(world, x + 2.5, y, z + 1.5),
            ArmorStand.class,
            e -> {
              e.setGravity(false);
              e.getEquipment().setHelmet(new ItemStack(Material.GOLDEN_HELMET));
              e.setArms(true);
              e.setPersistent(true);
            });
        sender.sendMessage("FIXTURE_CREATED");
      } else if (a[0].equals("bed")) {
        org.bukkit.block.data.type.Bed foot =
            (org.bukkit.block.data.type.Bed) Material.WHITE_BED.createBlockData();
        foot.setFacing(BlockFace.EAST);
        foot.setPart(org.bukkit.block.data.type.Bed.Part.FOOT);
        org.bukkit.block.data.type.Bed head = (org.bukkit.block.data.type.Bed) foot.clone();
        head.setPart(org.bukkit.block.data.type.Bed.Part.HEAD);
        world.getBlockAt(x, y, z).setBlockData(foot, false);
        world.getBlockAt(x + 1, y, z).setBlockData(head, false);
        Objects.requireNonNull(Bukkit.getPlayer(a[4]))
            .setRespawnLocation(new Location(world, x + 1, y, z), false);
        sender.sendMessage("FIXTURE_BED");
      } else if (a[0].equals("inspect")) {
        int w = Integer.parseInt(a[4]), h = Integer.parseInt(a[5]), l = Integer.parseInt(a[6]);
        if (w * h * l > 32768) throw new IllegalArgumentException("Inspection too large");
        JsonArray blocks = new JsonArray(), entities = new JsonArray();
        Set<Entity> seen = new HashSet<>();
        for (int bx = x; bx < x + w; bx++)
          for (int bz = z; bz < z + l; bz++) {
            seen.addAll(Arrays.asList(world.getChunkAt(bx >> 4, bz >> 4).getEntities()));
            for (int by = y; by < y + h; by++) {
              Block block = world.getBlockAt(bx, by, bz);
              if (block.getType().isAir()) continue;
              byte[] bits =
                  block
                      .getChunk()
                      .getPersistentDataContainer()
                      .get(
                          NamespacedKey.fromString("lkjmc:built-v1"),
                          PersistentDataType.BYTE_ARRAY);
              boolean built =
                  bits != null
                      && BitSet.valueOf(bits)
                          .get((by - world.getMinHeight()) * 256 + (bz & 15) * 16 + (bx & 15));
              JsonObject out =
                  CoreClient.object(
                      "x",
                      bx - x,
                      "y",
                      by - y,
                      "z",
                      bz - z,
                      "material",
                      block.getType().name(),
                      "data",
                      block.getBlockData().getAsString(),
                      "built",
                      built);
              if (block.getState() instanceof InventoryHolder holder)
                out.add("items", items(holder.getInventory().getContents()));
              blocks.add(out);
            }
          }
        for (Entity entity : seen)
          if (!(entity instanceof Player)
              && entity.getX() >= x
              && entity.getX() < x + w
              && entity.getY() >= y
              && entity.getY() < y + h
              && entity.getZ() >= z
              && entity.getZ() < z + l) {
            JsonObject out =
                CoreClient.object(
                    "uuid",
                    entity.getUniqueId(),
                    "type",
                    entity.getType().name(),
                    "marker",
                    entity
                        .getPersistentDataContainer()
                        .get(
                            NamespacedKey.fromString("lkjmc:building-entity-slot"),
                            PersistentDataType.STRING));
            if (entity instanceof Tameable pet && pet.getOwner() != null)
              out.addProperty("owner", pet.getOwner().getUniqueId().toString());
            if (entity instanceof Villager villager) {
              out.addProperty("level", villager.getVillagerLevel());
              out.addProperty("xp", villager.getVillagerExperience());
              JsonArray trades = new JsonArray();
              for (MerchantRecipe r : villager.getRecipes())
                trades.add(
                    CoreClient.object(
                        "result",
                        items(new ItemStack[] {r.getResult()}),
                        "ingredients",
                        items(r.getIngredients().toArray(ItemStack[]::new)),
                        "uses",
                        r.getUses(),
                        "max_uses",
                        r.getMaxUses()));
              out.add("trades", trades);
            }
            if (entity instanceof ArmorStand armor)
              out.add("equipment", items(armor.getEquipment().getArmorContents()));
            entities.add(out);
          }
        sender.sendMessage(
            "FIXTURE_STATE "
                + CoreClient.JSON.toJson(
                    CoreClient.object("blocks", blocks, "entities", entities)));
      }
    } catch (Exception e) {
      sender.sendMessage("FIXTURE_ERROR " + e);
    }
    return true;
  }

  private JsonArray items(ItemStack[] contents) {
    JsonArray result = new JsonArray();
    for (ItemStack item : contents)
      if (item != null && !item.isEmpty())
        result.add(
            CoreClient.object("material", item.getType().name(), "amount", item.getAmount()));
    return result;
  }
}
