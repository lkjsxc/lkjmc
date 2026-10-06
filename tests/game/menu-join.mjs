// Acceptance helpers for network.mjs's explicit private offline fixture only.
// These are protocol bots, not production Java authentication or Bedrock proof.
import assert from "node:assert/strict";
import crypto from "node:crypto";
import net from "node:net";
import { setTimeout as sleep } from "node:timers/promises";

const token = (item) => item && JSON.stringify(item.components ?? item.nbt ?? {}).includes("menu_launcher");
const loadedMenuWindows = new WeakMap();

// currentWindow exists after open_window, before its slots arrive. Mineflayer
// emits windowOpen only once window_items has populated that particular window.
export function observeMenuContents(bot) {
  let loaded = loadedMenuWindows.get(bot);
  if (!loaded) {
    loaded = new WeakSet();
    loadedMenuWindows.set(bot, loaded);
    bot.on("windowOpen", (window) => loaded.add(window));
  }
  return () => bot.currentWindow && loaded.has(bot.currentWindow) ? bot.currentWindow : null;
}

export async function playerMenuChecks(c, { until, session }) {
  const bot = c.bot;
  const contents = observeMenuContents(bot);
  const menu = (title) => until(() => {
    const window = contents();
    return window && JSON.stringify(window.title).includes(title);
  }, "menu contents " + title, 15000);
  const itemText = (item) => Array.isArray(item) ? item.map(itemText).join("\n")
    : JSON.stringify(item?.components ?? item?.nbt ?? {});
  const choose = async (title, icon) => {
    const slot = bot.currentWindow.slots.slice(0, 45).findIndex((item) =>
      item && (!icon || item.name === icon) && itemText(item).includes(title));
    assert(slot >= 0, "visible menu action: " + title);
    await bot.clickWindow(slot, 0, 0);
  };
  const main = async () => { await bot.clickWindow(49, 0, 0); await menu("lkjmc"); };
  bot.chat("/menu");
  await menu("lkjmc");
  for (const [title, icon] of [["Play", "compass"], ["Worlds", "grass_block"],
    ["People", "armor_stand"], ["Timeline", "writable_book"], ["Account", "name_tag"]]) {
    assert(bot.currentWindow.slots.slice(0, 45).some((item) =>
      item?.name === icon && itemText(item).includes(title)), "player destination " + title);
  }
  assert.equal(bot.currentWindow.slots[53]?.name, "barrier");
  assert(!itemText(bot.currentWindow.slots.slice(0, 45)).includes("Console"), "player menu has no hosting console");
  await choose("People", "armor_stand");
  await menu("People");
  for (const [title, icon] of [["Friends", "lead"], ["Teams", "white_banner"], ["Party", "campfire"]]) {
    await choose(title, icon);
    await menu(title);
    if (title === "Teams") assert(!itemText(bot.currentWindow.slots.slice(0, 45)).includes("Create party"));
    if (title === "Party") assert(!itemText(bot.currentWindow.slots.slice(0, 45)).includes("Create team"));
    assert.equal(bot.currentWindow.slots[45]?.name, "arrow");
    await bot.clickWindow(45, 0, 0);
    await menu("People");
  }
  await choose("Party", "campfire");
  await menu("Party");
  await choose("Create party", "campfire");
  await until(() => contents() && contents().slots.slice(0,45).some(item => item?.name === "player_head" && itemText(item).includes(c.name)), "created party displays member without a name prompt",15000);
  assert(!itemText(contents().slots.slice(0,45)).includes("Ready for expedition"));
  await choose("Party name", "name_tag");
  await until(() => !bot.currentWindow, "later party naming prompt");
  const renamed = "Together " + c.name;
  bot.chat(renamed);
  await menu(renamed);
  await main(); await choose("People", "armor_stand"); await menu("People");
  await choose("Party", "campfire"); await menu(renamed);
  assert(contents().slots.slice(0,45).some(item => item?.name === "player_head"), "the sole party opens directly, not through another party card");
  await choose("Leave party", "oak_door"); await menu("Leave");
  await choose("Confirm", "lime_concrete"); await menu("Party");
  await until(() => contents() && contents().slots.slice(0,45).some(item => itemText(item).includes("Create party")), "party leave completes");
  await main();
  await choose("Timeline", "writable_book");
  await menu("Timeline");
  await choose("Conversations", "writable_book");
  await menu("Conversations");
  await main();
  await choose("Account", "name_tag");
  await menu("Account");
  await choose("Language", "writable_book");
  await menu("Language");
  await bot.clickWindow(11, 0, 0);
  await until(async () => (await session(c)).language === "ja", "game language saved");
  await menu("lkjmc");
  bot.chat("/menu language");
  await menu("言語");
  await bot.clickWindow(10, 0, 0);
  await until(async () => (await session(c)).language === "en", "English restored");
  await menu("lkjmc");
  await choose("Worlds", "grass_block");
  await menu("Worlds");
  assert.equal(bot.currentWindow.slots[45]?.name, "arrow");
  await bot.clickWindow(45, 0, 0);
  await menu("lkjmc");
  await bot.clickWindow(53, 0, 0);
  await until(() => !bot.currentWindow, "menu closed");
  console.log("PASS player destinations, People grouping, Timeline conversations, language switching, Back/Main/Close");
}

export async function smpMenuChecks(c, { until }) {
  const bot = c.bot;
  const contents = observeMenuContents(bot);
  const menu = (title) => until(() => {
    const window = contents();
    return window && JSON.stringify(window.title).includes(title);
  }, "SMP menu contents " + title, 15000);
  const itemText = (item) => Array.isArray(item) ? item.map(itemText).join("\n")
    : JSON.stringify(item?.components ?? item?.nbt ?? {});
  bot.chat("/menu");
  await menu("lkjmc");
  for (const [slot, title, icon] of [[10, "Homes", "red_bed"], [11, "Land", "oak_fence"],
    [12, "Market", "emerald"], [13, "Expeditions", "ender_eye"],
    [14, "People", "armor_stand"], [15, "Teleport request", "ender_pearl"]]) {
    assert.equal(bot.currentWindow.slots[slot]?.name, icon);
    assert(itemText(bot.currentWindow.slots[slot]).includes(title), "immediate SMP action " + title);
  }
  assert(!itemText(bot.currentWindow.slots.slice(0, 45)).includes("Return to SMP"), "Return belongs to an expedition, not the root");
  assert(!itemText(bot.currentWindow.slots.slice(0, 45)).includes("Help"), "The nonfunctional Help entry is retired");
  await bot.clickWindow(10, 0, 0);
  await menu("Homes");
  assert(!itemText(bot.currentWindow.slots.slice(0, 45)).includes("Protect this chunk"), "Homes has no land controls");
  await bot.clickWindow(49, 0, 0);
  await menu("lkjmc");
  await bot.clickWindow(13, 0, 0);
  await menu("Expeditions");
  assert(!itemText(bot.currentWindow.slots.slice(0, 45)).includes("Private End"), "temporary worlds use Expedition terminology");
  await bot.clickWindow(10, 0, 0);
  await menu("Prepare Expedition");
  assert(itemText(bot.currentWindow.slots.slice(0, 45)).includes("Eyes of Ender"), "preparation shows material requirements");
  assert(itemText(bot.currentWindow.slots.slice(0, 45)).includes("disappear"), "preparation explains temporary-world losses");
  await bot.clickWindow(49, 0, 0);
  await menu("lkjmc");
  await bot.clickWindow(53, 0, 0);
  await until(() => !bot.currentWindow, "SMP menu closed");
  console.log("PASS contextual SMP actions, separated Homes/Land, Expedition requirements and lifetime consequences");
}

export async function launcherChecks(c, { until, consoleCommand, lobby, reconnect }) {
  const bot = c.bot;
  const contents = observeMenuContents(bot);
  await until(() => token(bot.inventory.slots[36]), "tagged first-slot nether star installed");
  assert.equal(bot.inventory.slots[36].name, "nether_star");
  bot.setQuickBarSlot(0);
  const opened = () => until(contents, "launcher menu contents", 10000);
  const close = async () => {
    bot.closeWindow(bot.currentWindow);
    await until(() => !bot.currentWindow, "launcher menu closed");
    await sleep(400); // Test distinct gestures outside the deliberate debounce.
  };
  let count = 0;
  bot.on("windowOpen", () => count++);
  bot._client.write("arm_animation", { hand: 0 });
  await opened();
  await close();
  const before = count;
  bot.activateItem();
  bot.activateItem();
  await opened();
  await sleep(500);
  assert.equal(count, before + 1, "double right trigger opens exactly one view");
  await close();
  const block = bot.blockAt(bot.entity.position.offset(0, -1, 0));
  assert(block && block.name !== "air", "lobby floor fixture");
  bot._client.write("block_dig", { status: 0, location: block.position, face: 1, sequence: 0 });
  await opened();
  await close();
  bot.activateBlock(block);
  await opened();
  await close();
  assert.equal(bot.blockAt(block.position).name, block.name, "left-block launcher does not break the floor");
  // Existing players may still own the tagged compass from the prior adapter.
  await consoleCommand(lobby, `item replace entity ${c.name} hotbar.0 with minecraft:compass[minecraft:custom_data={PublicBukkitValues:{"lkjmc:menu_launcher":1b}}]`);
  await until(() => bot.inventory.slots[36]?.name === "compass" && token(bot.inventory.slots[36]), "legacy tagged compass fixture");
  bot.activateItem();
  await opened();
  await close();
  const entityTag = "ux_menu_" + c.name;
  await consoleCommand(lobby, `execute at ${c.name} run summon minecraft:cow ~2 ~ ~ {NoAI:1b,Tags:["${entityTag}"]}`);
  const cow = await until(() => Object.values(bot.entities).find((entity) => entity.name === "cow" && entity.position.distanceTo(bot.entity.position) < 4), "menu entity fixture");
  bot.activateEntity(cow);
  await opened();
  await close();
  await consoleCommand(lobby, `kill @e[type=minecraft:cow,tag=${entityTag}]`);
  await consoleCommand(lobby, `item replace entity ${c.name} weapon.offhand from entity ${c.name} hotbar.0`);
  bot.activateItem(true);
  await opened();
  await close();
  await consoleCommand(lobby, `item replace entity ${c.name} weapon.offhand with minecraft:stick`);
  bot._client.write("block_dig", { status: 6, location: { x: 0, y: 0, z: 0 }, face: 0, sequence: 0 });
  await sleep(300);
  assert(token(bot.inventory.slots[36]) && bot.inventory.slots[45]?.name === "stick", "hand swap involving launcher is cancelled");
  await bot.clickWindow(36, 0, 0); // Native player inventory slot, window 0.
  await opened();
  await close();
  await bot.clickWindow(36, 1, 0);
  await opened();
  await close();
  await bot.clickWindow(36, 0, 2); // Number-key swap must not move the owned token.
  await sleep(300);
  assert(token(bot.inventory.slots[36]));
  bot._client.write("block_dig", { status: 4, location: { x: 0, y: 0, z: 0 }, face: 0, sequence: 0 });
  await sleep(300);
  assert(token(bot.inventory.slots[36]), "owned launcher cannot be dropped");
  // A deliberately named ordinary book has no PDC authority.
  await consoleCommand(lobby, `item replace entity ${c.name} hotbar.1 with minecraft:book[minecraft:custom_name='"Game menu"']`);
  bot.setQuickBarSlot(1);
  bot.activateItem();
  await sleep(500);
  assert(!bot.currentWindow, "user-crafted named item is not a launcher");
  // The first slot may contain ordinary property; rejoin relocates it rather than discarding it.
  await consoleCommand(lobby, `item replace entity ${c.name} hotbar.0 with minecraft:diamond 3`);
  const next = await reconnect(c);
  await until(() => token(next.bot.inventory.slots[36]) && next.bot.inventory.slots[36].name === "nether_star", "first-slot menu restored");
  assert.equal(next.bot.inventory.items().filter(item => item.name === "diamond").reduce((n,item) => n+item.count,0),3,"ordinary first-slot diamonds survive migration");
  assert.equal(next.bot.inventory.slots[37]?.name, "book", "ordinary named book survives");
  assert.equal(next.bot.inventory.slots.filter(token).length, 1, "exactly one owned token");
  console.log("PASS launcher left/right AIR/BLOCK, inventory left/right, legacy compass, offhand/entity, debounce, protected swap/drop and ordinary-item preservation");
  return next;
}

export async function sleepingJoinChecks(c, helpers) {
  const { until, submit, job, session, ids, fixtureSql, startOfficial, reconnect } = helpers;
  const name = "official development";
  await fixtureSql(`UPDATE servers SET observed='stopped',desired='stopped',last_observed_at=now(),error=NULL WHERE id='${ids.official}'`);
  const first = await submit(c, { type: "server_join", id: ids.official });
  const duplicate = await submit(c, { type: "server_join", id: ids.official });
  assert.equal(duplicate.job_id, first.job_id, "duplicate sleeping target coalesces");
  await until(() => c.messages.some((m) => m.includes(`Waking ${name}`)), "waking chat feedback");
  c.bot.chat("/go cancel");
  await until(async () => (await job(c, first.job_id)).state === "cancelled", "waiting request cancelled");
  await until(() => c.messages.some((m) => m.includes("Travel cancelled")), "cancellation chat feedback");
  assert.equal((await session(c)).server_id, ids.lobby);
  const old = await submit(c, { type: "server_join", id: ids.official });
  const chosen = await submit(c, { type: "server_join", id: ids.lobby });
  await until(async () => (await job(c, old.job_id)).state === "cancelled", "new target supersedes waiting request");
  await until(async () => (await job(c, chosen.job_id)).state === "succeeded", "new chosen lobby observed");
  const stale = await submit(c, { type: "server_join", id: ids.official });
  const oldSession = (await session(c)).session_id;
  const next = await reconnect(c);
  assert.notEqual((await session(next)).session_id, oldSession);
  await until(async () => (await job(next, stale.job_id)).state === "failed", "old session job fails after reconnect");
  await sleep(6500);
  assert.equal((await session(next)).server_id, ids.lobby, "stale request cannot move the reconnected person");
  const messageStart = next.messages.length;
  // A vanilla Velocity /server request previously woke the backend and
  // abandoned the player. Do not call the durable command on their behalf
  // from the fixture: the real pre-connect event must create that intent.
  const context = await session(next);
  for (const id of [context.account_id, context.session_id, ids.official]) assert(/^[0-9a-f-]{36}$/.test(id));
  next.bot.chat(`/server lkjmc-${ids.official}`);
  const travelId = await until(async () => {
    const result = await fixtureSql(`SELECT id FROM jobs WHERE actor='${context.account_id}' AND kind='player.join' AND server_id='${ids.official}' AND payload->>'session_id'='${context.session_id}' ORDER BY created_at DESC LIMIT 1`);
    return result.stdout.match(/[0-9a-f]{8}-[0-9a-f-]{27}/)?.[0];
  }, "ordinary first-attempt connection retains a durable destination",15000);
  const travel = { job_id:travelId };
  await until(() => next.messages.slice(messageStart).some((m) => m.includes(`Waking ${name}`)), "sleeping target waking");
  assert.equal((await session(next)).server_id, ids.lobby);
  // This fixture manually performs the host's start after Core has durably queued it.
  // Paper was absent up to this point; verified readiness comes from the real adapter.
  await fixtureSql(`UPDATE servers SET observed='starting',last_observed_at=now() WHERE id='${ids.official}'`);
  const officialStarting = startOfficial();
  await until(() => next.messages.slice(messageStart).some((m) => m.includes(`Preparing ${name}`)), "preparing transition feedback");
  assert.notEqual((await job(next, travel.job_id)).state, "succeeded", "preparing is not success");
  const official = await officialStarting;
  const done = await until(async () => {
    const status = await job(next, travel.job_id);
    return ["succeeded", "failed"].includes(status.state) && status;
  }, "sleeping server eventual transfer", 610000);
  assert.equal(done.state, "succeeded", JSON.stringify(done));
  assert.equal(done.result.session_id, (await session(next)).session_id);
  await until(() => next.world === "minecraft:living", "actual protocol world arrival");
  await until(() => next.messages.slice(messageStart).some((m) => m.includes(`Arrived at ${name}`)), "actual success chat");
  assert(!next.ended);
  const progress = next.messages.slice(messageStart).filter((m) => /Waking |Preparing |Saving and connecting/.test(m));
  assert(progress.length <= 34, "bounded progress, not every poll");
  console.log("PASS first ordinary /server attempt -> durable waiting -> waking -> actual arrival without a second command; duplicate, supersession, cancellation and stale reconnect");
  return { client: next, official };
}

export async function failedJoinChecks(c, { fixtureSql, submit, job, until, session, ids, destinationPort = 25799, destinationName = "unavailable fixture", move }) {
  // A closed loopback destination exercises an ordinary connection failure while
  // the real source Paper performs its signed departure/save and release.
  const target = crypto.randomUUID();
  const player = await session(c);
  await fixtureSql(`INSERT INTO servers(id,name,kind,visibility,desired,observed,version,software,memory_mib,cpu_millis,storage_mib,last_observed_at,capabilities,address) VALUES('${target}','${destinationName}','legacy','public','running','running','fixture','paper',2048,2000,10240,now(),'{"proxy_join":true}', '127.0.0.1:${destinationPort}')`);
  const before = c.messages.length;
  const travel = await submit(c, { type: "server_join", id: target });
  const done = await until(async () => {
    const j = await job(c, travel.job_id);
    return j.state === "failed" && j;
  }, "closed destination fails promptly", 45000);
  assert.equal(done.result.effect, "none");
  assert.equal((await session(c)).session_id, player.session_id);
  const actual = (await session(c)).server_id;
  assert([ids.official, ids.lobby].includes(actual), "failure retains the source or returns safely to the lobby");
  assert(!c.ended, "ordinary connection failure keeps the player connected");
  await until(() => c.messages.slice(before).some((m) => m.includes(`Travel to ${destinationName} failed:`)), "actionable target-specific failure chat");
  const source = c.bot.entity.position.clone();
  c.bot.setControlState("forward", true);
  await until(() => c.bot.entity.position.distanceTo(source) > .2, "signed departure lock released", 10000);
  c.bot.setControlState("forward", false);
  await fixtureSql(`UPDATE servers SET visibility='private' WHERE id='${target}'`);
  if (actual === ids.lobby) await move(c, ids.official);
  console.log("PASS connection failure feedback, source/lobby retained and signed departure released");
}

export async function timeoutJoinChecks(c, helpers) {
  const sockets = new Set();
  const stalled = net.createServer((socket) => {
    sockets.add(socket);
    socket.on("close", () => sockets.delete(socket));
    // Accept TCP but never reply to the Minecraft handshake. Velocity must reach
    // its bounded read timeout and release the source instead of disconnecting it.
  });
  await new Promise((resolve, reject) => {
    stalled.once("error", reject);
    stalled.listen(0, "127.0.0.1", resolve);
  });
  try {
    await failedJoinChecks(c, { ...helpers, destinationPort: stalled.address().port,
      destinationName: "timeout fixture" });
    console.log("PASS stalled destination times out with failure chat and signed release; player remains connected");
  } finally {
    for (const socket of sockets) socket.destroy();
    await new Promise((resolve) => stalled.close(resolve));
  }
}

export async function sameWorldPlayerChecks(a, b, { until, session, api }) {
  const aContents = observeMenuContents(a.bot), bContents = observeMenuContents(b.bot);
  await until(() => a.bot.players[b.name] && b.bot.players[a.name]
    && [...a.tabEntries.values()].some(entry=>entry.name===b.name && entry.listed)
    && [...b.tabEntries.values()].some(entry=>entry.name===a.name && entry.listed),
    "same-world players are explicitly listed in both Tab UIs despite distant chunks", 15000);
  const itemText = item => JSON.stringify(item?.components ?? item?.nbt ?? {});
  const menu = (contents,title) => until(() => contents() && JSON.stringify(contents().title).includes(title),"player selection menu " + title,15000);
  const choose = async (bot,contents,title,material) => {
    const index = contents().slots.slice(0,45).findIndex(item => item?.name === material && itemText(item).includes(title));
    assert(index >= 0,"visible player action: " + title);
    await bot.clickWindow(index,0,0);
  };
  a.bot.chat("/menu"); await menu(aContents,"lkjmc");
  await choose(a.bot,aContents,"Teleport request","ender_pearl"); await menu(aContents,"Teleport request");
  await choose(a.bot,aContents,"Go to a player","ender_pearl"); await menu(aContents,"Go to a player");
  const candidate = aContents().slots.slice(0,45).findIndex(item => item?.name === "player_head" && itemText(item).includes(b.name));
  const manual = aContents().slots.slice(0,45).findIndex(item => item?.name === "name_tag" && itemText(item).includes("Enter the other player’s name"));
  assert(candidate >= 0 && manual > candidate, "same-world candidate precedes optional name input: " + JSON.stringify({candidate,manual,items:aContents().slots.slice(0,45).filter(Boolean).map(item => ({name:item.name,text:itemText(item)}))}));
  await a.bot.clickWindow(candidate,0,0);
  const readInvites = async () => (await api("/internal/v1/game/view",{...(await session(b)),view:"home",query:{}})).invitations;
  const aId = (await session(a)).account_id;
  const invite = await until(async () => (await readInvites()).find(invite => invite.kind === "teleport" && invite.sender === aId),"listed target received the teleport request",15000);
  b.bot.chat("/menu"); await menu(bContents,"lkjmc");
  await choose(b.bot,bContents,"Timeline","writable_book"); await menu(bContents,"Timeline");
  await choose(b.bot,bContents,"Invitations","bell"); await menu(bContents,"Invitations");
  await choose(b.bot,bContents,a.name,"player_head"); await menu(bContents,"Respond to invitation");
  assert((await readInvites()).some(item => item.id === invite.id),"opening a request does not silently accept it");
  await choose(b.bot,bContents,"Decline","gray_dye");
  await until(async () => !(await readInvites()).some(item => item.id === invite.id),"decline resolves exactly the shown request",15000);
  console.log("PASS distant same-world tab entries, player-list-first request and explicit invitation decision");
}
