package com.lkjmc.paper;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;

import java.util.List;
import org.junit.jupiter.api.Test;

final class ExchangeInventoryPlannerTest {
    @Test
    void countsOnlyMatchingMaterial() {
        var slots = List.of(slot("COBBLESTONE", 32), slot("DIRT", 64), slot("COBBLESTONE", 2));

        assertEquals(34, ExchangeInventoryPlanner.count(slots, "COBBLESTONE"));
    }

    @Test
    void removesExactAmountAcrossStacksWithoutMutatingOriginal() {
        var slots = List.of(slot("COBBLESTONE", 32), slot("COBBLESTONE", 40));

        var updated = ExchangeInventoryPlanner.remove(slots, "COBBLESTONE", 64);

        assertEquals(32, slots.get(0).amount());
        assertEquals(List.of(slot("COBBLESTONE", 8)), updated);
    }

    @Test
    void refusesRemovalWhenInventoryIsInsufficient() {
        var slots = List.of(slot("COBBLESTONE", 1));

        assertThrows(IllegalArgumentException.class,
            () -> ExchangeInventoryPlanner.remove(slots, "COBBLESTONE", 2));
    }

    private static ExchangeInventoryPlanner.Slot slot(String material, int amount) {
        return new ExchangeInventoryPlanner.Slot(material, amount);
    }
}
