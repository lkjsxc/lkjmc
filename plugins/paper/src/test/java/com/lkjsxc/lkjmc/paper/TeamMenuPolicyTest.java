package com.lkjsxc.lkjmc.paper;

import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import com.lkjsxc.lkjmc.common.CoreClient;
import java.util.List;

/** Capability and command checks need neither a Paper process nor a mutable fixture database. */
public final class TeamMenuPolicyTest {
  private static void check(boolean valid, String reason) {
    if (!valid) throw new AssertionError(reason);
  }

  private static JsonObject permissions(boolean build, boolean admin) {
    return CoreClient.object(
        "can_build",
        build,
        "can_sell",
        false,
        "can_spend",
        false,
        "can_manage_members",
        false,
        "can_administer",
        admin);
  }

  private static JsonObject owner(String id, boolean build, boolean contribution, long capacity) {
    return CoreClient.object(
        "id",
        id,
        "leader",
        "leader",
        "permissions",
        permissions(build, false),
        "is_contribution_team",
        contribution,
        "used_chunks",
        0,
        "land",
        CoreClient.object("chunks", capacity));
  }

  public static void main(String[] args) {
    JsonObject personal = owner("personal", true, false, 16);
    JsonObject selected = owner("selected-but-no-build", false, true, 16);
    JsonObject other = owner("other-team-with-build", true, false, 0);
    JsonArray owners = new JsonArray();
    owners.add(personal);
    owners.add(selected);
    owners.add(other);
    check(
        TeamMenuPolicy.eligibleOwners(owners, "can_build").stream()
            .map(value -> value.get("id").getAsString())
            .toList()
            .equals(List.of("personal", "other-team-with-build")),
        "Contribution choice cannot authorize land or exclude another eligible membership");
    check(
        !TeamMenuPolicy.allowed(selected, "can_spend"),
        "Contribution does not choose a wallet or grant spending");
    check(!TeamMenuPolicy.canClaim(other), "New teams have zero land capacity");
    other.getAsJsonObject("land").addProperty("chunks", 1);
    check(TeamMenuPolicy.canClaim(other), "Earned capacity enables an authorized team claim");
    other.addProperty("used_chunks", 1);
    check(
        !TeamMenuPolicy.canClaim(other), "Used team capacity does not consume personal allowance");
    check(TeamMenuPolicy.canClaim(personal), "A full team cannot disable a personal claim");
    JsonObject malformed = owner("bad", true, true, 16);
    malformed.getAsJsonObject("permissions").addProperty("can_build", "true");
    check(
        !TeamMenuPolicy.allowed(malformed, "can_build"),
        "Capabilities must be authoritative booleans");
    check(
        !TeamMenuPolicy.allowed(new JsonObject(), "can_build"), "Missing capabilities fail closed");
    JsonObject listing = CoreClient.object("kind", "items", "seller", "seller");
    personal.getAsJsonObject("permissions").addProperty("can_spend", true);
    selected.getAsJsonObject("permissions").addProperty("can_spend", true);
    check(
        TeamMenuPolicy.canBuy(personal, listing),
        "Personal wallet can buy separately from contributions");
    check(
        TeamMenuPolicy.canBuy(selected, listing),
        "Spending permission is enough for items without building permission");
    listing.addProperty("kind", "land");
    check(
        !TeamMenuPolicy.canBuy(selected, listing),
        "Land buyer requires both spending and building permission");
    check(
        TeamMenuPolicy.canBuy(personal, listing),
        "Personal land purchase has independent permissions");
    listing.addProperty("seller", "personal");
    check(
        !TeamMenuPolicy.canBuy(personal, listing),
        "The listing seller is excluded from buyer selection");
    check(
        !TeamMenuPolicy.canBuy(other, listing),
        "Building permission alone cannot spend from another team");
    JsonArray required = new JsonArray();
    required.add("pet-owner");
    JsonObject consentAsset =
        CoreClient.object("manifest", CoreClient.object("required_consents", required));
    check(
        TeamMenuPolicy.canConsent(consentAsset, "pet-owner")
            && !TeamMenuPolicy.canConsent(consentAsset, "other-member"),
        "Only the named pet owner can consent");
    check(
        !TeamMenuPolicy.leader(new JsonObject(), ""),
        "Unknown session or leader identity cannot grant a role");

    JsonObject team = owner("explicit-team", true, false, 0);
    JsonObject member = CoreClient.object("account_id", "member", "can_administer", false);
    JsonObject leader = CoreClient.object("account_id", "leader", "can_administer", false);
    check(
        !TeamMenuPolicy.canEditMember(team, member, "viewer"),
        "A membership does not grant role administration");
    team.getAsJsonObject("permissions").addProperty("can_administer", true);
    check(
        TeamMenuPolicy.canEditMember(team, member, "administrator"),
        "Administrator may edit a regular member");
    check(
        !TeamMenuPolicy.canEditMember(team, leader, "administrator"),
        "Leader flags require leadership transfer");
    member.addProperty("can_administer", true);
    check(
        !TeamMenuPolicy.canEditMember(team, member, "administrator"),
        "Only leader may preserve or grant administrator status");
    check(
        TeamMenuPolicy.canEditMember(team, member, "leader"),
        "Leader can administer another member");
    check(
        !TeamMenuPolicy.canEditMember(team, leader, "leader"),
        "Leader cannot edit own leader role");
    JsonObject storedLeader = permissions(false, false);
    storedLeader.addProperty("account_id", "leader");
    check(
        TeamMenuPolicy.effectiveMemberPermissions(team, storedLeader)
            .get("can_spend")
            .getAsBoolean(),
        "Roster shows a selected team's leader's effective permissions, despite unchanged raw"
            + " flags");
    JsonObject otherTeam = owner("another-team", false, true, 0);
    otherTeam.addProperty("leader", "another-leader");
    check(
        !TeamMenuPolicy.effectiveMemberPermissions(otherTeam, storedLeader)
            .get("can_spend")
            .getAsBoolean(),
        "Leadership permissions cannot leak into another membership");
    check(
        !storedLeader.get("can_spend").getAsBoolean(),
        "Displaying effective permissions does not alter role-edit flags");
    JsonObject draft = permissions(true, false);
    JsonObject command = TeamMenuPolicy.permissionsCommand(team, member, draft);
    check(
        command.get("team").getAsString().equals("explicit-team"),
        "Role changes name the selected team");
    check(
        command.get("member").getAsString().equals("member"),
        "Role changes name the selected member");
    check(
        command.get("build").getAsBoolean()
            && !command.get("spend").getAsBoolean()
            && !command.get("administer").getAsBoolean(),
        "Stored flags map independently to scoped command fields");
    check(
        !command.has("contribution_team_id"),
        "Contribution attribution never enters permission commands");
    check(
        CoreClient.object("type", "team_contribution_set", "team", null).get("team").isJsonNull(),
        "No contribution team is an explicit null");
    System.out.println("Paper multi-team capability and command checks passed");
  }
}
