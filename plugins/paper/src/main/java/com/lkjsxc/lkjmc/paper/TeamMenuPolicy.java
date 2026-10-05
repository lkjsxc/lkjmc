package com.lkjsxc.lkjmc.paper;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.lkjsxc.lkjmc.common.CoreClient;
import java.util.ArrayList;
import java.util.List;

/**
 * UI capabilities come from the explicit owner's effective permissions, never contribution choice.
 */
final class TeamMenuPolicy {
  record Permission(String field, String commandField, String titleId) {}

  static final List<Permission> PERMISSIONS =
      List.of(
          new Permission("can_build", "build", "text.build"),
          new Permission("can_sell", "sell", "text.sell"),
          new Permission("can_spend", "spend", "text.spend_shared_coins"),
          new Permission("can_manage_members", "members", "text.manage_members"),
          new Permission("can_administer", "administer", "text.manage_team"));

  private TeamMenuPolicy() {}

  static boolean allowed(JsonObject owner, String capability) {
    if (owner == null || !owner.has("permissions") || !owner.get("permissions").isJsonObject())
      return false;
    JsonObject permissions = owner.getAsJsonObject("permissions");
    return permissions.has(capability)
        && permissions.get(capability).isJsonPrimitive()
        && permissions.get(capability).getAsJsonPrimitive().isBoolean()
        && permissions.get(capability).getAsBoolean();
  }

  static List<JsonObject> eligibleOwners(JsonArray owners, String capability) {
    List<JsonObject> eligible = new ArrayList<>();
    for (JsonElement value : owners) {
      JsonObject owner = value.getAsJsonObject();
      if (allowed(owner, capability)) eligible.add(owner);
    }
    return eligible;
  }

  static boolean leader(JsonObject team, String account) {
    return account != null
        && !account.isBlank()
        && CoreClient.string(team, "leader", "").equals(account);
  }

  static boolean canBuy(JsonObject owner, JsonObject listing) {
    return allowed(owner, "can_spend")
        && !owner.get("id").equals(listing.get("seller"))
        && (!CoreClient.string(listing, "kind", "").equals("land") || allowed(owner, "can_build"));
  }

  static boolean canConsent(JsonObject asset, String account) {
    JsonObject manifest = asset.getAsJsonObject("manifest");
    if (manifest == null || !manifest.has("required_consents")) return false;
    for (JsonElement value : manifest.getAsJsonArray("required_consents"))
      if (value.getAsString().equals(account)) return true;
    return false;
  }

  static boolean canClaim(JsonObject owner) {
    return allowed(owner, "can_build")
        && owner.get("used_chunks").getAsLong()
            < owner.getAsJsonObject("land").get("chunks").getAsLong();
  }

  static boolean canEditMember(JsonObject team, JsonObject member, String account) {
    return allowed(team, "can_administer")
        && !CoreClient.string(member, "account_id", "")
            .equals(CoreClient.string(team, "leader", ""))
        // Core reserves granting or preserving administrator status to the leader.
        && (leader(team, account) || !member.get("can_administer").getAsBoolean());
  }

  static JsonObject effectiveMemberPermissions(JsonObject team, JsonObject member) {
    JsonObject flags = member.deepCopy();
    if (leader(team, CoreClient.string(member, "account_id", ""))
        || member.get("can_administer").getAsBoolean())
      for (Permission permission : PERMISSIONS) flags.addProperty(permission.field(), true);
    return flags;
  }

  static JsonObject permissionsCommand(JsonObject team, JsonObject member, JsonObject draft) {
    JsonObject command =
        CoreClient.object(
            "type", "team_permissions", "team", team.get("id"), "member", member.get("account_id"));
    for (Permission permission : PERMISSIONS)
      command.addProperty(permission.commandField(), draft.get(permission.field()).getAsBoolean());
    return command;
  }
}
