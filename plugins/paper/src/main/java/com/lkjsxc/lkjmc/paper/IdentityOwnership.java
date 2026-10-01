package com.lkjsxc.lkjmc.paper;

import com.destroystokyo.paper.event.entity.EntityAddToWorldEvent;
import com.google.gson.*;
import com.lkjsxc.lkjmc.common.*;
import java.util.*;
import org.bukkit.*;
import org.bukkit.entity.*;
import org.bukkit.event.*;
import org.bukkit.event.entity.*;
import org.bukkit.event.world.EntitiesLoadEvent;
import org.bukkit.persistence.PersistentDataType;

/**
 * Ordered, durable owner substitutions. The epoch travels with the native entity NBT, so an
 * unloaded pet is transformed exactly once before it resumes activity. New pets and deliberate
 * sales are stamped with the current epoch, including when the Java UUID was previously the
 * discarded profile's owner.
 */
public final class IdentityOwnership implements Listener {
  private static final UUID POLICY = UUID.fromString("b94587e9-0ae0-497c-b2e0-b948463c85ec");
  private static final NamespacedKey EPOCH = new NamespacedKey("lkjmc", "identity-owner-epoch");
  private static final NamespacedKey PLACED_BY = new NamespacedKey("lkjmc", "placed-by-account");
  private final PaperContext ctx;
  private final Journal journal;
  private JsonArray steps;

  public IdentityOwnership(PaperContext ctx) throws Exception {
    this.ctx = ctx;
    journal = new Journal(ctx.plugin().getDataFolder().toPath().resolve("identity-ownership"));
    steps = journal.read(POLICY).map(v -> v.getAsJsonArray("steps")).orElseGet(JsonArray::new);
  }

  public int install(UUID job, JsonObject plan) throws Exception {
    if (!Bukkit.isPrimaryThread())
      throw new IllegalStateException("Owner migration requires the server thread");
    for (int i = 0; i < steps.size(); i++) {
      JsonObject step = steps.get(i).getAsJsonObject();
      if (step.get("job_id").getAsString().equals(job.toString())) {
        if (!Journal.digest(step.get("plan")).equals(Journal.digest(plan)))
          throw new IllegalStateException("Owner migration plan changed");
        reconcileLoaded();
        return i + 1;
      }
    }
    JsonArray next = steps.deepCopy();
    next.add(CoreClient.object("job_id", job, "plan", plan));
    journal.write(POLICY, CoreClient.object("phase", "committed", "steps", next));
    steps = next;
    Faults.hit(ctx, "identity.policy");
    reconcileLoaded();
    return steps.size();
  }

  public void reconcileLoaded() throws Exception {
    Set<Chunk> changed = new HashSet<>();
    for (World world : Bukkit.getWorlds())
      for (Entity entity : world.getEntities())
        if (resolve(entity)) changed.add(entity.getLocation().getChunk());
    WorldDurability.flush(changed, List.of());
  }

  private boolean resolve(Entity entity) {
    Tameable pet = entity instanceof Tameable p ? p : null;
    String placed = entity.getPersistentDataContainer().get(PLACED_BY, PersistentDataType.STRING);
    if (pet == null && placed == null) return false;
    int epoch =
        entity.getPersistentDataContainer().getOrDefault(EPOCH, PersistentDataType.INTEGER, 0);
    if (epoch < 0 || epoch > steps.size())
      throw new IllegalStateException("Pet owner history is inconsistent with native data");
    UUID original = pet == null ? null : pet.getOwnerUniqueId(), owner = original;
    String placedAfter = placed;
    for (int i = epoch; i < steps.size(); i++) {
      JsonObject plan = steps.get(i).getAsJsonObject().getAsJsonObject("plan");
      if (owner != null) {
        if (owner.equals(nullable(plan, "discarded")))
          owner = CoreClient.uuid(plan, "archive_owner");
        else if (owner.equals(nullable(plan, "selected"))) owner = nullable(plan, "canonical");
      }
      if (placedAfter != null
          && plan.has("other_account")
          && placedAfter.equals(plan.get("other_account").getAsString()))
        placedAfter = plan.get("retained_account").getAsString();
    }
    if (!Objects.equals(original, owner))
      pet.setOwner(owner == null ? null : Bukkit.getOfflinePlayer(owner));
    if (!Objects.equals(placed, placedAfter))
      entity.getPersistentDataContainer().set(PLACED_BY, PersistentDataType.STRING, placedAfter);
    entity.getPersistentDataContainer().set(EPOCH, PersistentDataType.INTEGER, steps.size());
    return epoch != steps.size()
        || !Objects.equals(original, owner)
        || !Objects.equals(placed, placedAfter);
  }

  public void assign(Tameable pet, UUID owner) {
    pet.setOwner(Bukkit.getOfflinePlayer(owner));
    stamp(pet);
  }

  private void stamp(Entity entity) {
    if (entity instanceof Tameable
        || entity.getPersistentDataContainer().has(PLACED_BY, PersistentDataType.STRING))
      entity.getPersistentDataContainer().set(EPOCH, PersistentDataType.INTEGER, steps.size());
  }

  private void observe(Entity entity) {
    try {
      resolve(entity);
    } catch (Exception error) {
      if (entity instanceof Mob mob) mob.setAI(false);
      entity.setInvulnerable(true);
      ctx.plugin()
          .getLogger()
          .log(
              java.util.logging.Level.SEVERE,
              "Pet owner reconciliation failed; stopping to preserve data",
              error);
      Bukkit.shutdown();
    }
  }

  @EventHandler(priority = EventPriority.LOWEST)
  public void loaded(EntitiesLoadEvent event) {
    event.getEntities().forEach(this::observe);
  }

  @EventHandler(priority = EventPriority.LOWEST)
  public void added(EntityAddToWorldEvent event) {
    observe(event.getEntity());
  }

  @EventHandler(priority = EventPriority.MONITOR, ignoreCancelled = true)
  public void spawned(EntitySpawnEvent event) {
    if (event.getEntity().getPersistentDataContainer().has(EPOCH, PersistentDataType.INTEGER))
      observe(event.getEntity());
    else stamp(event.getEntity());
  }

  @EventHandler(priority = EventPriority.MONITOR, ignoreCancelled = true)
  public void tamed(EntityTameEvent event) {
    stamp(event.getEntity());
  }

  @EventHandler(priority = EventPriority.MONITOR, ignoreCancelled = true)
  public void bred(EntityBreedEvent event) {
    stamp(event.getEntity());
  }

  @EventHandler(priority = EventPriority.MONITOR, ignoreCancelled = true)
  public void placed(EntityPlaceEvent event) {
    stamp(event.getEntity());
  }

  static UUID nullable(JsonObject value, String key) {
    return value.has(key) && !value.get(key).isJsonNull() ? CoreClient.uuid(value, key) : null;
  }
}
