package com.lkjsxc.lkjmc.paper;

import java.nio.channels.FileChannel;
import java.nio.file.*;

/** Explicit opt-in crash boundaries for private acceptance tests. Inert in production. */
final class Faults {
  private Faults() {}

  static void hit(PaperContext ctx, String boundary) throws Exception {
    if (!Boolean.getBoolean("lkjmc.testFaults")) return;
    Path file = ctx.plugin().getDataFolder().toPath().resolve("test-crash-once");
    if (Files.isRegularFile(file) && Files.readString(file).trim().equals(boundary)) {
      Files.delete(file);
      try (var directory = FileChannel.open(file.getParent(), StandardOpenOption.READ)) {
        directory.force(true);
      }
      Runtime.getRuntime().halt(86);
    }
  }
}
