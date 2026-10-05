import fs from "node:fs/promises";
const catalogRoot = new URL("../../locales/messages.json", import.meta.url);
const manifest = JSON.parse(await fs.readFile(catalogRoot, "utf8"));
const messageIds = new Map(Object.entries(manifest).map(([id,entry])=>[entry.source,id]));
export const systemFixtureMessage = (source, params = {}) => typeof source === "object" ? source : ({id:messageIds.get(source) ?? "system.unknown", params: messageIds.has(source) ? params : {reference:"fixture-message"}});
import path from "node:path";
import { fileURLToPath } from "node:url";
export const sid = "00000000-0000-0000-0000-000000000001";
export const otherSid = "00000000-0000-0000-0000-000000000009";
export const aid = "00000000-0000-0000-0000-000000000002";
export const rid = "00000000-0000-0000-0000-000000000003";
export const groupId = "00000000-0000-0000-0000-000000000004";
export const teamId = "00000000-0000-0000-0000-000000000006";
const root = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "../dist",
);
export async function mountFixture(context, { language = "en", width } = {}) {
  const state = {
    language,
    administrator:true,
    identities: [{issuer:"java",subject:"verified-fixture-java",display_name:"Alex"}],
    play:{preferred_server_id:sid,identity_ready:true,game_session:{server_id:sid,client:"java"}},
    worlds:null,
    expeditions:[],
    expeditionCost:{coins:1000,ender_eyes:12},
    durationSeconds:10800,
    preparation:{is_leader:true,available_coins:10000,can_prepare:true,participants:[{account_id:aid,name:"Alex",ready:true,online:true,in_combat:false,occupied:false}]},
    expeditionPages:{},
    accountId: aid,
    hosting: {
      limits:{server_count:3,concurrent_servers:2,memory_mib:4096,cpu_millis:4000,storage_mib:32768},
      owned:{server_count:1,storage_mib:16384},
      reserved:{server_count:1,memory_mib:2048,cpu_millis:2000},
      remaining:{server_count:2,storage_mib:16384,concurrent_servers:1,memory_mib:2048,cpu_millis:2000},
      minimum_server_storage_mib:1024,minimum_server_memory_mib:512,minimum_server_cpu_millis:1000,server_cpu_step_millis:1000,
      can_create:true,creation_blocked_reason:null,
    },
    csrf: "fixture",
    requests: [],
    failures: {},
    responseDelays: {},
    updates: [],
    removedIds: [],
    commands: [],
    timelineRequests: [],
    jobGets: [],
    failNext: {},
    delays: {},
    jobs: new Map(),
    pausedJobs: false,
    sendFailure: false,
    tailVersion: 0,
    files: new Map([
      ["", { kind: "directory" }],
      ["documents", { kind: "directory" }],
      [
        "documents/notes.txt",
        { kind: "file", text: "enabled: true\n", sha: "sha-original" },
      ],
      ["notes.txt", { kind: "file", text: "server notes\n", sha: "sha-notes" }],
    ]),
    logLines: {
      live: ["[INFO] Fixture server stdout"],
      "2026-10-01": ["[INFO] Historical October 1"],
      "2026-10-02": ["[INFO] Historical October 2"],
    },
  };
  const team = {
    id:teamId,name:"Builders team",leader:"bea",room_id:groupId,
    permissions:{can_build:true,can_sell:true,can_spend:true,can_manage_members:false,can_administer:false},
    member_count:2,is_contribution_team:true,
    members:[{account_id:aid,name:"Alex"},{account_id:"bea",name:"Bea"}],
  };
  state.server = {
    id: sid,
    status:{machine_state:"running",game_state:"stopped",observation_fresh:true,joinable:false,activity:null,actions:{join:{allowed:true,reason:null},start:{allowed:true,reason:null},stop:{allowed:false,reason:"already_stopped"},logs:{allowed:true,reason:null},files:{allowed:true,reason:null}}},
    name: "Workshop",
    kind: "custom",
    software: "paper",
    version: "1.21.11",
    desired: "stopped",
    observed: "stopped",
    maintenance: false,
    inspection: { id: "fixture-inspection", state: "ready", guest_ready: true, expires_at: "2026-10-03T10:10:00Z" },
    can_manage: true,
    can_administer: true,
    memory_mib: 2048,
    cpu_millis: 2000,
    storage_mib: 16384,
    visibility: "private",
    players: 0,
    last_observed_at: new Date().toISOString(),
    capabilities: { proxy_join: true, bedrock: true },
    members: [
      {
        account_id: aid,
        name: "Alex",
        role: "operator",
        minecraft_operator_job: null,
        minecraft_identity: { ready: true, identity: { uuid: "verified-fixture-java", name: "Bea" } },
      },
    ],
    backups: [],
  };
  state.rooms = [
    {
      id: rid,
      kind: "dm",
      name: "Alex and Bea",
      role: "member",
      members: [
        { id: aid, name: "Alex" },
        { id: "bea", name: "Bea" },
      ],
    },
    {
      id: groupId,
      kind: "group",
      name: "Builders",
      role: "owner",
      members: [{ id: aid, name: "Alex" }],
    },
  ];
  state.message = (n, body, roomId = rid) => ({
    id: `message:${n}`,
    message_id: n,
    type: "message",
    room_id: roomId,
    room_name: roomId === rid ? "Alex and Bea" : "Builders",
    room_kind: roomId === rid ? "dm" : "group",
    author: n % 2 ? aid : "bea",
    author_name: n % 2 ? "Alex" : "Bea",
    created_at: `2026-10-03T08:${String(Math.floor(n / 60)).padStart(2, "0")}:${String(n % 60).padStart(2, "0")}Z`,
    body,
  });
  state.items = Array.from({ length: 24 }, (_, i) =>
    state.message(i + 10, `Message ${i + 10}`),
  );
  state.items.push(
    {
      id: "job:completed",
      type: "job",
      job_id: "completed",
      kind: "server.backup",
      state: "succeeded",
      server_id: sid,
      server_name: "Workshop",
      created_at: "2026-10-03T08:01:00Z",
    },
    {
      id: "notification:1",
      notification_id: 1,
      type: "notification",
      kind: "transfer",
      created_at: "2026-10-03T08:01:01Z",
      body: { amount: 40, target_name: "Alex", message: systemFixtureMessage("Coins received") },
    },
  );
  state.jobs.set("completed", {
    id: "completed",
    kind: "server.backup",
    state: "succeeded",
    server_id: sid,
    server_name: "Workshop",
    result: { backup_id: "backup-123", verified: true },
    created_at: "2026-10-03T08:01:00Z",
  });
  const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
  await context.route("**/*", async (route) => {
    if (!route.request().url().startsWith("https://ux.fixture/"))
      return route.abort("blockedbyclient");
    const url = new URL(route.request().url()),
      p = url.pathname;
    state.requests.push(p);
    const json = async (body) => {
      const snapshot = JSON.stringify(body);
      if (state.responseDelays[p]) await sleep(state.responseDelays[p]);
      return route.fulfill({ body: snapshot, contentType: "application/json" });
    };
    const error = (message, status = 409) => {
      let payload=systemFixtureMessage(message);
      if (typeof message === "string" && payload.id === "system.unknown") payload=status===403?{id:"error.forbidden",params:{}}:status===404?{id:"error.not_found",params:{}}:{id:"text.request_failed_0",params:{"0":status}};
      return route.fulfill({status,json:{error:{message:payload}}});
    };
    if (!p.startsWith("/api/") && !p.startsWith("/health/")) {
      const file = p === "/" ? "index.html" : p.slice(1);
      if (!/^([a-zA-Z0-9_.-]+\/)*[a-zA-Z0-9_.-]+$/.test(file))
        return error("Invalid fixture asset", 404);
      try {
        return route.fulfill({
          body: await fs.readFile(path.join(root, file)),
          contentType: file.endsWith(".js")
            ? "text/javascript"
            : file.endsWith(".css")
              ? "text/css"
              : "text/html",
        });
      } catch {
        return error("Fixture asset not found", 404);
      }
    }
    if (state.failures[p])
      return error("Fixture access failure", state.failures[p]);
    if (p === "/api/v1/me")
      return json({
        account: {
          id: state.accountId,
          name: "Alex",
          administrator: state.administrator,
          language: state.language,
          rank: {
            name: "Member",
            server_count: 3,
            concurrent_servers: 2,
            memory_mib: 4096,
            cpu_millis: 4000,
            storage_mib: 32768,
          },
          identities: state.identities,
          dm_policy: "friends",
          activity_policy: "friends",
        },
        csrf: state.csrf,
        game_address: "example.test:25591",
        voice_available: false,
        development: false,
      });
    if (p === "/health/ready") return json({ login_configured: true,game_address:"example.test:25591" });
    if (p === "/api/v1/players")
      return json({ players: [{ id: "bea", name: "Bea", rank: "Member" }] });
    if (p === "/api/v1/rooms") {
      const known = (url.searchParams.get("known") ?? "").split(",").filter(Boolean);
      return json({ rooms: state.rooms, rooms_next_cursor: null,
        removed_room_ids: known.filter((id) => !state.rooms.some((r) => r.id === id)) });
    }
    if (p === "/api/v1/timeline") {
      state.timelineRequests.push(url.search);
      if (state.timelineFailure)
        return error("Timeline temporarily unavailable", 503);
      if (state.delays.timeline) await sleep(state.delays.timeline);
      let items = url.searchParams.has("before")
        ? Array.from({ length: 10 }, (_, i) => state.message(i, `Earlier ${i}`))
        : state.items;
      const room = url.searchParams.get("room"),
        kind = url.searchParams.get("kind");
      if (room)
        items = items.filter(
          (i) => i.room_id === room || (i.type !== "message" && room === rid),
        );
      if (kind === "messages")
        items = items.filter((i) => i.type === "message");
      if (kind === "events") items = items.filter((i) => i.type !== "message");
      return json({
        items,
        rooms: state.rooms,
        updates: state.updates.filter((item) =>
          (url.searchParams.get("known") ?? "")
            .split(",")
            .includes(item.id),
        ),
        removed_ids: state.removedIds,
        next_cursor: url.searchParams.has("before")
          ? null
          : "opaque/page:older",
      });
    }
    if (p.startsWith("/api/v1/jobs/")) {
      const id = p.split("/").at(-1);
      state.jobGets.push(id);
      if (state.failNext.job) {
        const message = state.failNext.job;
        delete state.failNext.job;
        return error(message, 503);
      }
      const job = state.jobs.get(id);
      if (!job) return error("Missing job", 404);
      job.gets = (job.gets ?? 0) + 1;
      if (state.delays.job) await sleep(state.delays.job);
      if (job.gets < 2 || state.pausedJobs)
        return json({
          ...job,
          state: "leased",
          result: null,
          error: null,
          progress: { message: systemFixtureMessage("Waiting to resume") },
        });
      if (job.onDone) {
        job.onDone();
        delete job.onDone;
      }
      return json(job);
    }
    if (p === "/api/v1/reports/preview")
      return json({
        evidence: state.items
          .filter((i) =>
            route.request().postDataJSON().message_ids.includes(i.message_id),
          )
          .map((i) => ({ ...i, id: i.message_id })),
      });
    if (p.endsWith("/artifacts"))
      return json({ id: "artifact-fixture", name: "attachment.txt" });
    if (p === "/api/v1/commands") {
      const request = route.request().postDataJSON();
      const c = request.command;
      state.commands.push(c);
      if (state.delays[c.type]) await sleep(state.delays[c.type]);
      if (state.failNext[c.type]) {
        const message = state.failNext[c.type];
        delete state.failNext[c.type];
        return error(message);
      }
      if (
        state.hostRefusals &&
        ["server_files", "server_file_read", "server_logs"].includes(c.type) &&
        (state.server.observed === "stopped" ||
          /^(plugins|config)(\/|$)/.test(c.path ?? ""))
      )
        return error(
          "Host read unavailable for sleeping VM or protected path",
          403,
        );
      if (c.type === "language") {
        state.language = c.language;
        return json({ result: { language: c.language } });
      }
      if (c.type === "message_send") {
        if (state.sendFailure)
          return error(
            "Could not send to this conversation. Please retry.",
            503,
          );
        const n = state.items.length + 100;
        state.items.push(state.message(n, c.body, c.room));
        return json({ result: { sent: true } });
      }
      if (c.type === "message_delete") {
        state.items = state.items.map((i) =>
          i.message_id === c.id
            ? { ...i, deleted_at: new Date().toISOString() }
            : i,
        );
        return json({ result: { deleted: true } });
      }
      if (c.type === "direct_room") return json({ result: { room_id: rid } });
      if (c.type === "room_create")
        return json({ result: { room_id: groupId } });
      if (c.type === "server_configure") {
        state.server.name = c.name;
        state.server.visibility = c.visibility;
        return json({ result: { updated: true } });
      }
      if (c.type === "server_create") return json({result:{server_id:otherSid,state:"queued"}});
      if (
        [
          "server_logs",
          "server_files",
          "server_file_read",
          "server_file_write",
          "server_file_delete",
          "server_directory_create",
          "server_install",
          "server_console",
          "server_operator",
          "server_inspection",
          "server_start",
          "server_join",
          "asset_place",
          "expedition_prepare",
          "expedition_enter",
          "expedition_cancel",
          "expedition_return",
        ].includes(c.type)
      ) {
        const id = `fixture-${state.commands.length}`;
        let result = {},
          failure,
          onDone;
        if (c.type === "server_inspection") {
          state.server.inspection = c.open ? { id, state: "opening", guest_ready: false } : { ...state.server.inspection, state: "closing" };
          result = { open: c.open, guest_ready: c.open, effect: "committed" };
          onDone = () => { state.server.inspection = c.open ? { id, state: "ready", guest_ready: true, expires_at: "2026-10-03T10:10:00Z" } : null; };
        }
        if (c.type === "server_logs")
          result = {
            lines: state.logLines[c.date ?? "live"] ?? [],
            date: c.date,
            dates: ["2026-10-01", "2026-10-02"],
            truncated: false,
          };
        if (c.type === "server_files")
          result = {
            path: c.path,
            entries: [...state.files]
              .filter(
                ([p]) =>
                  p &&
                  p.slice(
                    0,
                    p.lastIndexOf("/") < 0 ? 0 : p.lastIndexOf("/"),
                  ) === c.path,
              )
              .map(([p, f]) => ({
                name: p.split("/").at(-1),
                path: p,
                kind: f.kind,
                bytes: f.text?.length,
                modified_at: "2026-10-03T08:00:00Z",
              })),
          };
        if (c.type === "server_file_read") {
          const f = state.files.get(c.path);
          if (f?.text)
            result = {
              path: c.path,
              text: f.text,
              sha256: f.sha,
              bytes: f.text.length,
            };
          else failure = "File is not bounded UTF-8 text.";
        }
        if (c.type === "server_file_write") {
          const current = state.files.get(c.path);
          if ((current?.sha ?? null) !== c.expected_sha256)
            failure = "File changed on server. Reload before saving.";
          else {
            result = { path: c.path, sha256: "sha-saved", effect: "committed" };
            onDone = () =>
              state.files.set(c.path, {
                kind: "file",
                text: c.text,
                sha: "sha-saved",
              });
          }
        }
        if (c.type === "server_file_delete") {
          if (state.files.get(c.path)?.sha !== c.expected_sha256)
            failure = "File changed on server. Reload before deleting.";
          else {
            result = { path: c.path, effect: "committed" };
            onDone = () => state.files.delete(c.path);
          }
        }
        if (c.type === "server_directory_create") {
          result = { path: c.path, effect: "committed" };
          onDone = () => state.files.set(c.path, { kind: "directory" });
        }
        if (c.type === "server_console") {
          result = { sent: c.line };
          onDone = () => state.logLines.live.push(`[INFO] Received ${c.line}`);
        }
        if (c.type === "server_operator") {
          result = {
            member: c.member,
            operator: c.operator,
            effective: "next_start",
            effect: "committed",
          };
          onDone = () =>
            (state.server.members[0].minecraft_operator_job = {
              id,
              state: "succeeded",
              operator: c.operator,
              result,
            });
        }
        if (c.type === "asset_place")
          result = {
            clear: true,
            preview_hash: "preview-safe",
            summary: { blocks: 50 },
          };
        if (c.type === "server_join")
          result = state.joinResult ?? { effect:"committed",session_id:"fixture-session",server_id:c.id,actual_server_id:c.id };
        if (c.type === "expedition_prepare") {result={expedition_id:"fixture-expedition"};onDone=()=>state.expeditions.unshift({id:"fixture-expedition",owner:aid,state:"active",created_at:"2026-10-03T10:00:00Z",opens_at:"2026-10-03T10:00:00Z",expires_at:new Date(Date.parse("2026-10-03T10:00:00Z")+state.durationSeconds*1000).toISOString(),participants:[{account_id:aid,name:"Alex"}],can_enter:true,can_return:false,can_cancel:false});}
        if (c.type.startsWith("expedition_") && c.type !== "expedition_prepare") {result={effect:"committed",expedition_id:c.id};onDone=()=>{const entry=state.expeditions.find(e=>e.id===c.id);if(entry && c.type==="expedition_cancel"){entry.state="refunded";entry.can_cancel=false;}if(entry && c.type==="expedition_enter"){entry.can_enter=false;entry.can_return=true;}if(entry && c.type==="expedition_return"){entry.can_enter=true;entry.can_return=false;}};}
        const job = {
          id,
          kind: c.type.replaceAll("_", "."),
          state: failure ? "failed" : "succeeded",
          error: failure ? systemFixtureMessage(failure.startsWith("File changed")?"File changed or already exists; read it again before saving":failure) : undefined,
          result,
          server_id: c.type === "asset_place" ? sid : c.id,
          server_name: state.server.name,
          onDone,
        };
        state.jobs.set(id, job);
        return json({ result: { job_id: id } });
      }
      return json({ result: { updated: true } });
    }
    if (p === "/api/v1/view/expedition") return json({expeditions:state.expeditions.slice(0,100),preparation:state.preparation,cost:state.expeditionCost,duration_seconds:state.durationSeconds,destination:"end",lifetime:"temporary",access:"participants"});
    if (p === "/api/v1/expeditions") {
      const cursor = url.searchParams.get("cursor");
      const page = state.expeditionPages[cursor ?? "latest"];
      if (page) return json(page);
      const offset = cursor === "fixture25" ? 25 : 0;
      return json({expeditions:state.expeditions.slice(offset,offset+25),next_cursor:state.expeditions.length > offset+25 ? "fixture25" : null});
    }
    if (p.startsWith("/api/v1/expeditions/")) {
      const id = p.split("/").at(-1);
      const expedition = state.expeditions.find(entry => entry.id === id) ?? Object.values(state.expeditionPages).flatMap(page=>page.expeditions ?? []).find(entry=>entry.id === id);
      return expedition ? json({expedition}) : error("Not found",404);
    }
    if (p.startsWith("/api/v1/servers/")) {
      const server = p.includes(otherSid)
        ? { ...state.server, id: otherSid, name: "Second server" }
        : state.server;
      return json({
        server,
        play:state.play,
        servers: [server],
        jobs: [...state.jobs.values()].filter(
          (j) =>
            !["server.logs", "server.files", "server.file.read"].includes(
              j.kind,
            ),
        ),
        owners: [{ id: aid, name: "Alex", kind: "account", wallet: { balance: 10000, reserved: 0 }, land: { chunks: 64 }, used_chunks: 4 }],
        claims: [{ id: "claim1", owner: aid, name: "Hill", state: "active" }],
        listings: [],
        assets: [
          {
            id: "asset1",
            name: "House",
            kind: "building",
            state: "escrowed",
            owner: aid,
          },
        ],
        backups: [],
      });
    }
    if (p === "/api/v1/server-presets")
      return json({
        minimum_storage_mib: state.hosting.minimum_server_storage_mib,
        hosting:state.hosting,
        presets: [{ software: "paper", version: "1.21.11", java: 21 }],
      });
    if (p === "/api/v1/view/social" && url.searchParams.get("section") === "teams")
      return json({teams:[team],contribution_team_id:teamId});
    if (p === "/api/v1/teams/" + teamId)
      return json({team,contribution_team_id:teamId,members_next_after:null});
    const notices = state.notices ?? [
      {
        id: 1,
        kind: "job_finished",
        created_at: "2026-10-03T08:00:00Z",
        body: {
          id: "completed",
          job_kind: "server.backup",
          server_id: sid,
          server_name: "Workshop",
        },
      },
      {
        id: 2,
        kind: "transfer",
        created_at: "2026-10-03T08:00:01Z",
        body: { amount: 40, target_name: "Alex", message: systemFixtureMessage("Coins received") },
      },
    ];
    return json({
      servers: state.worlds ?? [
        state.server,
        { ...state.server, id: otherSid, name: "Second server" },
      ],
      play:state.play,
      hosting:state.hosting,
      friends: [{ id: "bea", name: "Bea", state: "accepted" }],
      rooms: state.rooms,
      team: {
        id: "team1",
        name: "Builders team",
        leader: aid,
        room_id: groupId,
        members: [
          { account_id: aid, name: "Alex" },
          { account_id: "bea", name: "Bea" },
        ],
      },
      party: null,
      communities: [],
      links: [],
      blocks: [],
      reports: [],
      ranks: [],
      audit: [],
      counts: { notifications: 2, invitations: 0, jobs: 1 },
      invitations: [],
      notifications: notices,
      jobs: [state.jobs.get("completed")],
      next_cursor: null,
    });
  });
  return state;
}
