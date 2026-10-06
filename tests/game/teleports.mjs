// Actual command/menu movement through the guarded loopback protocol rig.
import assert from "node:assert/strict";
import { observeMenuContents } from "./menu-join.mjs";

export async function teleportChecks(a, b, c, { until, api, session, submit, job, consoleCommand, official }) {
  const aView = observeMenuContents(a.bot), bView = observeMenuContents(b.bot);
  const text = item => JSON.stringify(item?.components ?? item?.nbt ?? {});
  const menu = (view,title) => until(() => view() && JSON.stringify(view().title).includes(title), "teleport menu " + title,15000);
  const choose = async (bot,view,title,material) => {
    const slot=view().slots.slice(0,45).findIndex(item => item?.name===material && text(item).includes(title));
    assert(slot>=0,"visible teleport action: "+title);
    await bot.clickWindow(slot,0,0);
  };
  const home = async player => api("/internal/v1/game/view",{...(await session(player)),view:"home",query:{}});
  const pending = async () => (await home(b)).invitations.filter(item => item.kind==="teleport");
  const knownJobs = async player => new Set((await home(player)).jobs.map(item=>item.id));
  const completed = async (traveler,before) => {
    const result = await until(async () => (await home(traveler)).jobs.find(item=>item.kind==="player.teleport" && !before.has(item.id) && ["succeeded","failed"].includes(item.state)),"actual teleport job completes",30000);
    assert.equal(result.state,"succeeded",JSON.stringify(result));
    for (const observer of [a,b]) assert.equal((await job(observer,result.id)).state,"succeeded","both request participants can read the outcome");
  };
  const originA=a.bot.entity.position.clone(), originB=b.bot.entity.position.clone();
  const bId=(await session(b)).account_id;
  const first=await submit(a,{type:"teleport_request",target:bId});
  const second=await submit(c,{type:"teleport_request",target:bId});
  b.bot.chat("/tpaccept"); await menu(bView,"Teleport requests");
  for (const player of [a,c]) assert(bView().slots.slice(0,45).some(item=>item?.name==="player_head" && text(item).includes(player.name)));
  assert(a.bot.entity.position.distanceTo(originA)<4,"two pending requests require a requester choice");
  const before=await knownJobs(a), am=a.messages.length, bm=b.messages.length;
  await choose(b.bot,bView,a.name,"player_head");
  await completed(a,before);
  await until(()=>a.bot.entity.position.distanceTo(originB)<4,"tpa moved the requester",15000);
  await until(()=>a.messages.slice(am).some(message=>message.includes("Teleport completed")) && b.messages.slice(bm).some(message=>message.includes("Teleport completed")),"actual completion reaches both players",15000);
  assert((await pending()).some(item=>item.id===second.id),"the other requester remains pending");
  b.bot.chat("/tpdeny");
  await until(async()=>!(await pending()).some(item=>item.id===second.id),"one request declines immediately",15000);
  await consoleCommand(official,`execute in minecraft:living run tp ${a.name} ${originA.x} ${originA.y} ${originA.z}`);
  await until(()=>a.bot.entity.position.distanceTo(originA)<4,"return to the previously verified safe test position");
  a.bot.chat("/tpahere"); await menu(aView,"Ask a player to come here");
  await choose(a.bot,aView,b.name,"player_head");
  const aId=(await session(a)).account_id;
  const here=await until(async()=>(await pending()).find(item=>item.sender===aId && item.teleport_here),"direction-specific request",15000);
  await until(()=>b.messages.some(message=>message.includes(a.name) && message.includes("asks you to teleport to them")),"recipient receives direction and requester",15000);
  b.bot.chat(`/lkjmc tp-request ${here.id}`); await menu(bView,"Respond to invitation");
  assert((await pending()).some(item=>item.id===here.id),"notification opens a decision rather than accepting");
  assert(b.bot.entity.position.distanceTo(originB)<4,"recipient has not moved before accepting");
  const beforeHere=await knownJobs(b);
  await choose(b.bot,bView,"Accept","lime_dye");
  await completed(b,beforeHere);
  await until(()=>b.bot.entity.position.distanceTo(a.bot.entity.position)<4,"tpahere moved the recipient",15000);
  const messages=b.messages.length;
  b.bot.chat("/tpaccept");
  await until(()=>b.messages.slice(messages).some(message=>message.includes("No pending teleport requests")),"empty acceptance is an explicit failure",15000);
  console.log("PASS both teleport directions, multi-request selection, explicit notification decisions, empty/decline handling and both-party actual-arrival outcomes");
}
