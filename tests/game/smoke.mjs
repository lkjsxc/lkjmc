// Runs against scripts/game_dev.py only. Uses a private proxy fixture, not Mojang authentication.
// Bot protocol 26.1 passes through ViaBackwards into the real Paper 26.2 server.
import fs from 'node:fs/promises';
import crypto from 'node:crypto';
import assert from 'node:assert/strict';
import readline from 'node:readline';
import mineflayer from 'mineflayer';
import { setTimeout as sleep } from 'node:timers/promises';
const local=new URL('../../.local/game/',import.meta.url);
const token=(await fs.readFile(new URL('proxy-token',local),'utf8')).trim();
const ids=JSON.parse(await fs.readFile(new URL('ids.json',local),'utf8'));
const base='http://127.0.0.1:18091';
async function api(path,body){
 const response=await fetch(base+path,{method:'POST',headers:{authorization:'Bearer '+token,'content-type':'application/json'},body:JSON.stringify(body)});
 const result=await response.json();if(!response.ok)throw new Error(JSON.stringify(result));return result;
}
function offlineUUID(name){const bytes=crypto.createHash('md5').update('OfflinePlayer:'+name).digest();bytes[6]=(bytes[6]&15)|48;bytes[8]=(bytes[8]&63)|128;const hex=bytes.toString('hex');return [hex.slice(0,8),hex.slice(8,12),hex.slice(12,16),hex.slice(16,20),hex.slice(20)].join('-');}
async function until(check,description,timeout=120000){const deadline=Date.now()+timeout;while(Date.now()<deadline){const result=await check();if(result)return result;await sleep(500);}throw new Error('Timeout: '+description);}
const connected=[];
async function connect(name){
 const native=offlineUUID(name),sessionId=crypto.randomUUID();
 const session=await api('/internal/v1/game/connect',{issuer:'java',subject:native,display_name:name,native_uuid:native,session_id:sessionId});
 const beat=()=>api('/internal/v1/game/heartbeat',{...session,server_id:ids.official});await beat();
 const timer=setInterval(()=>beat().catch(error=>console.error('heartbeat',error.message)),10000);
 const bot=mineflayer.createBot({host:'127.0.0.1',port:25691,username:name,auth:'offline',version:'26.1',checkTimeoutInterval:120000});
 const entry={bot,session,timer,name,positions:[]};connected.push(entry);
 bot.on('error',error=>console.error(name,'error',error.message));bot.on('kicked',reason=>console.error(name,'kicked',JSON.stringify(reason)));
 bot.on('messagestr',message=>console.log(name,'message',message));
 bot.on('spawn',()=>console.log(name,'spawn',bot.game.dimension,bot.entity.position));
 for(const packet of ['login','respawn'])bot._client.on(packet,data=>{entry.world=data.worldState.name;});
 bot._client.on('position',packet=>entry.positions.push({dimension:entry.world,x:packet.x,y:packet.y,z:packet.z}));
 await until(()=>bot.entity&&entry.world==='minecraft:living',name+' reaches a living point');
 return entry;
}
async function command(entry,command,request_id=crypto.randomUUID()){
 const response=await api('/internal/v1/game/command',{...entry.session,request_id,command});
 if(!response.result.job_id)return response.result;
 return await until(async()=>{
  const job=await api('/internal/v1/game/view',{...entry.session,view:'job',query:{id:response.result.job_id}});
  if(job.state==='failed')throw new Error(job.error);return job.state==='succeeded'?job.result:false;
 },command.type+' job');
}
try{
 const first=await connect('LkjProbeA');const second=await connect('LkjProbeB');
 const a=first.bot.entity.position.clone(),b=second.bot.entity.position.clone();
 assert(Math.hypot(a.x-b.x,a.z-b.z)>=10000,'independent starts must be at least 10,000 horizontal blocks apart');
 for(const client of connected){
  const living=client.positions.filter(position=>position.dimension==='minecraft:living');assert(living.length>0,'must observe an actual living-world position packet');
  for(const position of living)assert(Math.hypot(position.x,position.z)>256,'must not expose a living default spawn');
 }
 console.log('PASS distant first appearances',JSON.stringify({a,b,distance:Math.hypot(a.x-b.x,a.z-b.z)}));
 first.bot.chat('/menu');await until(()=>first.bot.currentWindow,'real game menu opens',10000);
 assert(first.bot.currentWindow.slots.some(item=>item?.name==='compass'));console.log('PASS inventory menu opens');first.bot.closeWindow(first.bot.currentWindow);
 const home=await command(first,{type:'home_set',name:'Native test home'});assert(home.location.world_id);console.log('PASS native home job',JSON.stringify(home));
 await fs.writeFile(new URL('smoke-sessions.json',local),JSON.stringify(connected.map(({name,session})=>({name,session}))),{mode:0o600});
 console.log('READY for console/inventory/respawn checks. Type JSON {"player":0,"command":{...}} or {"chat":"/menu"}; exit to disconnect.');
 const lines=readline.createInterface({input:process.stdin});
 for await(const line of lines){
  if(line==='exit')break;
  try{const input=JSON.parse(line),entry=connected[input.player??0];if(input.chat)entry.bot.chat(input.chat);else if(input.inspect)console.log('INSPECT',JSON.stringify({world:entry.world,position:entry.bot.entity.position,inventory:entry.bot.inventory.items().map(item=>({name:item.name,count:item.count,slot:item.slot})),positions:entry.positions}));else console.log('RESULT',JSON.stringify(await command(entry,input.command)));}catch(error){console.error('COMMAND ERROR',error.message);}
 }
}finally{
 for(const entry of connected){clearInterval(entry.timer);entry.bot.quit();await api('/internal/v1/game/disconnect',entry.session).catch(()=>{});}
}
