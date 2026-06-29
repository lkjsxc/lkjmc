package com.lkjmc.paper;

import java.util.ArrayList;
import java.util.List;
import org.bukkit.Material;
import org.bukkit.inventory.ItemStack;

final class ExchangeInventoryPlanner {
    private ExchangeInventoryPlanner() {}

    static long count(List<Slot> slots, String material) {
        return slots.stream().filter(slot -> slot.material.equals(material)).mapToLong(slot -> slot.amount).sum();
    }

    static List<Slot> remove(List<Slot> slots, String material, long amount) {
        var remaining = amount;
        var result = new ArrayList<Slot>();
        for (var slot : slots) {
            if (!slot.material.equals(material) || remaining <= 0) {
                result.add(slot);
                continue;
            }
            var take = Math.min(slot.amount, remaining);
            if (slot.amount > take) {
                result.add(new Slot(slot.material, (int) (slot.amount - take)));
            }
            remaining -= take;
        }
        if (remaining != 0) {
            throw new IllegalArgumentException("not enough inventory items");
        }
        return List.copyOf(result);
    }

    static long count(ItemStack[] contents, Material material) {
        var total = 0L;
        for (var item : contents) {
            if (item != null && item.getType() == material) {
                total += item.getAmount();
            }
        }
        return total;
    }

    static ItemStack[] remove(ItemStack[] contents, Material material, long amount) {
        var copy = contents.clone();
        var remaining = amount;
        for (var index = 0; index < copy.length && remaining > 0; index++) {
            var item = copy[index];
            if (item == null || item.getType() != material) {
                continue;
            }
            var edited = item.clone();
            var take = Math.min(edited.getAmount(), remaining);
            edited.setAmount((int) (edited.getAmount() - take));
            copy[index] = edited.getAmount() <= 0 ? null : edited;
            remaining -= take;
        }
        if (remaining != 0) {
            throw new IllegalArgumentException("not enough inventory items");
        }
        return copy;
    }

    record Slot(String material, int amount) {}
}
