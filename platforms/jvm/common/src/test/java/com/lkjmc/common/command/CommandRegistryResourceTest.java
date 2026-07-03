package com.lkjmc.common.command;

import static org.junit.jupiter.api.Assertions.assertTrue;

import com.google.gson.JsonParser;
import java.io.InputStreamReader;
import java.nio.charset.StandardCharsets;
import java.util.HashMap;
import java.util.Map;
import java.util.Set;
import org.junit.jupiter.api.Test;

final class CommandRegistryResourceTest {
    private static final Set<String> LOCAL_TARGETS = Set.of(
        "config.check", "proxy.send", "temporary.send", "wake.send", "restart.warn"
    );

    @Test
    void daemonTargetsExistInCopiedRegistry() throws Exception {
        var contracts = contracts();
        for (var spec : LkjmcCommandTree.specs()) {
            if (LOCAL_TARGETS.contains(spec.target())) {
                continue;
            }
            assertTrue(contracts.containsKey(spec.target()), spec.target());
            var surfaces = contracts.get(spec.target());
            for (var platform : spec.platforms()) {
                assertTrue(surfaces.contains(surface(platform)), spec.target() + " " + platform);
            }
        }
    }

    private static Map<String, Set<String>> contracts() throws Exception {
        var stream = CommandRegistryResourceTest.class.getResourceAsStream("/commands.json");
        if (stream == null) {
            throw new IllegalStateException("commands.json resource missing");
        }
        var root = JsonParser.parseReader(new InputStreamReader(stream, StandardCharsets.UTF_8))
            .getAsJsonObject();
        var result = new HashMap<String, Set<String>>();
        for (var item : root.getAsJsonArray("commands")) {
            var object = item.getAsJsonObject();
            var name = object.get("name").getAsString();
            var values = new java.util.HashSet<String>();
            for (var surface : object.getAsJsonArray("surfaces")) {
                values.add(surface.getAsString());
            }
            result.put(name, values);
        }
        return result;
    }

    private static String surface(CommandPlatform platform) {
        return switch (platform) {
            case PAPER -> "paper";
            case VELOCITY -> "velocity";
        };
    }
}
