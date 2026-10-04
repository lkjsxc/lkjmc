package com.lkjsxc.lkjmc.paper;

import com.google.gson.*;
import com.lkjsxc.lkjmc.common.*;
import java.io.*;
import java.nio.channels.FileChannel;
import java.nio.file.*;
import java.security.MessageDigest;
import java.util.*;
import java.util.zip.*;
import org.bukkit.*;
import org.enginehub.linbus.stream.LinBinaryIO;
import org.enginehub.linbus.tree.*;

/** Offline profile replacement from immutable, hashed archives. Never combines datasets. */
public final class IdentityTransactions {
  public static final class Waiting extends Exception {
    public Waiting(SystemMessage message) {
      super(message.toString());
    }
  }

  private final PaperContext ctx;
  private final SpawnPolicy spawns;
  private final IdentityOwnership ownership;
  private final GameEvents events;
  private final Journal journal;
  private final Path root, players, locations;
  private final Map<UUID, Set<UUID>> unpublished = new java.util.concurrent.ConcurrentHashMap<>();
  private static final List<String> PARTS =
      List.of("data", "data_old", "advancements", "stats", "location");
  private static final int MAX_FILE = 64 * 1024 * 1024;

  public IdentityTransactions(
      PaperContext ctx, SpawnPolicy spawns, IdentityOwnership ownership, GameEvents events)
      throws Exception {
    this.ctx = ctx;
    this.spawns = spawns;
    this.ownership = ownership;
    this.events = events;
    root = ctx.plugin().getDataFolder().toPath().resolve("identity-archives");
    journal = new Journal(root.resolve("journal"));
    players = Bukkit.getServer().getLevelDirectory().resolve("players");
    locations = ctx.plugin().getDataFolder().toPath().resolve("player-locations");
    try (var files = Files.list(root.resolve("journal"))) {
      for (Path file : files.filter(p -> p.getFileName().toString().endsWith(".json")).toList()) {
        UUID id = UUID.fromString(file.getFileName().toString().replace(".json", ""));
        JsonObject row = journal.read(id).orElseThrow();
        if (!row.has("published") || !row.get("published").getAsBoolean())
          unpublished.put(id, nativeIds(row.getAsJsonObject("plan")));
      }
    }
  }

  public boolean blocked(UUID nativeId) {
    return unpublished.values().stream().anyMatch(ids -> ids.contains(nativeId));
  }

  public List<UUID> unpublishedJobs() {
    return List.copyOf(unpublished.keySet());
  }

  public void failedBeforePreparation(UUID id) throws Exception {
    if (journal.read(id).isPresent())
      throw new IllegalStateException("Prepared identity migration cannot be discarded");
    unpublished.remove(id);
  }

  public void acknowledged(UUID id, JsonObject result) throws Exception {
    JsonObject state = journal.read(id).orElseThrow();
    if (!state.get("phase").getAsString().equals("committed")
        || !Journal.digest(state.get("result")).equals(Journal.digest(result)))
      throw new IllegalStateException("Identity acknowledgement differs from its native receipt");
    state.addProperty("published", true);
    journal.write(id, state);
    unpublished.remove(id);
  }

  private static Set<UUID> nativeIds(JsonObject plan) {
    Set<UUID> ids = new HashSet<>();
    for (String key : List.of("canonical", "selected", "discarded")) {
      UUID nativeId = IdentityOwnership.nullable(plan, key);
      if (nativeId != null) ids.add(nativeId);
    }
    return ids;
  }

  public boolean pending(UUID id) throws Exception {
    return journal
        .read(id)
        .filter(v -> !v.get("phase").getAsString().equals("committed"))
        .isPresent();
  }

  public JsonObject execute(JsonObject job) throws Exception {
    UUID id = CoreClient.uuid(job, "id");
    JsonObject payload = job.getAsJsonObject("payload"),
        plan = payload.getAsJsonObject("native_plan");
    Set<UUID> nativeIds = nativeIds(plan);
    boolean connected =
        ctx.main(
            () -> {
              boolean found = false;
              for (UUID nativeId : nativeIds)
                if (Bukkit.getPlayer(nativeId) instanceof org.bukkit.entity.Player player) {
                  player.kick(
                      ctx.text(
                          player.getUniqueId(),
                          "text.linking_accounts_using_the_selected_game_data_reconnect_d4d6df623a"));
                  found = true;
                }
              return found;
            });
    if (connected)
      throw new Waiting(
          SystemMessage.of("text.waiting_for_both_accounts_to_disconnect_and_save"));
    events.drainFor(
        Set.of(
            CoreClient.uuid(payload, "retained_account"),
            CoreClient.uuid(payload, "other_account")));
    if (!ctx.core()
        .post(
            "/internal/v1/jobs/" + id + "/identity-ready",
            CoreClient.object("lease_token", job.get("lease_token")))
        .get("ready")
        .getAsBoolean())
      throw new Waiting(
          SystemMessage.of("text.waiting_for_all_game_connections_and_pvp_restrictions_to_end"));
    JsonObject state = journal.read(id).orElse(null);
    if (state != null && !state.get("payload_sha256").getAsString().equals(Journal.digest(payload)))
      throw new IllegalStateException("Identity migration payload changed");
    if (state != null && state.get("phase").getAsString().equals("committed"))
      return state.getAsJsonObject("result");
    unpublished.put(id, nativeIds);
    Path directory = root.resolve(id.toString());
    if (state == null) {
      JsonArray files = new JsonArray();
      for (UUID nativeId : nativeIds)
        for (String part : PARTS) {
          Path source = live(nativeId, part);
          String name = nativeId + "/" + part;
          byte[] bytes = part.equals("location") ? ctx.main(() -> read(source)) : read(source);
          if (bytes != null) Journal.atomic(directory.resolve("original").resolve(name), bytes);
          files.add(
              CoreClient.object(
                  "native_uuid",
                  nativeId,
                  "part",
                  part,
                  "path",
                  name,
                  "sha256",
                  bytes == null ? null : sha(bytes),
                  "bytes",
                  bytes == null ? 0 : bytes.length));
        }
      UUID selected = IdentityOwnership.nullable(plan, "selected"),
          canonical = IdentityOwnership.nullable(plan, "canonical");
      if (selected != null
          && original(files, selected, "data") == null
          && (original(files, selected, "data_old") != null
              || original(files, selected, "location") != null))
        throw new IllegalStateException(
            "Selected native player data is missing; recover its saved data before linking");
      JsonArray targets = new JsonArray();
      if (canonical != null)
        for (String part : PARTS) {
          // Never retain a discarded fallback .dat_old. The selected data also becomes its
          // fallback, so Minecraft's recovery path cannot resurrect the discarded inventory.
          String sourcePart = part.equals("data_old") ? "data" : part;
          JsonObject source = selected == null ? null : original(files, selected, sourcePart);
          byte[] bytes =
              source == null
                  ? null
                  : verified(
                      directory.resolve("original").resolve(source.get("path").getAsString()),
                      source.get("sha256").getAsString());
          if (bytes != null && (part.equals("data") || part.equals("data_old")))
            bytes = nativeUuid(bytes, canonical);
          if (bytes != null) Journal.atomic(directory.resolve("selected").resolve(part), bytes);
          targets.add(
              CoreClient.object(
                  "native_uuid",
                  canonical,
                  "part",
                  part,
                  "sha256",
                  bytes == null ? null : sha(bytes)));
        }
      state =
          CoreClient.object(
              "id",
              id,
              "phase",
              "prepared",
              "payload_sha256",
              Journal.digest(payload),
              "plan",
              plan,
              "original",
              files,
              "targets",
              targets);
      journal.write(id, state);
      Faults.hit(ctx, "identity.prepared");
    }
    // Verify every archived byte before changing anything, including the discarded side.
    for (JsonElement e : state.getAsJsonArray("original")) {
      JsonObject row = e.getAsJsonObject();
      if (CoreClient.string(row, "sha256", null) != null)
        verified(
            directory.resolve("original").resolve(row.get("path").getAsString()),
            row.get("sha256").getAsString());
    }
    JsonObject ownerPlan = plan.deepCopy();
    ownerPlan.add("other_account", payload.get("other_account"));
    ownerPlan.add("retained_account", payload.get("retained_account"));
    int epoch = ctx.main(() -> ownership.install(id, ownerPlan));
    UUID canonical = IdentityOwnership.nullable(plan, "canonical");
    for (JsonElement e : state.getAsJsonArray("targets")) {
      JsonObject row = e.getAsJsonObject();
      String part = row.get("part").getAsString();
      byte[] bytes =
          CoreClient.string(row, "sha256", null) == null
              ? null
              : verified(
                  directory.resolve("selected").resolve(part), row.get("sha256").getAsString());
      if (part.equals("location"))
        ctx.main(
            () -> {
              replace(live(canonical, part), bytes);
              spawns.reloadIdentity(canonical);
              return null;
            });
      else replace(live(canonical, part), bytes);
      if (part.equals("data")) Faults.hit(ctx, "identity.playerdata");
    }
    for (UUID nativeId : nativeIds)
      if (!nativeId.equals(canonical)) {
        for (String part : PARTS) {
          if (part.equals("location"))
            ctx.main(
                () -> {
                  replace(live(nativeId, part), null);
                  spawns.reloadIdentity(nativeId);
                  return null;
                });
          else replace(live(nativeId, part), null);
        }
      }
    // Check the installed bytes, not just the archive. Both missing and unexpected files fail.
    for (JsonElement e : state.getAsJsonArray("targets")) {
      JsonObject row = e.getAsJsonObject();
      byte[] actual = read(live(canonical, row.get("part").getAsString()));
      if (CoreClient.string(row, "sha256", null) == null
          ? actual != null
          : actual == null || !sha(actual).equals(row.get("sha256").getAsString()))
        throw new IllegalStateException("Installed player data failed verification");
    }
    JsonObject manifest =
        CoreClient.object(
            "id",
            id,
            "payload_sha256",
            state.get("payload_sha256"),
            "plan",
            plan,
            "original",
            state.get("original"),
            "targets",
            state.get("targets"),
            "pet_policy_epoch",
            epoch);
    Journal.atomic(
        directory.resolve("manifest.json"),
        CoreClient.JSON
            .toJson(Journal.canonical(manifest))
            .getBytes(java.nio.charset.StandardCharsets.UTF_8));
    JsonObject result =
        CoreClient.object(
            "effect",
            "committed",
            "native_data_verified",
            true,
            "native_uuid",
            canonical,
            "archive_owner",
            plan.get("archive_owner"),
            "manifest_sha256",
            Journal.digest(manifest),
            "pet_policy_durable",
            true,
            "pet_policy_epoch",
            epoch);
    state.addProperty("phase", "committed");
    state.add("result", result);
    journal.write(id, state);
    Faults.hit(ctx, "identity.committed");
    return result;
  }

  private Path live(UUID id, String part) {
    return switch (part) {
      case "data" -> players.resolve("data").resolve(id + ".dat");
      case "data_old" -> players.resolve("data").resolve(id + ".dat_old");
      case "location" -> locations.resolve(id + ".json");
      case "advancements", "stats" -> players.resolve(part).resolve(id + ".json");
      default -> throw new IllegalArgumentException("Unknown native data part");
    };
  }

  private static JsonObject original(JsonArray rows, UUID id, String part) {
    for (JsonElement e : rows) {
      JsonObject row = e.getAsJsonObject();
      if (row.get("native_uuid").getAsString().equals(id.toString())
          && row.get("part").getAsString().equals(part)
          && CoreClient.string(row, "sha256", null) != null) return row;
    }
    return null;
  }

  private static byte[] read(Path path) throws Exception {
    if (!Files.exists(path, LinkOption.NOFOLLOW_LINKS)) return null;
    if (!Files.isRegularFile(path, LinkOption.NOFOLLOW_LINKS) || Files.size(path) > MAX_FILE)
      throw new IOException("Unsafe or oversized native file: " + path.getFileName());
    return Files.readAllBytes(path);
  }

  private static byte[] verified(Path path, String digest) throws Exception {
    byte[] bytes = read(path);
    if (bytes == null || !sha(bytes).equals(digest))
      throw new IOException("Identity archive checksum mismatch");
    return bytes;
  }

  private static void replace(Path path, byte[] bytes) throws Exception {
    if (bytes != null) Journal.atomic(path, bytes);
    else if (Files.deleteIfExists(path))
      try (FileChannel parent = FileChannel.open(path.getParent(), StandardOpenOption.READ)) {
        parent.force(true);
      }
  }

  private static String sha(byte[] bytes) throws Exception {
    return HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(bytes));
  }

  private static byte[] nativeUuid(byte[] bytes, UUID target) throws Exception {
    byte[] uncompressed;
    try (var gzip = new GZIPInputStream(new ByteArrayInputStream(bytes))) {
      uncompressed = gzip.readNBytes(MAX_FILE + 1);
    }
    if (uncompressed.length > MAX_FILE)
      throw new IOException("Native player data expands beyond limit");
    LinRootEntry root =
        LinBinaryIO.readUsing(
            new DataInputStream(new ByteArrayInputStream(uncompressed)), LinRootEntry::readFrom);
    long high = target.getMostSignificantBits(), low = target.getLeastSignificantBits();
    LinCompoundTag value =
        root.value().toBuilder()
            .putIntArray(
                "UUID", new int[] {(int) (high >>> 32), (int) high, (int) (low >>> 32), (int) low})
            .build();
    ByteArrayOutputStream out = new ByteArrayOutputStream();
    try (var gzip = new GZIPOutputStream(out);
        var data = new DataOutputStream(gzip)) {
      LinBinaryIO.write(data, new LinRootEntry(root.name(), value));
    }
    return out.toByteArray();
  }
}
