package com.lkjmc.common.ui.document;

import java.util.List;
import java.util.Map;
import java.util.Optional;

public final class RegionCatalog {
    private static final Map<String, List<Integer>> REGIONS = Map.of(
        "interior-28", List.of(
            10, 11, 12, 13, 14, 15, 16,
            19, 20, 21, 22, 23, 24, 25,
            28, 29, 30, 31, 32, 33, 34,
            37, 38, 39, 40, 41, 42, 43),
        "interior-21", List.of(
            19, 20, 21, 22, 23, 24, 25,
            28, 29, 30, 31, 32, 33, 34,
            37, 38, 39, 40, 41, 42, 43),
        "filter-row", List.of(10, 11, 12, 13, 14, 15, 16),
        "detail-band", List.of(20, 21, 22, 23, 24),
        "confirm-pair", List.of(11, 15)
    );

    private RegionCatalog() {}

    public static Optional<List<Integer>> find(String name) {
        var slots = REGIONS.get(name);
        return slots == null ? Optional.empty() : Optional.of(slots);
    }

    public static List<Integer> require(String name) {
        return find(name).orElseThrow(() -> new IllegalArgumentException("unknown region: " + name));
    }

    public static boolean exists(String name) {
        return REGIONS.containsKey(name);
    }

    public static Map<String, List<Integer>> regions() {
        return REGIONS;
    }
}
