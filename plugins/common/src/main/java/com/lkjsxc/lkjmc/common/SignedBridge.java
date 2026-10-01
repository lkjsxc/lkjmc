package com.lkjsxc.lkjmc.common;

import com.google.gson.*;
import java.nio.charset.StandardCharsets;
import java.nio.file.*;
import java.security.MessageDigest;
import java.util.*;
import javax.crypto.Mac;
import javax.crypto.spec.SecretKeySpec;

/** A separate first-party key. Never distribute this key to tenant servers. */
public final class SignedBridge {
  public static final String CHANNEL = "lkjmc:departure";
  private final byte[] key;

  public SignedBridge(Path file) throws Exception {
    key = HexFormat.of().parseHex(Files.readString(file).trim());
    if (key.length != 32) throw new IllegalArgumentException("Departure key must be 32 bytes");
  }

  private byte[] signature(JsonObject value) throws Exception {
    Mac mac = Mac.getInstance("HmacSHA256");
    mac.init(new SecretKeySpec(key, "HmacSHA256"));
    return mac.doFinal(
        CoreClient.JSON.toJson(Journal.canonical(value)).getBytes(StandardCharsets.UTF_8));
  }

  public byte[] encode(JsonObject value) throws Exception {
    return CoreClient.JSON
        .toJson(
            CoreClient.object(
                "payload", value, "signature", HexFormat.of().formatHex(signature(value))))
        .getBytes(StandardCharsets.UTF_8);
  }

  public JsonObject decode(byte[] bytes) throws Exception {
    if (bytes.length > 8192) throw new IllegalArgumentException("Bridge frame too large");
    JsonObject frame =
        JsonParser.parseString(new String(bytes, StandardCharsets.UTF_8)).getAsJsonObject();
    JsonObject value = frame.getAsJsonObject("payload");
    if (!MessageDigest.isEqual(
        signature(value), HexFormat.of().parseHex(frame.get("signature").getAsString())))
      throw new IllegalArgumentException("Invalid bridge signature");
    long expires = value.get("expires_at").getAsLong(), now = System.currentTimeMillis();
    if (expires < now || expires > now + 20_000)
      throw new IllegalArgumentException("Expired bridge frame");
    return value;
  }
}
