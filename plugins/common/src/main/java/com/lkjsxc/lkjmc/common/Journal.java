package com.lkjsxc.lkjmc.common;

import com.google.gson.*;
import java.io.IOException;
import java.nio.ByteBuffer;
import java.nio.channels.FileChannel;
import java.nio.file.*;
import java.security.MessageDigest;
import java.util.*;
import java.util.stream.Stream;

/** Durable write-ahead manifests and receipts; world mutations are replayed by job UUID. */
public final class Journal {
  private final Path root;

  public Journal(Path root) throws IOException {
    this.root = root;
    Files.createDirectories(root);
  }

  public synchronized Optional<JsonObject> read(UUID job) throws IOException {
    Path file = path(job);
    return Files.exists(file)
        ? Optional.of(JsonParser.parseString(Files.readString(file)).getAsJsonObject())
        : Optional.empty();
  }

  public synchronized void write(UUID job, JsonObject manifest) throws IOException {
    atomic(
        path(job),
        CoreClient.JSON
            .toJson(canonical(manifest))
            .getBytes(java.nio.charset.StandardCharsets.UTF_8));
  }

  public synchronized void remove(UUID id) throws IOException {
    Files.deleteIfExists(path(id));
    try (FileChannel directory = FileChannel.open(root, StandardOpenOption.READ)) {
      directory.force(true);
    }
  }

  public synchronized List<JsonObject> unfinished() throws IOException {
    List<JsonObject> result = new ArrayList<>();
    try (Stream<Path> paths = Files.list(root)) {
      for (Path path :
          paths.filter(p -> p.getFileName().toString().endsWith(".json")).sorted().toList()) {
        JsonObject item = JsonParser.parseString(Files.readString(path)).getAsJsonObject();
        if (!Set.of("committed", "rolled_back").contains(CoreClient.string(item, "phase", "")))
          result.add(item);
      }
    }
    return result;
  }

  private Path path(UUID id) {
    return root.resolve(id + ".json");
  }

  public static void atomic(Path target, byte[] bytes) throws IOException {
    Files.createDirectories(target.getParent());
    Path temporary = target.resolveSibling(target.getFileName() + ".tmp-" + UUID.randomUUID());
    try (FileChannel file =
        FileChannel.open(temporary, StandardOpenOption.CREATE_NEW, StandardOpenOption.WRITE)) {
      ByteBuffer buffer = ByteBuffer.wrap(bytes);
      while (buffer.hasRemaining()) file.write(buffer);
      file.force(true);
    }
    Files.move(
        temporary, target, StandardCopyOption.ATOMIC_MOVE, StandardCopyOption.REPLACE_EXISTING);
    try (FileChannel directory = FileChannel.open(target.getParent(), StandardOpenOption.READ)) {
      directory.force(true);
    }
  }

  public static JsonElement canonical(JsonElement value) {
    if (value.isJsonObject()) {
      JsonObject sorted = new JsonObject();
      value.getAsJsonObject().keySet().stream()
          .sorted()
          .forEach(k -> sorted.add(k, canonical(value.getAsJsonObject().get(k))));
      return sorted;
    }
    if (value.isJsonArray()) {
      JsonArray out = new JsonArray();
      value.getAsJsonArray().forEach(v -> out.add(canonical(v)));
      return out;
    }
    return value.deepCopy();
  }

  public static String digest(JsonElement manifest) {
    try {
      return HexFormat.of()
          .formatHex(
              MessageDigest.getInstance("SHA-256")
                  .digest(
                      CoreClient.JSON
                          .toJson(canonical(manifest))
                          .getBytes(java.nio.charset.StandardCharsets.UTF_8)));
    } catch (java.security.NoSuchAlgorithmException e) {
      throw new IllegalStateException(e);
    }
  }
}
