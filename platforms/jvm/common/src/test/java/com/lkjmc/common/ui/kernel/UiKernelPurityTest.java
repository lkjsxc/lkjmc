package com.lkjmc.common.ui.kernel;

import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import org.junit.jupiter.api.Test;

final class UiKernelPurityTest {
    private static final List<String> FORBIDDEN = List.of(
        "org.bukkit", "io.papermc", "com.destroystokyo.paper", "com.velocitypowered",
        "java.net", "java.nio.file", "java.io.File", "java.sql", "java.lang.Process");

    @Test
    void productionDocumentAndKernelPackagesHaveNoAdapterOrIoImports() throws Exception {
        var source = sourceRoot();
        var violations = new java.util.ArrayList<String>();
        try (var files = Files.walk(source)) {
            for (var file : files.filter(path -> path.toString().endsWith(".java")).toList()) {
                for (var line : Files.readAllLines(file)) {
                    if (line.startsWith("import ") && FORBIDDEN.stream().anyMatch(line::contains)) {
                        violations.add(source.relativize(file) + ": " + line);
                    }
                }
            }
        }
        assertTrue(violations.isEmpty(), violations::toString);
    }

    private static Path sourceRoot() {
        var cwd = Path.of(System.getProperty("user.dir")).toAbsolutePath();
        for (var path = cwd; path != null; path = path.getParent()) {
            var candidate = path.resolve("platforms/jvm/common/src/main/java/com/lkjmc/common/ui");
            if (Files.isDirectory(candidate)) {
                return candidate;
            }
            var module = path.resolve("src/main/java/com/lkjmc/common/ui");
            if (Files.isDirectory(module)) {
                return module;
            }
        }
        throw new IllegalStateException("common ui source root not found from " + cwd);
    }
}
