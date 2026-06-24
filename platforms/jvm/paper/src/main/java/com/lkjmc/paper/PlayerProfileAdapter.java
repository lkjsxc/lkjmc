package com.lkjmc.paper;

import java.io.ByteArrayOutputStream;
import java.security.MessageDigest;
import java.util.Base64;
import org.bukkit.entity.Player;
import org.bukkit.inventory.ItemStack;
import org.bukkit.util.io.BukkitObjectOutputStream;

public final class PlayerProfileAdapter {
    public ProfileSnapshot capture(Player player) {
        try {
            var output = new ByteArrayOutputStream();
            try (var object = new BukkitObjectOutputStream(output)) {
                object.writeObject(player.getInventory().getContents());
                object.writeObject(player.getInventory().getArmorContents());
                object.writeObject(player.getInventory().getItemInOffHand());
                object.writeObject(player.getEnderChest().getContents());
                object.writeInt(player.getLevel());
                object.writeFloat(player.getExp());
            }
            var bytes = output.toByteArray();
            return new ProfileSnapshot(Base64.getEncoder().encodeToString(bytes), sha256(bytes));
        } catch (Exception error) {
            throw new IllegalStateException("capture profile snapshot", error);
        }
    }

    private static String sha256(byte[] bytes) throws Exception {
        var digest = MessageDigest.getInstance("SHA-256").digest(bytes);
        var builder = new StringBuilder();
        for (byte value : digest) {
            builder.append(String.format("%02x", value));
        }
        return builder.toString();
    }
}
