// Native menus use the same explicit multi-team contract as the web player UI.
// network.mjs supplies its guarded, isolated fixture SQL and authenticated API.
import assert from "node:assert/strict";
import crypto from "node:crypto";
import { observeMenuContents } from "./menu-join.mjs";

export async function teamMenuChecks(c, { until, session, submit, api, fixtureSql }) {
  const bot = c.bot, contents = observeMenuContents(bot);
  const prefix = "Menu" + crypto.randomBytes(3).toString("hex");
  const account = (await session(c)).account_id;
  assert(/^[0-9a-f-]{36}$/.test(account));
  const view = async (name, query = {}) => api("/internal/v1/game/view", {
    ...(await session(c)), view: name, query,
  });
  const teams = () => view("social", { section: "teams" });
  const alpha = await submit(c, { type: "team_create", name: prefix + "Alpha" });
  assert.equal((await teams()).contribution_team_id, alpha.team_id, "First membership becomes contribution team");
  const beta = await submit(c, { type: "team_create", name: prefix + "Beta" });
  assert.equal((await teams()).contribution_team_id, alpha.team_id, "Additional membership preserves contribution choice");
  const life = await view("life");
  assert.equal(life.owners.find(owner => owner.id === alpha.team_id).land.chunks, 0, "New team starts with no land capacity");
  const personal = life.owners.find(owner => owner.id === account).name;
  const otherLeader = crypto.randomUUID();
  const listedAsset = crypto.randomUUID(), listing = crypto.randomUUID(), restrictedAsset = crypto.randomUUID();
  const extra = Array.from({ length: 28 }, (_, i) => ({
    id: crypto.randomUUID(), room: crypto.randomUUID(), name: prefix + "Zulu" + String(i).padStart(2, "0"),
  }));
  await fixtureSql(`BEGIN;
    INSERT INTO principals(id,kind,name) VALUES('${otherLeader}','account','${prefix}Leader');
    INSERT INTO accounts(id) VALUES('${otherLeader}');
    INSERT INTO team_members(team_id,account_id,can_build,can_administer) VALUES('${alpha.team_id}','${otherLeader}',false,false);
    INSERT INTO room_members(room_id,account_id) VALUES('${alpha.room_id}','${otherLeader}');
    UPDATE land_allowances SET chunks=16 WHERE owner='${beta.team_id}';
    ${extra.map(team => `INSERT INTO principals(id,kind,name) VALUES('${team.id}','team','${team.name}');
      INSERT INTO rooms(id,kind,name,owner) VALUES('${team.room}','team','${team.name}','${otherLeader}');
      INSERT INTO teams(id,leader,room_id) VALUES('${team.id}','${otherLeader}','${team.room}');
      INSERT INTO team_members(team_id,account_id,can_build,can_administer) VALUES
        ('${team.id}','${otherLeader}',true,true),('${team.id}','${account}',false,false);
      INSERT INTO room_members(room_id,account_id,role) VALUES
        ('${team.room}','${otherLeader}','owner'),('${team.room}','${account}','member');
      INSERT INTO wallets(owner) VALUES('${team.id}');
      INSERT INTO land_allowances(owner,chunks) VALUES('${team.id}',16);`).join("\n")}
    INSERT INTO assets(id,owner,kind,title,state) VALUES
      ('${listedAsset}','${alpha.team_id}','items','${prefix}Listing','listed'),
      ('${restrictedAsset}','${extra[0].id}','items','${prefix}ReadOnly','escrowed');
    INSERT INTO listings(id,asset_id,seller,price) VALUES('${listing}','${listedAsset}','${alpha.team_id}',100);
    COMMIT;`);
  const restricted = extra[0];
  const itemText = item => JSON.stringify(item?.components ?? item?.nbt ?? {});
  const menu = title => until(() => {
    const window = contents();
    return window && JSON.stringify(window.title).includes(title) && window;
  }, "team menu contents " + title, 15000);
  const pageChanged = previous => until(() => contents() && contents() !== previous, "team menu page contents", 15000);
  const choose = async (title, icon) => {
    for (let page = 0; page < 8; page++) {
      const window = contents();
      assert(window, "Menu contents arrived before selection");
      const slot = window.slots.slice(0, 45).findIndex(item =>
        item && (!icon || item.name === icon) && itemText(item).includes(title));
      if (slot >= 0) { await bot.clickWindow(slot, 0, 0); return; }
      assert(window.slots[50]?.name === "arrow", "Visible explicit action: " + title);
      await bot.clickWindow(50, 0, 0);
      await pageChanged(window);
    }
    assert.fail("Menu pagination did not reach " + title);
  };
  const openTeams = async () => {
    bot.chat("/menu"); await menu("lkjmc");
    await choose("People", "armor_stand"); await menu("People");
    await choose("Teams", "white_banner"); await menu("Teams");
  };
  const confirm = async () => { await choose("Confirm", "lime_concrete"); };
  await openTeams();
  const collectionItems = [];
  for (let page = 0; page < 8; page++) {
    const window = contents();
    assert(window, "Team collection contents are available");
    collectionItems.push(...window.slots.slice(0,45).filter(Boolean));
    if (window.slots[50]?.name !== "arrow") break;
    await bot.clickWindow(50,0,0);
    await pageChanged(window);
    assert(page < 7, "The bounded fixture collection must have an end");
  }
  const createIndex = collectionItems.findIndex(item => itemText(item).includes("Create team"));
  assert(createIndex >= 0, "Create team remains reachable after the existing collection");
  assert.equal(collectionItems[createIndex].name,"writable_book", "Creation is visually distinct from an existing team's banner");
  for (const name of [prefix+"Alpha",prefix+"Beta",...extra.map(team=>team.name)]) {
    const index=collectionItems.findIndex(item=>item.name==="white_banner" && itemText(item).includes(name));
    assert(index>=0 && index<createIndex, "Existing team precedes creation: "+name);
  }
  console.log("Native team collection precedes its distinct Create action across all pages");
  await openTeams();
  await choose(extra.at(-1).name, "white_banner"); await menu(extra.at(-1).name);
  assert(!contents().slots.slice(0, 45).some(item => itemText(item).includes("Invite member")), "Selected team's permissions control invitations");
  await openTeams();
  await choose(prefix + "Alpha", "white_banner"); await menu(prefix + "Alpha");
  for (const label of ["Invite member", "Transfer leadership", "Disband team"])
    assert(contents().slots.slice(0, 45).some(item => itemText(item).includes(label)), "Leader action is scoped: " + label);
  assert(!contents().slots.slice(0, 45).some(item => itemText(item).includes("Leave team")), "Leader must transfer or disband");
  await choose("Members and permissions", "book"); await menu("Members and permissions");
  await choose(prefix + "Leader", "player_head"); await menu("Permissions for " + prefix + "Leader");
  assert(contents().slots.slice(0, 45).some(item => item?.name === "white_banner" && itemText(item).includes(prefix + "Alpha")), "Permission editor names the selected team");
  const beforeToggle = contents();
  await choose("Spend shared coins", "gray_dye"); await pageChanged(beforeToggle);
  await choose("Save", "lime_concrete"); await menu("Save"); await confirm();
  await until(async () => (await view("team", { id: alpha.team_id })).team.members.find(member => member.account_id === otherLeader).can_spend,
    "Native role change in selected team");
  assert.equal((await view("team", { id: restricted.id })).team.members.find(member => member.account_id === otherLeader).can_spend, false,
    "Permission change does not alter the same player's role in another team");
  await openTeams();
  await choose(prefix + "Beta", "white_banner"); await menu(prefix + "Beta");
  await choose("Contribution team", "experience_bottle"); await menu("Update contribution team");
  await confirm();
  await until(async () => (await teams()).contribution_team_id === beta.team_id, "Native explicit contribution selection");
  await menu("Teams");
  await choose("Contribution team", "experience_bottle"); await menu("Contribution team");
  await choose("No team", "gray_dye"); await menu("Update contribution team"); await confirm();
  await until(async () => (await teams()).contribution_team_id === null, "Native no-team contribution choice");
  await menu("Teams");
  await choose(restricted.name, "white_banner"); await menu(restricted.name);
  await choose("Contribution team", "experience_bottle"); await menu("Update contribution team"); await confirm();
  await until(async () => (await teams()).contribution_team_id === restricted.id, "Contribution can select a membership without build rights");
  await menu("Teams");

  bot.chat("/claim"); await menu("Land");
  await choose("Protect this chunk", "oak_fence"); await menu("Choose land owner");
  const ownerItems = contents().slots.slice(0, 45);
  for (const name of [personal, prefix + "Alpha", prefix + "Beta"])
    assert(ownerItems.some(item => itemText(item).includes(name)), "Every eligible personal/team owner is visible: " + name);
  assert(!ownerItems.some(item => itemText(item).includes(restricted.name)), "Contribution selection does not grant land permission");
  await choose(prefix + "Beta", "white_banner");
  await until(() => !bot.currentWindow, "Private claim-name input");
  const claimName = prefix + "Claim";
  bot.chat(claimName); await menu("Protect this chunk");
  assert(contents().slots.slice(0, 45).some(item => itemText(item).includes("Owner: " + prefix + "Beta")), "Confirmation names the chosen owner, independently of contributions");
  await choose("Cancel", "red_concrete");
  assert(!(await view("life")).claims.some(claim => claim.name === claimName), "Cancelling the scoped claim has no effect");

  bot.chat("/menu"); await menu("lkjmc");
  await choose("Market", "emerald"); await menu("Market");
  await choose(prefix + "Listing", "emerald"); await menu(prefix + "Listing");
  await choose("Buy", "emerald"); await menu("Choose buyer");
  const buyers = contents().slots.slice(0, 45);
  assert(buyers.some(item => itemText(item).includes(personal)) && buyers.some(item => itemText(item).includes(prefix + "Beta")), "Personal and authorized team wallets are offered");
  assert(!buyers.some(item => itemText(item).includes(prefix + "Alpha")), "Seller is excluded from buyer choices");
  assert(!buyers.some(item => itemText(item).includes(restricted.name)), "Contribution selection cannot authorize shared spending");
  await choose(prefix + "Beta", "white_banner"); await menu("Buy");
  assert(contents().slots.slice(0, 45).some(item => itemText(item).includes("Owner: " + prefix + "Beta")), "Purchase confirmation names actual wallet and asset owner");
  await choose("Cancel", "red_concrete");
  assert((await view("market")).listings.some(item => item.id === listing), "Cancelled purchase preserves listing and balances");
  bot.chat("/menu"); await menu("lkjmc"); await choose("Market", "emerald"); await menu("Market");
  await choose("Deposit held item", "chest"); await menu("Choose asset owner");
  assert(contents().slots.slice(0, 45).some(item => itemText(item).includes(prefix + "Beta")), "Authorized team asset owner is selectable");
  assert(!contents().slots.slice(0, 45).some(item => itemText(item).includes(restricted.name)), "Deposit selector requires the actual owner's sell permission");
  await choose(prefix + "Beta", "white_banner"); await until(() => !bot.currentWindow, "Private deposit-name input");
  bot.chat(prefix + "Deposit"); await menu("Deposit item");
  assert(contents().slots.slice(0, 45).some(item => itemText(item).includes("Owner: " + prefix + "Beta")), "Deposit confirmation preserves selected team ownership");
  await choose("Cancel", "red_concrete");
  bot.chat("/menu"); await menu("lkjmc"); await choose("Market", "emerald"); await menu("Market");
  await choose(prefix + "ReadOnly", "barrel"); await menu(prefix + "ReadOnly");
  for (const denied of ["Create listing", "Collect into inventory"])
    assert(!contents().slots.slice(0, 45).some(item => itemText(item).includes(denied)), "Stored asset permission remains owner-scoped: " + denied);

  bot.chat("/menu"); await menu("lkjmc");
  await choose("Account", "name_tag"); await menu("Account");
  await choose("Achievements & balance", "experience_bottle"); await menu("Personal and team progress");
  assert(contents().slots.slice(0, 45).some(item => itemText(item).includes("Personal progress")), "Personal achievement scope remains separate");
  await choose("Team progress · " + prefix + "Beta", "experience_bottle"); await menu("Team progress · " + prefix + "Beta");
  assert(contents().slots.slice(0, 45).some(item => item?.name === "gold_ingot" && itemText(item).includes(prefix + "Beta")), "Team progress shows that team's separate balance and allowance");

  await openTeams();
  await choose(restricted.name, "white_banner"); await menu(restricted.name);
  await choose("Leave team", "oak_door"); await menu("Leave team"); await confirm();
  await until(async () => {
    const state = await teams();
    return !state.teams.some(team => team.id === restricted.id) && state;
  }, "Native leave only selected team");
  const retained = await teams();
  assert.equal(retained.contribution_team_id, null, "Leaving selected membership clears contribution selection");
  assert(retained.teams.some(team => team.id === alpha.team_id) && retained.teams.some(team => team.id === beta.team_id), "Other memberships survive explicit leave");
  await menu("Teams"); bot.closeWindow(bot.currentWindow);
  console.log("PASS native multi-team pagination, selected-team capabilities, contribution one/none, scoped land/spending/deposit confirmations, separate progress, and explicit leave");
}
