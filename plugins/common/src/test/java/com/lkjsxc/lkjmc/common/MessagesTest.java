package com.lkjsxc.lkjmc.common;

import com.google.gson.JsonPrimitive;
import java.util.Map;

/** No server or plugin boot is needed to verify the shared message contract. */
public final class MessagesTest {
  private static void equal(String expected, String actual) {
    if (!expected.equals(actual))
      throw new AssertionError("Expected " + expected + "; got " + actual);
  }

  public static void main(String[] args) {
    equal("Please sign in.", Messages.text("en", "error.login_required"));
    equal("ログインしてください。", Messages.text("ja", "error.login_required"));
    equal("Please sign in.", Messages.text("unsupported", "error.login_required"));
    String player = "日本語 {reference} $1 \\";
    equal(
        "The action failed. Reference: " + player,
        Messages.render(
            "en",
            new SystemMessage("error.internal", Map.of("reference", new JsonPrimitive(player)))));
    if (Messages.error("en", "秘密の内部診断").contains("秘密"))
      throw new AssertionError("Diagnostic leaked");
    if (!Messages.text("ja", "unknown.id").startsWith("操作を確認できませんでした。"))
      throw new AssertionError("Wrong fallback language");
    if (!Messages.text("en", "error.internal").contains("could not be verified"))
      throw new AssertionError("Missing parameter accepted");
    if (!Messages.text("en", "error.login_required", "extra").contains("could not be verified"))
      throw new AssertionError("Extra parameter accepted");
    equal(
        "Please sign in.",
        Messages.render(
            "en", SystemMessage.parse(SystemMessage.of("error.login_required").json())));
    System.out.println("Shared Java locale contract passed");
  }
}
