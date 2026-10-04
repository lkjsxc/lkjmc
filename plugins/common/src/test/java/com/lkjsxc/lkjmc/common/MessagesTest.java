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
    String[][] durableMessages = {
      {
        "text.no_blocks_or_entities_obstruct_the_area_check_the_origi_fea4c451e2",
        "No blocks or entities obstruct the area. Check the origin, rotation, and bounds before confirming.",
        "設置範囲にブロックや生き物はいません。原点・回転・範囲を確認して確定してください。"
      },
      {
        "text.clear_the_placement_area_including_empty_spaces_within_ea2fbf1c18",
        "Clear the placement area, including empty spaces within the building. Existing blocks and entities are never overwritten.",
        "設置範囲を空にしてください。建物内の空間も含め、既存のブロックや生き物は上書きしません。"
      },
      {
        "text.waiting_for_pet_owner_consent_the_original_remains_protected",
        "Waiting for pet-owner consent. The original remains protected.",
        "ペットの飼い主の同意を待っています。原本は保護された状態で残っています。"
      },
      {
        "text.waiting_for_both_accounts_to_disconnect_and_save",
        "Waiting for both accounts to disconnect and save.",
        "連携する両アカウントの切断・保存を待っています。"
      },
      {
        "text.waiting_for_all_game_connections_and_pvp_restrictions_to_end",
        "Waiting for all game connections and PvP restrictions to end.",
        "全サーバーでの切断とPvP制限の終了を待っています。"
      }
    };
    for (String[] row : durableMessages) {
      // Preview JSON and durable exception/ack TEXT retain IDs across a locale change.
      SystemMessage durable = SystemMessage.decode(SystemMessage.of(row[0]).toString());
      equal(row[1], Messages.render("en", durable.json()));
      equal(row[2], Messages.render("ja", durable.json()));
      equal(row[1], Messages.render("en", durable.json()));
    }
    System.out.println("Shared Java locale contract passed");
  }
}
