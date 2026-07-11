package com.lkjmc.paper;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import java.nio.charset.StandardCharsets;
import java.security.MessageDigest;
import java.util.Base64;
import org.bukkit.entity.Player;
import org.bukkit.inventory.ItemStack;

public final class PlayerProfileAdapter {
    private static final int MAX_PAYLOAD_BYTES = 1_000_000;
    private static final int MAX_ITEM_BYTES = 65_536;

    public ProfileSnapshot capture(Player player) {
        var root = new JsonObject();
        root.addProperty("version", 1);
        root.add("inventory", items(player.getInventory().getContents()));
        root.add("armor", items(player.getInventory().getArmorContents()));
        root.add("offHand", item(player.getInventory().getItemInOffHand()));
        root.add("enderChest", items(player.getEnderChest().getContents()));
        root.addProperty("level", player.getLevel());
        root.addProperty("exp", player.getExp());
        var bytes = root.toString().getBytes(StandardCharsets.UTF_8);
        return new ProfileSnapshot(Base64.getEncoder().encodeToString(bytes), sha256(bytes));
    }

    public void apply(Player player, String payloadBase64) {
        try {
            var bytes = Base64.getDecoder().decode(payloadBase64 == null ? "" : payloadBase64);
            if (bytes.length == 0 || bytes.length > MAX_PAYLOAD_BYTES) throw new IllegalArgumentException("payload bounds");
            var root = JsonParser.parseString(new String(bytes, StandardCharsets.UTF_8)).getAsJsonObject();
            if (!root.has("version") || root.get("version").getAsInt() != 1) throw new IllegalArgumentException("version");
            player.getInventory().setContents(items(root, "inventory", player.getInventory().getSize()));
            player.getInventory().setArmorContents(items(root, "armor", 4));
            player.getInventory().setItemInOffHand(item(root.get("offHand")));
            player.getEnderChest().setContents(items(root, "enderChest", player.getEnderChest().getSize()));
            player.setLevel(integer(root, "level", 0, Integer.MAX_VALUE));
            player.setExp(decimal(root, "exp", 0.0F, 1.0F));
        } catch (RuntimeException error) {
            throw new IllegalStateException("apply profile snapshot", error);
        }
    }

    private static JsonArray items(ItemStack[] values) {
        var encoded = new JsonArray();
        for (var value : values) encoded.add(item(value));
        return encoded;
    }

    private static JsonElement item(ItemStack value) {
        if (value == null || value.getType().isAir()) return com.google.gson.JsonNull.INSTANCE;
        var bytes = value.serializeAsBytes();
        if (bytes.length > MAX_ITEM_BYTES) throw new IllegalStateException("item exceeds profile bounds");
        return new com.google.gson.JsonPrimitive(Base64.getEncoder().encodeToString(bytes));
    }

    private static ItemStack[] items(JsonObject root, String name, int size) {
        if (!root.has(name) || !root.get(name).isJsonArray() || root.getAsJsonArray(name).size() != size) {
            throw new IllegalArgumentException("invalid " + name);
        }
        var values = new ItemStack[size];
        for (var index = 0; index < size; index++) values[index] = item(root.getAsJsonArray(name).get(index));
        return values;
    }

    private static ItemStack item(JsonElement value) {
        if (value == null || value.isJsonNull()) return null;
        if (!value.isJsonPrimitive() || !value.getAsJsonPrimitive().isString()) throw new IllegalArgumentException("item");
        var bytes = Base64.getDecoder().decode(value.getAsString());
        if (bytes.length == 0 || bytes.length > MAX_ITEM_BYTES) throw new IllegalArgumentException("item bounds");
        return ItemStack.deserializeBytes(bytes);
    }

    private static int integer(JsonObject root, String name, int min, int max) {
        var value = root.get(name).getAsInt();
        if (value < min || value > max) throw new IllegalArgumentException(name);
        return value;
    }

    private static float decimal(JsonObject root, String name, float min, float max) {
        var value = root.get(name).getAsFloat();
        if (!Float.isFinite(value) || value < min || value > max) throw new IllegalArgumentException(name);
        return value;
    }

    private static String sha256(byte[] bytes) {
        try {
            var digest = MessageDigest.getInstance("SHA-256").digest(bytes);
            var value = new StringBuilder();
            for (byte item : digest) value.append(String.format("%02x", item));
            return value.toString();
        } catch (Exception error) {
            throw new IllegalStateException("hash profile snapshot", error);
        }
    }
}
