package com.lkjsxc.lkjmc.paper;

import com.google.gson.*;
import com.lkjsxc.lkjmc.common.*;
import java.nio.file.*;
import java.util.*;
import org.bukkit.*;
import org.bukkit.entity.Player;

/** Crash recovery rolls forward under the same quarantine before players can see a world. */
public final class BuildingTransactions {
  private final PaperContext ctx;
  private final BuildingStore store;
  private final WorldLocks locks;
  private final Journal journal;
  private final SpawnPolicy spawns;

  public BuildingTransactions(
      PaperContext ctx,
      ClaimProtection claims,
      Provenance provenance,
      WorldLocks locks,
      SpawnPolicy spawns)
      throws Exception {
    this.ctx = ctx;
    this.locks = locks;
    store = new BuildingStore(ctx, claims, provenance);
    this.spawns = spawns;
    journal = new Journal(ctx.plugin().getDataFolder().toPath().resolve("building-journal"));
  }

  public static final class Waiting extends Exception {
    public Waiting(SystemMessage message) {
      super(message.toString());
    }
  }

  public static boolean handles(JsonObject job) {
    String kind = job.get("kind").getAsString();
    return kind.equals("asset.capture")
            && !job.getAsJsonObject("payload").get("kind").getAsString().equals("items")
        || Set.of("asset.place", "asset.preview").contains(kind);
  }

  public void recover() throws Exception {
    // Committed records also retain their lock until the receipt has reached Core.
    Path directory = ctx.plugin().getDataFolder().toPath().resolve("building-journal");
    try (var files = Files.list(directory)) {
      for (Path path : files.filter(p -> p.toString().endsWith(".json")).toList()) {
        UUID id = UUID.fromString(path.getFileName().toString().replace(".json", ""));
        JsonObject row = journal.read(id).orElseThrow();
        String phase = row.get("phase").getAsString();
        if (Set.of("acknowledged", "rolled_back").contains(phase)) continue;
        locks.hold(id, WorldLocks.Box.read(row.getAsJsonObject("box")));
        if (phase.equals("prepared")) apply(row);
      }
    }
  }

  public boolean pending(UUID id) throws Exception {
    return journal
        .read(id)
        .filter(r -> !Set.of("rolled_back", "acknowledged").contains(r.get("phase").getAsString()))
        .isPresent();
  }

  public Optional<JsonObject> receipt(UUID id) throws Exception {
    return journal
        .read(id)
        .filter(r -> Set.of("committed", "acknowledged").contains(r.get("phase").getAsString()))
        .map(r -> r.getAsJsonObject("result"));
  }

  public void acknowledged(UUID id) throws Exception {
    Optional<JsonObject> row = journal.read(id);
    if (row.isEmpty()) return;
    JsonObject record = row.get();
    record.addProperty("phase", "acknowledged");
    journal.write(id, record);
    ctx.main(
        () -> {
          locks.release(id);
          return null;
        });
  }

  public void transferPets(JsonObject payload) throws Exception {
    if (payload.has("asset_id")) {
      UUID asset = CoreClient.uuid(payload, "asset_id");
      spawns.invalidateRespawns(WorldLocks.Box.read(store.read(asset).getAsJsonObject("source")));
      store.transferPets(asset, CoreClient.uuid(payload, "buyer_account"));
    }
  }

  public JsonObject execute(JsonObject job, Player actor) throws Exception {
    UUID id = CoreClient.uuid(job, "id");
    JsonObject payload = job.getAsJsonObject("payload");
    UUID asset = CoreClient.uuid(payload, "asset_id");
    Optional<JsonObject> complete = receipt(id);
    if (complete.isPresent()) return complete.get();
    Optional<JsonObject> prepared =
        journal.read(id).filter(r -> r.get("phase").getAsString().equals("prepared"));
    if (prepared.isPresent()) {
      ctx.main(
          () -> {
            apply(prepared.get());
            return null;
          });
      return prepared.get().getAsJsonObject("result");
    }
    if (job.get("kind").getAsString().equals("asset.capture")) return capture(job, actor);
    JsonObject record = store.read(asset);
    if (!Journal.digest(record.get("manifest"))
        .equals(payload.get("manifest_sha256").getAsString()))
      throw new IllegalStateException("Stored building does not match Core escrow");
    JsonObject placement = payload.getAsJsonObject("placement");
    UUID account = CoreClient.uuid(job, "actor");
    var target = ctx.main(() -> store.placement(placement, record, account));
    JsonObject preview =
        CoreClient.object(
            "asset_id",
            asset,
            "manifest_sha256",
            payload.get("manifest_sha256"),
            "footprint",
            target.box().json(),
            "origin",
            List.of(target.origin().x(), target.origin().y(), target.origin().z()),
            "rotation",
            target.degrees(),
            "claim_id",
            placement.get("claim_id"));
    String hash = Journal.digest(preview);
    boolean clear = ctx.main(() -> store.clear(target));
    preview.addProperty("preview_hash", hash);
    preview.addProperty("clear", clear);
    preview.add(
        "message",
        SystemMessage.of(
                clear
                    ? "text.no_blocks_or_entities_obstruct_the_area_check_the_origi_fea4c451e2"
                    : "text.clear_the_placement_area_including_empty_spaces_within_ea2fbf1c18")
            .json());
    if (job.get("kind").getAsString().equals("asset.preview")) return preview;
    if (!hash.equals(CoreClient.string(placement, "preview_hash", "")))
      throw new IllegalArgumentException(
          com.lkjsxc.lkjmc.common.SystemMessage.of("text.the_placement_position_or_rotation_changed_after_the_pr_a57df6d512").toString());
    JsonObject row = journal.read(id).orElse(null);
    if (row == null) {
      if (!clear)
        throw new IllegalArgumentException(
            com.lkjsxc.lkjmc.common.SystemMessage.of("text.blocks_or_entities_obstruct_the_placement_area_clear_it_9de61e69a4").toString());
      // Core already holds the destination claim. This lock survives until settlement is confirmed.
      row =
          CoreClient.object(
              "id",
              id,
              "asset_id",
              asset,
              "phase",
              "prepared",
              "operation",
              "place",
              "actor",
              account,
              "box",
              target.box().json(),
              "placement",
              placement,
              "origin",
              List.of(target.origin().x(), target.origin().y(), target.origin().z()),
              "rotation",
              target.degrees(),
              "result",
              CoreClient.object(
                  "effect", "committed", "asset_id", asset, "footprint", target.box().json()));
      JsonObject initial = row;
      ctx.main(
          () -> {
            locks.hold(id, target.box());
            if (!store.clear(target)) {
              locks.release(id);
              throw new IllegalArgumentException(com.lkjsxc.lkjmc.common.SystemMessage.of("text.the_placement_area_has_changed").toString());
            }
            journal.write(id, initial);
            fault("building.prepared");
            return null;
          });
    }
    JsonObject current = row;
    ctx.main(
        () -> {
          apply(current);
          return null;
        });
    return current.getAsJsonObject("result");
  }

  private JsonObject capture(JsonObject job, Player actor) throws Exception {
    UUID id = CoreClient.uuid(job, "id"),
        asset = CoreClient.uuid(job.getAsJsonObject("payload"), "asset_id");
    JsonObject row = journal.read(id).orElse(null);
    if (row == null) {
      Optional<JsonObject> existing = store.existing(asset);
      var box =
          existing.isPresent()
              ? WorldLocks.Box.read(existing.get().getAsJsonObject("source"))
              : ctx.main(() -> store.source(job, actor));
      JsonObject snapshot =
          ctx.main(
              () -> {
                store.noPlayers(box);
                locks.hold(id, box);
                try {
                  return store.capture(job, box);
                } catch (Exception e) {
                  locks.release(id);
                  throw e;
                }
              });
      row =
          CoreClient.object(
              "id",
              id,
              "asset_id",
              asset,
              "phase",
              "awaiting_consent",
              "operation",
              job.getAsJsonObject("payload").get("kind").getAsString().equals("land")
                  ? "land"
                  : "capture",
              "actor",
              job.get("actor"),
              "box",
              box.json(),
              "manifest",
              snapshot.get("manifest"),
              "result",
              CoreClient.object(
                  "effect",
                  "committed",
                  "original_removed",
                  !job.getAsJsonObject("payload").get("kind").getAsString().equals("land"),
                  "manifest",
                  snapshot.get("manifest")));
      journal.write(id, row);
    }
    String phase = row.get("phase").getAsString();
    if (phase.equals("rolled_back"))
      throw new IllegalArgumentException(com.lkjsxc.lkjmc.common.SystemMessage.of("text.packing_was_cancelled_the_original_is_unchanged").toString());
    if (phase.equals("awaiting_consent")) {
      JsonObject manifest = row.getAsJsonObject("manifest");
      String hash = Journal.digest(manifest);
      ctx.core()
          .ack(
              job,
              "leased",
              null,
              CoreClient.object("phase", "awaiting_consent", "manifest", manifest),
              null);
      ctx.refreshProjection();
      JsonObject assetState = null;
      for (JsonElement e : ctx.projection().getAsJsonArray("assets"))
        if (e.getAsJsonObject().get("id").getAsString().equals(asset.toString()))
          assetState = e.getAsJsonObject();
      if (assetState == null) throw new IllegalStateException("Capture reservation disappeared");
      if (assetState.get("cancel_requested").getAsBoolean()) {
        row.addProperty("phase", "rolled_back");
        journal.write(id, row);
        ctx.main(
            () -> {
              locks.release(id);
              return null;
            });
        throw new IllegalArgumentException(com.lkjsxc.lkjmc.common.SystemMessage.of("text.packing_cancelled_the_original_was_not_changed").toString());
      }
      for (JsonElement owner : manifest.getAsJsonArray("required_consents")) {
        boolean found = false;
        for (JsonElement e : ctx.projection().getAsJsonArray("consents")) {
          JsonObject c = e.getAsJsonObject();
          if (c.get("asset_id").getAsString().equals(asset.toString())
              && c.get("owner").equals(owner)
              && c.get("manifest_sha256").getAsString().equals(hash)) found = true;
        }
        if (!found)
          throw new Waiting(
              SystemMessage.of("text.waiting_for_pet_owner_consent_the_original_remains_protected"));
      }
      // Serialized with withdrawal by Core. The acknowledgement is a mutation authorization.
      ctx.core()
          .ack(
              job,
              "leased",
              null,
              CoreClient.object("phase", "removing", "manifest", manifest),
              null);
      row.addProperty("phase", "prepared");
      journal.write(id, row);
      fault("building.prepared");
    }
    JsonObject prepared = row;
    ctx.main(
        () -> {
          apply(prepared);
          return null;
        });
    return prepared.getAsJsonObject("result");
  }

  private void apply(JsonObject row) throws Exception {
    UUID id = CoreClient.uuid(row, "id"), asset = CoreClient.uuid(row, "asset_id");
    JsonObject record = store.read(asset);
    WorldLocks.Box box = WorldLocks.Box.read(row.getAsJsonObject("box"));
    locks.hold(id, box);
    String operation = row.get("operation").getAsString();
    if (operation.equals("capture")) {
      spawns.invalidateRespawns(box);
      store.remove(record, store.clipboard(asset));
    } else if (operation.equals("place")) {
      JsonArray origin = row.getAsJsonArray("origin");
      int rotation = row.get("rotation").getAsInt();
      var target =
          new BuildingStore.Placement(
              box,
              com.sk89q.worldedit.math.BlockVector3.at(
                  origin.get(0).getAsInt(), origin.get(1).getAsInt(), origin.get(2).getAsInt()),
              new com.sk89q.worldedit.math.transform.AffineTransform().rotateY(-rotation),
              rotation);
      store.place(record, store.clipboard(asset), target, asset, CoreClient.uuid(row, "actor"));
    }
    WorldDurability.flush(box.chunks(), List.of());
    fault("building.flushed");
    row.addProperty("phase", "committed");
    journal.write(id, row);
    fault("building.committed");
  }

  private void fault(String boundary) throws Exception {
    Faults.hit(ctx, boundary);
  }
}
