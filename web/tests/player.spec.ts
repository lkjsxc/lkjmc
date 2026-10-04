import fs from "node:fs";
import { test, expect, type BrowserContext, type Page } from "@playwright/test";
import { mountFixture, sid, otherSid, aid } from "./fixture.mjs";

const url = (path: string) => "https://ux.fixture/#" + path;
const catalogs = {
  en: JSON.parse(
    fs.readFileSync(new URL("../../locales/en.json", import.meta.url), "utf8"),
  ),
  ja: JSON.parse(
    fs.readFileSync(new URL("../../locales/ja.json", import.meta.url), "utf8"),
  ),
};
function japanese(english: string) {
  const id = Object.keys(catalogs.en).find(
    (key) => catalogs.en[key] === english,
  );
  if (!id || !catalogs.ja[id])
    throw new Error("Missing translation for " + english);
  return catalogs.ja[id] as string;
}
async function setup(context: BrowserContext, page: Page) {
  const state = await mountFixture(context);
  await page.clock.install({ time: new Date("2026-10-03T10:00:00Z") });
  return state;
}
async function tick(page: Page, count = 3) {
  for (let i = 0; i < count; i++) {
    await page.clock.fastForward(2500);
    await page.waitForTimeout(50);
  }
}
const action = (allowed: boolean, reason: string | null = null) => ({
  allowed,
  reason,
});
function serverStatus(overrides: Record<string, any> = {}) {
  return {
    machine_state: "stopped",
    game_state: "stopped",
    observation_fresh: true,
    joinable: false,
    activity: null,
    ...overrides,
    actions: {
      join: action(true),
      start: action(true),
      stop: action(false, "already_stopped"),
      logs: action(true),
      files: action(true),
      ...overrides.actions,
    },
  };
}

test("Play resumes the preferred world and follows projected identity, session and join permissions", async ({
  context,
  page,
}) => {
  const state = await setup(context, page);
  state.play = {
    preferred_server_id: otherSid,
    identity_ready: false,
    game_session: null,
  };
  await page.goto(url("/play"));
  await expect(page.locator(".resume-world strong")).toHaveText(
    "Second server",
  );
  await expect(
    page.getByRole("link", {
      name: "Link your Minecraft account",
      exact: true,
    }),
  ).toHaveAttribute("href", "#/account/linking");
  await expect(
    page.getByRole("button", {
      name: /Join world|Wake and join|Copy Minecraft address/,
    }),
  ).toHaveCount(0);
  expect(
    state.commands.filter((command: any) => command.type === "server_join"),
  ).toHaveLength(0);

  state.play.identity_ready = true;
  await page.reload();
  await expect(
    page.getByRole("button", { name: "Copy Minecraft address", exact: true }),
  ).toBeEnabled();
  await expect(
    page.getByRole("button", { name: /Join world|Wake and join/ }),
  ).toHaveCount(0);
  await expect(page.locator(".join-guidance")).toContainText(
    "Open Minecraft, connect to example.test:25591",
  );
  state.identities = [];
  state.play.game_session = { server_id: sid, client: "java" };
  await page.reload();
  await expect(
    page
      .locator(".play-hero")
      .getByRole("button", { name: "Wake and join", exact: true }),
  ).toBeEnabled();
  state.server.status = serverStatus({
    actions: { join: action(false, "permission_required") },
  });
  await page.reload();
  await expect(page.locator(".hero-actions button")).toBeDisabled();
  expect(
    state.commands.filter((command: any) => command.type === "server_join"),
  ).toHaveLength(0);
  state.server.observed = "running";
  state.server.players = 777;
  state.server.status = serverStatus({
    machine_state: "unknown",
    game_state: "unknown",
    observation_fresh: false,
    actions: {
      join: action(false, "observation_stale"),
      start: action(false, "observation_stale"),
      stop: action(false, "observation_stale"),
    },
  });
  await page.reload();
  await expect(page.locator(".resume-world .status")).toHaveText(
    "Checking status",
  );
  await expect(page.locator(".hero-actions button")).toBeDisabled();
  await page.goto(url("/worlds"));
  await expect(
    page.locator(".world-card").first().locator(".world-meta"),
  ).toContainText("— players online");
  await expect(page.locator(".world-grid")).not.toContainText("777");
  await page
    .locator(".world-card")
    .first()
    .getByRole("link", { name: "Open world", exact: true })
    .click();
  const playerCount = page
    .locator(".world-overview-grid .details-list > div")
    .filter({ has: page.getByText("Players online", { exact: true }) });
  await expect(playerCount.locator("dd")).toHaveText("—");
  await expect(page.locator("main")).not.toContainText("777");
});

test("offline Play copies the address without submitting a transfer", async ({
  context,
  page,
}) => {
  const state = await setup(context, page);
  state.server.name = "建築ワールド";
  state.play = {
    preferred_server_id: sid,
    identity_ready: true,
    game_session: null,
  };
  await page.addInitScript(() => {
    Object.defineProperty(navigator, "clipboard", {
      configurable: true,
      value: {
        writeText: async (value: string) => {
          (window as any).__copiedAddress = value;
        },
      },
    });
  });
  await page.goto(url("/play"));
  await expect(page.locator(".resume-world strong")).toHaveText("建築ワールド");
  await page
    .getByRole("button", { name: "Copy Minecraft address", exact: true })
    .click();
  await expect
    .poll(() => page.evaluate(() => (window as any).__copiedAddress))
    .toBe("example.test:25591");
  expect(
    state.commands.filter((command: any) => command.type === "server_join"),
  ).toHaveLength(0);
  await expect(page.getByRole("dialog")).toHaveCount(0);
});

test("online Play sends one preferred-world join and waits for the actual transfer receipt", async ({
  context,
  page,
}) => {
  const state = await setup(context, page);
  state.play = {
    preferred_server_id: otherSid,
    identity_ready: true,
    game_session: { server_id: sid, client: "java" },
  };
  state.joinResult = {
    effect: "committed",
    session_id: "session-42",
    server_id: otherSid,
    actual_server_id: otherSid,
  };
  state.pausedJobs = true;
  await page.goto(url("/play"));
  await page
    .locator(".play-hero")
    .getByRole("button", { name: "Wake and join", exact: true })
    .click();
  await expect(
    page
      .locator(".play-hero")
      .getByRole("button", { name: "Joining…", exact: true }),
  ).toBeDisabled();
  expect(
    state.commands.filter((command: any) => command.type === "server_join"),
  ).toEqual([{ type: "server_join", id: otherSid }]);
  await page
    .locator(".toast")
    .getByRole("button", { name: "View details", exact: true })
    .click();
  const dialog = page.getByRole("dialog");
  await tick(page);
  await expect(
    dialog.getByText("Transfer completed.", { exact: true }),
  ).toHaveCount(0);
  state.pausedJobs = false;
  await tick(page);
  await expect(
    dialog.getByText("Transfer completed.", { exact: true }),
  ).toBeVisible();
  await dialog.getByText("Full response", { exact: true }).click();
  await expect(dialog.locator(".job-response pre")).toContainText(
    `"actual_server_id": "${otherSid}"`,
  );
  await expect(
    dialog.getByRole("link", { name: "Open server", exact: true }),
  ).toHaveAttribute("href", `#/worlds/${otherSid}`);
  expect(
    state.commands.filter((command: any) => command.type === "server_join"),
  ).toHaveLength(1);
});

test("Worlds gives each world its own title and keeps expeditions in the Worlds navigation context", async ({
  context,
  page,
}) => {
  const state = await setup(context, page);
  state.server.kind = "official";
  state.server.name = "建築の森";
  await page.goto(url("/worlds"));
  const menu = page.getByRole("navigation", { name: "Main menu", exact: true });
  await expect(
    menu.getByRole("link", { name: "Worlds", exact: true }),
  ).toHaveAttribute("aria-current", "page");
  const world = page.locator(".world-card").filter({
    has: page.getByRole("heading", { name: "建築の森", exact: true }),
  });
  await world.getByRole("link", { name: "Open world", exact: true }).click();
  await expect(page).toHaveURL(url(`/worlds/${sid}`));
  await expect(page.locator("h1")).toHaveText("建築の森");
  await expect(page).toHaveTitle("建築の森 · lkjmc");
  const tabs = page.getByRole("navigation", { name: "Page menu", exact: true });
  await expect(tabs.getByRole("link")).toHaveText([
    "Overview",
    "World",
    "Economy",
  ]);
  await page.getByRole("link", { name: /^End expeditions / }).click();
  await expect(page.locator("h1")).toHaveText("Expeditions");
  await expect(
    menu.getByRole("link", { name: "Worlds", exact: true }),
  ).toHaveAttribute("aria-current", "page");
  await expect(page.locator("main")).not.toContainText("Private End");
});

test("expedition preparation reviews the server-owned price and lifetime", async ({
  context,
  page,
}) => {
  const state = await setup(context, page);
  state.expeditionCost = { coins: 2750, ender_eyes: 7 };
  state.durationSeconds = 5400;
  await page.goto(url("/expeditions"));
  await expect(page.locator(".expedition-requirements strong")).toHaveText([
    "2,750",
    "7",
    "1.5",
  ]);
  await page
    .getByRole("button", { name: "Prepare expedition", exact: true })
    .click();
  const dialog = page.getByRole("dialog");
  await expect(dialog.getByRole("heading")).toHaveText(
    "Prepare an End expedition",
  );
  await expect(dialog.locator(".modal-note")).toContainText(
    "Reserve 2,750 coins and 7 Eyes of Ender.",
  );
  await expect(dialog.locator(".modal-note")).toContainText(
    "The world closes 1.5 hours after opening.",
  );
  await page
    .getByRole("button", { name: "Reserve and prepare", exact: true })
    .click();
  await expect
    .poll(() =>
      state.commands.filter(
        (command: any) => command.type === "expedition_prepare",
      ),
    )
    .toEqual([{ type: "expedition_prepare" }]);
});

test("expedition preparation shows the roster and wallet while honoring server eligibility", async ({
  context,
  page,
}) => {
  const state = await setup(context, page);
  const ready = {
    account_id: aid,
    name: "Alex",
    ready: true,
    online: true,
    in_combat: false,
    occupied: false,
  };
  const friend = { ...ready, account_id: "bea", name: "勇者" };
  const cases = [
    {
      preparation: {
        is_leader: true,
        available_coins: 10000,
        can_prepare: false,
        participants: [ready, { ...friend, online: false }],
      },
      reason: "Must be online in SMP",
    },
    {
      preparation: {
        is_leader: false,
        available_coins: 10000,
        can_prepare: false,
        participants: [ready, friend],
      },
      reason: "Ask your party leader to prepare this Expedition.",
    },
    {
      preparation: {
        is_leader: true,
        available_coins: 499,
        can_prepare: false,
        participants: [ready, friend],
      },
      reason: "More coins needed",
    },
    {
      preparation: {
        is_leader: true,
        available_coins: 10000,
        can_prepare: false,
        participants: [ready, friend],
      },
      reason: "Unavailable",
    },
  ];
  for (const [index, entry] of cases.entries()) {
    state.preparation = entry.preparation;
    if (index === 0) await page.goto(url("/expeditions"));
    else await page.reload();
    await expect(page.locator(".preparation-roster li strong")).toHaveText([
      "Alex",
      "勇者",
    ]);
    await expect(page.locator(".expedition-preparation")).toContainText(
      `Available: ${entry.preparation.available_coins.toLocaleString("en-US")} coins`,
    );
    await expect(page.locator(".preparation-reason")).toContainText(
      entry.reason,
    );
    await expect(
      page.getByRole("button", { name: "Prepare expedition", exact: true }),
    ).toBeDisabled();
    await expect(page.getByRole("dialog")).toHaveCount(0);
  }
  expect(
    state.commands.filter(
      (command: any) => command.type === "expedition_prepare",
    ),
  ).toHaveLength(0);
  state.preparation = {
    is_leader: true,
    available_coins: 10000,
    can_prepare: true,
    participants: [ready, friend],
  };
  await page.reload();
  await expect(
    page.getByRole("button", { name: "Prepare expedition", exact: true }),
  ).toBeEnabled();
  await expect(page.locator(".preparation-reason")).toHaveCount(0);
});

test("expedition journal pages and scoped details revoke private data and reject late old-session responses", async ({
  context,
  page,
}) => {
  const state = await setup(context, page);
  const recent = {
    id: "00000000-0000-0000-0000-000000000042",
    state: "closed",
    participants: [{ name: "Recent explorers" }],
    created_at: "2026-10-03T09:00:00Z",
    can_enter: false,
    can_return: false,
    can_cancel: false,
  };
  const older = {
    id: "00000000-0000-0000-0000-000000000043",
    state: "active",
    participants: [{ name: "Private older roster" }],
    created_at: "2026-10-03T08:00:00Z",
    can_enter: true,
    can_return: false,
    can_cancel: false,
  };
  const cursor = "opaque/older:42";
  state.expeditionPages = {
    latest: { expeditions: [recent], next_cursor: cursor },
    [cursor]: { expeditions: [older], next_cursor: null },
  };
  await page.goto(url("/expeditions/journal"));
  await expect(page.locator("h1")).toHaveText("Expedition journal");
  await expect(page.locator(".expedition-entry")).toHaveCount(1);
  await expect(page.locator(".expedition-entry")).toContainText(
    "Recent explorers",
  );
  const pagination = page.getByRole("navigation", {
    name: "History pages",
    exact: true,
  });
  await pagination.getByRole("link", { name: "Older", exact: true }).click();
  await expect(page).toHaveURL(
    url("/expeditions/journal?cursor=" + encodeURIComponent(cursor)),
  );
  await expect(page.locator(".expedition-entry")).toContainText(
    "Private older roster",
  );
  await expect(page.locator("main")).not.toContainText("Recent explorers");
  await expect(
    pagination.getByRole("link", { name: "Older", exact: true }),
  ).toHaveCount(0);
  await expect(
    pagination.getByRole("link", { name: "Latest", exact: true }),
  ).toHaveAttribute("href", "#/expeditions/journal");
  await page
    .locator(".expedition-entry")
    .getByRole("link", { name: "View details", exact: true })
    .click();
  await expect(page).toHaveURL(url("/expeditions/" + older.id));
  await expect(page.locator(".expedition-entry")).toHaveCount(1);
  await expect(
    page.getByRole("button", { name: "Enter expedition", exact: true }),
  ).toBeVisible();
  const detailPath = "/api/v1/expeditions/" + older.id;
  state.failures[detailPath] = 403;
  await tick(page, 7);
  await expect(page.getByRole("alert")).toContainText(
    "You do not have permission to do this.",
  );
  await expect(page.locator(".expedition-entry")).toHaveCount(0);
  await expect(
    page.getByRole("button", { name: "Enter expedition", exact: true }),
  ).toHaveCount(0);
  await expect(page.locator("main")).not.toContainText("Private older roster");

  delete state.failures[detailPath];
  state.responseDelays[detailPath] = 700;
  const pending = page.waitForRequest(
    (request) => new URL(request.url()).pathname === detailPath,
  );
  await page.getByRole("button", { name: "Reload", exact: true }).click();
  await pending;
  state.failures["/api/v1/me"] = 401;
  await page.evaluate(() => window.dispatchEvent(new Event("focus")));
  await expect(
    page.getByRole("link", { name: "Join the community", exact: true }),
  ).toBeVisible();
  await page.waitForTimeout(900);
  await expect(page.locator(".expedition-entry")).toHaveCount(0);
  await expect(page.locator("body")).not.toContainText("Private older roster");
});

test("missing expedition requirements prevent preparation", async ({
  context,
  page,
}) => {
  const state = await setup(context, page);
  state.expeditionCost = null;
  state.durationSeconds = 10800;
  await page.goto(url("/expeditions"));
  await expect(
    page.getByText(
      "Expedition requirements are unavailable. Try again once the service is ready.",
      { exact: true },
    ),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Prepare expedition", exact: true }),
  ).toBeDisabled();
  await expect(page.getByRole("dialog")).toHaveCount(0);
  expect(
    state.commands.filter(
      (command: any) => command.type === "expedition_prepare",
    ),
  ).toHaveLength(0);
  state.expeditionCost = { coins: 1000, ender_eyes: 12 };
  state.durationSeconds = 0;
  await page.reload();
  await expect(
    page.getByRole("button", { name: "Prepare expedition", exact: true }),
  ).toBeDisabled();
});

test("expedition capabilities govern entry, return, cancellation and the historical journal", async ({
  context,
  page,
}) => {
  const state = await setup(context, page);
  state.expeditions = [
    {
      id: "preparing-42",
      state: "preparing",
      participants: [{ name: "準備組" }],
      created_at: "2026-10-03T09:00:00Z",
      can_cancel: true,
      can_enter: false,
      can_return: false,
    },
    {
      id: "active-42",
      state: "active",
      participants: [{ name: "冒険組" }],
      created_at: "2026-10-03T08:00:00Z",
      expires_at: "2026-10-03T11:30:00Z",
      can_cancel: false,
      can_enter: true,
      can_return: false,
    },
    {
      id: "return-42",
      state: "active",
      participants: [{ name: "帰還組" }],
      created_at: "2026-10-03T08:00:00Z",
      expires_at: "2026-10-03T11:30:00Z",
      can_cancel: false,
      can_enter: false,
      can_return: true,
    },
    {
      id: "closed-42",
      state: "closed",
      participants: [{ name: "歴史組" }],
      created_at: "2026-10-01T08:00:00Z",
      can_cancel: false,
      can_enter: false,
      can_return: false,
    },
    {
      id: "refunded-42",
      state: "refunded",
      participants: [{ name: "返却組" }],
      created_at: "2026-10-02T08:00:00Z",
      material_asset: "materials-42",
      can_receive: true,
    },
  ];
  await page.goto(url("/expeditions"));
  const current = page.locator("section.card").filter({
    has: page.getByRole("heading", {
      name: "Current expeditions",
      exact: true,
    }),
  });
  const journal = page.locator("section.card").filter({
    has: page.getByRole("heading", {
      name: "Expedition journal",
      exact: true,
    }),
  });
  await expect(current.locator(".expedition-entry")).toHaveCount(3);
  await expect(journal.locator(".expedition-entry")).toHaveCount(2);
  const active = current
    .locator(".expedition-entry")
    .filter({ hasText: "冒険組" });
  await expect(active).toContainText("90 minutes remaining");
  await expect(
    active.getByRole("button", { name: "Cancel preparation", exact: true }),
  ).toHaveCount(0);
  await expect(
    active.getByRole("button", { name: "Return to survival", exact: true }),
  ).toHaveCount(0);
  const returning = current
    .locator(".expedition-entry")
    .filter({ hasText: "帰還組" });
  await expect(
    returning.getByRole("button", { name: "Enter expedition", exact: true }),
  ).toHaveCount(0);
  await active
    .getByRole("button", { name: "Enter expedition", exact: true })
    .click();
  await returning
    .getByRole("button", { name: "Return to survival", exact: true })
    .click();
  await expect
    .poll(() =>
      state.commands.filter((command: any) =>
        command.type.startsWith("expedition_"),
      ),
    )
    .toEqual([
      { type: "expedition_enter", id: "active-42" },
      { type: "expedition_return", id: "return-42" },
    ]);
  const closed = journal
    .locator(".expedition-entry")
    .filter({ hasText: "歴史組" });
  await expect(closed.getByRole("button")).toHaveCount(0);
  await journal
    .getByRole("button", { name: "Collect returned items", exact: true })
    .click();
  await expect
    .poll(() =>
      state.commands.find((command: any) => command.type === "asset_receive"),
    )
    .toEqual({ type: "asset_receive", id: "materials-42" });
  await current
    .getByRole("button", { name: "Cancel preparation", exact: true })
    .click();
  const dialog = page.getByRole("dialog");
  await expect(dialog).toContainText(
    "Check the result until the refund completes.",
  );
  await dialog
    .getByRole("button", { name: "Cancel and refund", exact: true })
    .click();
  await expect
    .poll(() =>
      state.commands.find(
        (command: any) => command.type === "expedition_cancel",
      ),
    )
    .toEqual({ type: "expedition_cancel", id: "preparing-42" });
});

test("anonymous language switching updates all Landing copy", async ({
  context,
  page,
}) => {
  const state = await setup(context, page);
  state.failures["/api/v1/me"] = 401;
  await page.goto(url("/play"));
  const heading = "A world is better with people.";
  const lead =
    "Build a home. Find your people. Set off on an adventure. A Minecraft community that keeps it all connected.";
  await expect(page.locator("h1")).toHaveText(heading);
  await page
    .getByRole("combobox", { name: "Language", exact: true })
    .selectOption("ja");
  await expect(page.locator("html")).toHaveAttribute("lang", "ja");
  await expect(page.locator("h1")).toHaveText(japanese(heading));
  await expect(page.locator(".lead")).toHaveText(japanese(lead));
  await expect(
    page.getByRole("link", {
      name: japanese("Join the community"),
      exact: true,
    }),
  ).toBeVisible();
  await page
    .getByRole("combobox", { name: japanese("Language"), exact: true })
    .selectOption("en");
  await expect(page.locator("h1")).toHaveText(heading);
  await expect(page.locator(".lead")).toHaveText(lead);
});

test("a language change from another tab relocalizes an open dialog and keeps the field draft", async ({
  context,
  page,
}) => {
  await setup(context, page);
  await page.goto(url(`/hosting/servers/${sid}/files`));
  await tick(page);
  await page.locator(".upload-zone input[type=file]").setInputFiles({
    name: "資料.txt",
    mimeType: "text/plain",
    buffer: Buffer.from("player-authored file"),
  });
  const dialog = page.getByRole("dialog");
  await expect(dialog.getByRole("heading")).toHaveText(
    "Apply an uploaded file",
  );
  await dialog
    .getByLabel("Destination in server", { exact: true })
    .fill("資料.txt");
  const other = await context.newPage();
  await other.goto(url("/play"));
  await expect(other.locator("h1")).toHaveText("Play");
  await other.evaluate(() => localStorage.setItem("lkjmc.language", "ja"));
  await expect(page.locator("html")).toHaveAttribute("lang", "ja");
  await expect(dialog.getByRole("heading")).toHaveText(
    japanese("Apply an uploaded file"),
  );
  await expect(
    dialog.getByLabel(japanese("Destination in server"), { exact: true }),
  ).toHaveValue("資料.txt");
  await expect(dialog.locator(".modal-note")).toContainText(
    japanese(
      "The uploaded file is saved. Stop the server before applying it. World ZIPs are extracted into the specified folder.",
    ),
  );
  await expect(
    dialog.getByRole("button", { name: japanese("Apply file"), exact: true }),
  ).toBeVisible();
  await other.close();
});

test("Hosting shows independent host and Minecraft observations and blocks stale power changes", async ({
  context,
  page,
}) => {
  const state = await setup(context, page);
  state.server.observed = "running";
  state.server.status = serverStatus({
    machine_state: "running",
    game_state: "stopped",
  });
  await page.goto(url(`/hosting/servers/${sid}`));
  await expect(page.locator(".runtime-phases")).toContainText("Host is awake");
  await expect(
    page.getByRole("heading", { name: "Minecraft is stopped", exact: true }),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Start Minecraft", exact: true }),
  ).toBeEnabled();
  await expect(
    page.getByRole("button", { name: "Save and stop", exact: true }),
  ).toBeDisabled();
  await expect(page.locator(".resource-grid")).toContainText("2 GiB");
  await expect(page.locator(".resource-grid")).toContainText("2 vCPU");

  state.server.inspection = null;
  state.server.status = serverStatus({
    machine_state: "unknown",
    game_state: "running",
    joinable: true,
    actions: { start: action(false, "already_running"), stop: action(true) },
  });
  await page.reload();
  await expect(page.locator(".runtime-phases")).toContainText(
    "Host status unknown",
  );
  await expect(page.locator(".runtime-phases")).toContainText(
    "Ready for players",
  );
  await expect(
    page.getByRole("heading", { name: "Minecraft is running", exact: true }),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Start Minecraft", exact: true }),
  ).toBeDisabled();
  await expect(
    page.getByRole("button", { name: "Save and stop", exact: true }),
  ).toBeEnabled();

  state.server.status = serverStatus({
    machine_state: "unknown",
    game_state: "unknown",
    observation_fresh: false,
    joinable: false,
    actions: {
      start: action(false, "observation_stale"),
      stop: action(false, "observation_stale"),
    },
  });
  await page.reload();
  await expect(
    page.getByRole("heading", { name: "Checking your server", exact: true }),
  ).toBeVisible();
  await expect(page.locator(".runtime-phases")).not.toContainText(
    "Ready for players",
  );
  await expect(
    page.getByRole("button", { name: "Start Minecraft", exact: true }),
  ).toBeDisabled();
  await expect(
    page.getByRole("button", { name: "Save and stop", exact: true }),
  ).toBeDisabled();
  expect(
    state.commands.filter((command: any) =>
      ["server_start", "server_stop"].includes(command.type),
    ),
  ).toHaveLength(0);
});

test("Hosting exposes coarse operations without granting access to another actor's details", async ({
  context,
  page,
}) => {
  const state = await setup(context, page);
  state.administrator = false;
  state.server.inspection = null;
  state.server.status = serverStatus({
    actions: {
      start: action(false, "restore_in_progress"),
      stop: action(false, "restore_in_progress"),
    },
  });
  state.server.active_operation = {
    id: "restore-42",
    kind: "server.restore",
    state: "waiting",
    progress: { phase: "restoring" },
    can_inspect: false,
  };
  await page.goto(url(`/hosting/servers/${sid}`));
  const operation = page.getByRole("region", {
    name: "Current operation",
    exact: true,
  });
  await expect(operation.getByRole("heading")).toHaveText("Restore a backup");
  await expect(
    operation.getByRole("button", { name: "View details", exact: true }),
  ).toHaveCount(0);
  await expect(
    operation.getByRole("link", { name: "Open logs", exact: true }),
  ).toHaveAttribute("href", `#/hosting/servers/${sid}/logs`);
  expect(state.jobGets).not.toContain("restore-42");

  state.server.active_operation = null;
  state.server.operation_status = {
    id: "console-42",
    kind: "server.console",
    state: "failed",
    phase: "delivery_unknown",
    terminal: true,
    outcome: "delivery_unknown",
    automatic_retry: false,
    can_inspect: false,
  };
  await page.reload();
  await expect(operation).toContainText(
    "This command may have been delivered. Check the logs before sending a new command.",
  );
  await expect(
    operation.getByRole("button", { name: /Retry|Send command/ }),
  ).toHaveCount(0);
  expect(
    state.commands.filter((command: any) => command.type === "server_console"),
  ).toHaveLength(0);
});
