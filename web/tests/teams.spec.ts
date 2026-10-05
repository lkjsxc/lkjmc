import fs from "node:fs";
import { test, expect, type BrowserContext, type Page } from "@playwright/test";
import { mountFixture, aid, groupId, sid } from "./fixture.mjs";

const buildersId = "00000000-0000-0000-0000-000000000006";
const explorersId = "00000000-0000-0000-0000-000000000007";
const beaId = "00000000-0000-0000-0000-000000000008";
const caraId = "00000000-0000-0000-0000-000000000010";
const url = (path: string) => "https://ux.fixture/#" + path;
const catalogs = Object.fromEntries(
  ["en", "ja"].map((language) => [
    language,
    JSON.parse(
      fs.readFileSync(
        new URL(`../../locales/${language}.json`, import.meta.url),
        "utf8",
      ),
    ),
  ]),
);
const label = (id: string, language = "en", ...params: string[]) =>
  catalogs[language][id].replace(
    /\{(\d+)\}/g,
    (_: string, index: string) => params[Number(index)],
  );
const fullPermissions = {
  can_build: true,
  can_sell: true,
  can_spend: true,
  can_manage_members: true,
  can_administer: true,
};

async function setup(context: BrowserContext, page: Page, language = "en") {
  const fixture = await mountFixture(context, { language });
  await page.clock.install({ time: new Date("2026-10-05T10:00:00Z") });
  const state = {
    contribution: buildersId as string | null,
    commands: [] as Record<string, any>[],
    unavailable: new Set<string>(),
    pageSize: 100,
    economy: null as Record<string, any> | null,
    teams: [
      {
        id: buildersId,
        name: "Builders team",
        leader: aid,
        room_id: groupId,
        permissions: { ...fullPermissions },
        members: [
          { account_id: aid, name: "Alex", ...fullPermissions },
          { account_id: beaId, name: "Bea", can_build: true },
        ],
      },
      {
        id: explorersId,
        name: "Explorers team",
        leader: beaId,
        room_id: groupId,
        permissions: {
          can_build: true,
          can_sell: false,
          can_spend: false,
          can_manage_members: false,
          can_administer: false,
        },
        members: [
          { account_id: aid, name: "Alex", can_build: true },
          { account_id: beaId, name: "Bea", ...fullPermissions },
        ],
      },
    ] as Record<string, any>[],
  };
  const summary = (team: Record<string, any>) => ({
    id: team.id,
    name: team.name,
    leader: team.leader,
    room_id: team.room_id,
    permissions: team.permissions,
    member_count: team.members.length,
    is_contribution_team: state.contribution === team.id,
  });
  await context.route(/\/api\/v1\/view\/social(?:\?|$)/, (route) =>
    route.fulfill({
      json: {
        teams: state.teams
          .filter((team) => !state.unavailable.has(team.id))
          .map(summary),
        contribution_team_id: state.contribution,
      },
    }),
  );
  await context.route("**/api/v1/teams/**", (route) => {
    const request = new URL(route.request().url());
    const team = state.teams.find((item) =>
      request.pathname.endsWith("/" + item.id),
    );
    if (!team || state.unavailable.has(team.id))
      return route.fulfill({
        status: 403,
        json: { error: { message: { id: "error.forbidden", params: {} } } },
      });
    const cursor = request.searchParams.get("after");
    const index = cursor
      ? team.members.findIndex((member: any) => member.account_id === cursor) +
        1
      : 0;
    const members = team.members.slice(index, index + state.pageSize);
    const next =
      index + members.length < team.members.length
        ? members.at(-1).account_id
        : null;
    return route.fulfill({
      json: {
        team: { ...summary(team), members, members_next_after: next },
        contribution_team_id: state.contribution,
      },
    });
  });
  await context.route("**/api/v1/commands", (route) => {
    const command = route.request().postDataJSON().command;
    state.commands.push(command);
    if (!command.type.startsWith("team_")) return route.fallback();
    if (command.type === "team_contribution_set")
      state.contribution = command.team;
    if (command.type === "team_leave") {
      state.unavailable.add(command.team);
      if (state.contribution === command.team) state.contribution = null;
    }
    return route.fulfill({ json: { result: { updated: true } } });
  });
  await context.route("**/api/v1/servers/" + sid + "?*", (route) =>
    state.economy
      ? route.fulfill({ json: { server: fixture.server, ...state.economy } })
      : route.fallback(),
  );
  return state;
}

async function poll(page: Page) {
  await page.clock.fastForward(15100);
  await page.waitForTimeout(50);
}

test("multiple teams keep distinct permissions and explicitly route contributions and leaving", async ({
  context,
  page,
}) => {
  const state = await setup(context, page);
  await page.goto(url("/people/teams"));
  await expect(page.locator(".team-list-item")).toHaveCount(2);
  await page.getByRole("button", { name: "Create team", exact: true }).click();
  await expect(page.getByRole("dialog")).toContainText(
    label("team.creation_note"),
  );
  await page
    .getByRole("dialog")
    .getByRole("button", { name: "Close", exact: true })
    .click();
  await page
    .locator(".team-name")
    .filter({ hasText: "Explorers team" })
    .click();
  await expect(page.locator(".team-permissions")).toHaveText("Build");
  await page
    .getByRole("button", {
      name: label("team.contribution_choose"),
      exact: true,
    })
    .click();
  await expect(page.getByRole("dialog")).toContainText("Explorers team");
  await page
    .getByRole("dialog")
    .getByRole("button", {
      name: label("team.contribution_choose"),
      exact: true,
    })
    .click();
  await expect.poll(() => state.contribution).toBe(explorersId);
  expect(state.commands.at(-1)).toEqual({
    type: "team_contribution_set",
    team: explorersId,
  });
  await page.goto(url("/people/teams/" + buildersId));
  await expect(page.locator(".team-permissions")).toContainText(
    "Spend shared coins",
  );
  await expect(
    page.getByRole("button", {
      name: label("team.contribution_choose"),
      exact: true,
    }),
  ).toBeVisible();
  await page.goto(url("/people/teams/" + explorersId + "/settings"));
  await page.getByRole("button", { name: "Leave team", exact: true }).click();
  await expect(page.getByRole("dialog")).toContainText("Explorers team");
  await page
    .getByRole("dialog")
    .getByRole("button", { name: "Leave team", exact: true })
    .click();
  await expect(page).toHaveURL(url("/people/teams"));
  await expect(page.locator(".team-list-item")).toHaveCount(1);
  await expect(page.locator(".team-name")).toHaveText("Builders team");
  await expect(page.locator(".team-contribution")).toContainText(
    label("team.contribution_none"),
  );
  expect(state.commands.at(-1)).toEqual({
    type: "team_leave",
    team: explorersId,
  });
});

test("a team administrator can edit a scoped member role but cannot assign administrators", async ({
  context,
  page,
}) => {
  const state = await setup(context, page);
  state.teams[0].leader = beaId;
  state.teams[0].members.push({
    account_id: caraId,
    name: "Cara",
    can_build: true,
    can_sell: false,
    can_spend: false,
    can_manage_members: false,
    can_administer: false,
  });
  await page.goto(url("/people/teams/" + buildersId + "/members"));
  const cara = page
    .locator(".team-member-row")
    .filter({ has: page.getByText("Cara", { exact: true }) });
  await cara.getByRole("button", { name: "Role", exact: true }).click();
  const dialog = page.getByRole("dialog");
  await expect(dialog).toContainText("Builders team");
  await expect(
    dialog.getByRole("checkbox", { name: "Manage team", exact: true }),
  ).toHaveCount(0);
  await dialog
    .getByRole("checkbox", { name: "Spend shared coins", exact: true })
    .check();
  await dialog.getByRole("button", { name: "Save", exact: true }).click();
  await expect.poll(() => state.commands.length).toBe(1);
  expect(state.commands[0]).toEqual({
    type: "team_permissions",
    team: buildersId,
    member: caraId,
    administer: false,
    build: true,
    sell: false,
    spend: true,
    members: false,
  });
  await cara.getByRole("button", { name: "Role", exact: true }).click();
  state.teams[0].permissions = { ...fullPermissions, can_administer: false };
  await poll(page);
  await expect(page.getByRole("dialog")).toHaveCount(0);
  await expect(
    cara.getByRole("button", { name: "Role", exact: true }),
  ).toHaveCount(0);
});

test("team confirmations close on route changes and membership loss removes private detail", async ({
  context,
  page,
}) => {
  const state = await setup(context, page);
  await page.goto(url("/people/teams/" + explorersId + "/settings"));
  await page.getByRole("button", { name: "Leave team", exact: true }).click();
  await page.evaluate(
    (path) => {
      location.hash = path;
    },
    "/people/teams/" + buildersId + "/settings",
  );
  await expect(page.getByRole("dialog")).toHaveCount(0);
  expect(state.commands).toHaveLength(0);
  await page.goto(url("/people/teams/" + explorersId + "/settings"));
  await page.getByRole("button", { name: "Leave team", exact: true }).click();
  state.unavailable.add(explorersId);
  await poll(page);
  await expect(page.getByRole("dialog")).toHaveCount(0);
  await expect(page.locator(".teams-workspace")).toHaveCount(0);
  await expect(
    page.getByRole("button", { name: "Leave team", exact: true }),
  ).toHaveCount(0);
  expect(state.commands).toHaveLength(0);
});

test("members can load later pages and expanded data is isolated to its team", async ({
  context,
  page,
}) => {
  const state = await setup(context, page);
  state.pageSize = 1;
  await page.goto(url("/people/teams/" + buildersId + "/members"));
  await expect(page.locator(".team-member-row")).toHaveCount(1);
  await page
    .getByRole("button", { name: label("team.load_members"), exact: true })
    .click();
  await expect(page.locator(".team-member-row")).toHaveCount(2);
  await expect(
    page.getByRole("button", { name: label("team.load_members"), exact: true }),
  ).toHaveCount(0);
  state.teams[0].members.push({ account_id: caraId, name: "Cara" });
  await poll(page);
  await expect(page.locator(".team-member-row")).toHaveCount(1);
  await expect(
    page.getByRole("button", { name: label("team.load_members"), exact: true }),
  ).toBeVisible();
  await page.goto(url("/people/teams/" + explorersId + "/members"));
  await expect(page.locator(".team-member-row")).toHaveCount(1);
  await expect(
    page.getByRole("button", { name: "Role", exact: true }),
  ).toHaveCount(0);
  await expect(
    page.getByRole("button", { name: "Invite member", exact: true }),
  ).toHaveCount(0);
});

test("contribution labels and confirmations remain fully Japanese while preserving team names", async ({
  context,
  page,
}) => {
  const state = await setup(context, page, "ja");
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(url("/people/teams/" + explorersId));
  await expect(page.locator(".team-contribution")).toContainText(
    label("team.contribution_description", "ja"),
  );
  await page
    .getByRole("button", {
      name: label("team.contribution_choose", "ja"),
      exact: true,
    })
    .click();
  await expect(page.getByRole("dialog")).toContainText(
    label("team.contribution_choose_note", "ja", "Explorers team"),
  );
  await page
    .getByRole("dialog")
    .getByRole("button", {
      name: label("team.contribution_choose", "ja"),
      exact: true,
    })
    .click();
  await expect.poll(() => state.contribution).toBe(explorersId);
  await page
    .getByRole("button", {
      name: label("team.contribution_clear", "ja"),
      exact: true,
    })
    .click();
  await page
    .getByRole("dialog")
    .getByRole("button", {
      name: label("team.contribution_clear", "ja"),
      exact: true,
    })
    .click();
  await expect.poll(() => state.contribution).toBe(null);
  await expect(page.locator("main")).not.toContainText(
    /Automatic team|No team selected|message-[a-f0-9]/,
  );
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= innerWidth,
    ),
  ).toBe(true);
});

function economicOwners() {
  return [
    {
      id: aid,
      name: "Alex",
      kind: "account",
      permissions: { ...fullPermissions },
    },
    {
      id: buildersId,
      name: "Builders team",
      kind: "team",
      permissions: { can_build: true, can_sell: false, can_spend: false },
    },
    {
      id: explorersId,
      name: "Explorers team",
      kind: "team",
      permissions: { can_build: false, can_sell: true, can_spend: true },
    },
  ].map((owner) => ({
    ...owner,
    wallet: { balance: 1000, reserved: 0 },
    land: { chunks: 16 },
    used_chunks: 1,
  }));
}

test("shared wallets and land offer only each team's authorized mutations", async ({
  context,
  page,
}) => {
  const state = await setup(context, page);
  state.economy = {
    owners: economicOwners(),
    claims: [
      {
        id: "personal-claim",
        owner: aid,
        name: "Personal field",
        state: "active",
        chunks: 1,
      },
      {
        id: "builders-claim",
        owner: buildersId,
        name: "Builders field",
        state: "active",
        chunks: 1,
      },
      {
        id: "explorers-claim",
        owner: explorersId,
        name: "Explorers field",
        state: "active",
        chunks: 1,
      },
    ],
  };
  await page.goto(url("/worlds/" + sid + "/economy?tab=wallet"));
  const builders = page
    .locator(".balance-card")
    .filter({ hasText: "Builders team" });
  const explorers = page
    .locator(".balance-card")
    .filter({ hasText: "Explorers team" });
  await expect(builders).toContainText("1,000");
  await expect(
    builders.getByRole("button", { name: "Send coins now", exact: true }),
  ).toHaveCount(0);
  await explorers
    .getByRole("button", { name: "Send coins now", exact: true })
    .click();
  await expect(page.getByRole("dialog")).toContainText(
    label("economy.send_coins_title", "en", "Explorers team"),
  );
  state.economy.owners[2].permissions.can_spend = false;
  await poll(page);
  await expect(page.getByRole("dialog")).toHaveCount(0);
  await page.goto(url("/worlds/" + sid + "/world?tab=land"));
  await page
    .getByRole("button", { name: "Protect land now", exact: true })
    .click();
  const owners = page
    .getByRole("dialog")
    .getByRole("combobox", { name: "Owner", exact: true });
  await expect(owners.locator("option")).toHaveText(["Alex", "Builders team"]);
  await expect(owners).toHaveValue(aid);
  await page
    .getByRole("dialog")
    .getByRole("button", { name: "Close", exact: true })
    .click();
  const buildersLand = page
    .locator(".list-row")
    .filter({ has: page.getByText("Builders field", { exact: true }) });
  const explorersLand = page
    .locator(".list-row")
    .filter({ has: page.getByText("Explorers field", { exact: true }) });
  await expect(
    buildersLand.getByRole("button", { name: "Remove", exact: true }),
  ).toHaveCount(0);
  await explorersLand
    .getByRole("button", { name: "Remove", exact: true })
    .click();
  await expect(page.getByRole("dialog")).toContainText(
    label("economy.owner_note", "en", "Explorers team"),
  );
});

test("market owner choices distinguish selling, building, and spending without using contribution selection", async ({
  context,
  page,
}) => {
  const state = await setup(context, page);
  state.contribution = explorersId;
  state.economy = {
    owners: economicOwners(),
    claims: [
      {
        id: "personal-claim",
        owner: aid,
        name: "Personal field",
        state: "active",
      },
      {
        id: "builders-claim",
        owner: buildersId,
        name: "Builders field",
        state: "active",
      },
      {
        id: "explorers-claim",
        owner: explorersId,
        name: "Explorers field",
        state: "active",
      },
    ],
    assets: [
      {
        id: "builders-building",
        owner: buildersId,
        title: "Builders house",
        kind: "building",
        state: "escrowed",
      },
      {
        id: "builders-items",
        owner: buildersId,
        title: "Builders items",
        kind: "items",
        state: "escrowed",
      },
      {
        id: "explorers-items",
        owner: explorersId,
        title: "Explorer items",
        kind: "items",
        state: "escrowed",
      },
    ],
    listings: [
      {
        id: "land-listing",
        seller: caraId,
        seller_name: "Cara",
        title: "Cara field",
        kind: "land",
        price: 100,
      },
      {
        id: "item-listing",
        seller: caraId,
        seller_name: "Cara",
        title: "Cara items",
        kind: "items",
        price: 100,
      },
    ],
  };
  await page.goto(url("/worlds/" + sid + "/economy?tab=storage"));
  const house = page
    .locator(".list-row")
    .filter({ has: page.getByText("Builders house", { exact: true }) });
  await expect(
    house.getByRole("button", { name: "Place", exact: true }),
  ).toBeVisible();
  await expect(
    house.getByRole("button", { name: "List for sale", exact: true }),
  ).toHaveCount(0);
  const builderItems = page
    .locator(".list-row")
    .filter({ has: page.getByText("Builders items", { exact: true }) });
  await expect(
    builderItems.getByRole("button", {
      name: label("text.collect_in_game"),
      exact: true,
    }),
  ).toHaveCount(0);
  const explorerItems = page
    .locator(".list-row")
    .filter({ has: page.getByText("Explorer items", { exact: true }) });
  await expect(
    explorerItems.getByRole("button", {
      name: label("text.collect_in_game"),
      exact: true,
    }),
  ).toBeVisible();
  await expect(
    explorerItems.getByRole("button", { name: "List for sale", exact: true }),
  ).toBeVisible();
  await page.goto(url("/worlds/" + sid + "/economy?tab=market"));
  await page
    .locator(".market-card")
    .filter({ hasText: "Cara field" })
    .getByRole("button", { name: "Buy", exact: true })
    .click();
  await expect(
    page
      .getByRole("dialog")
      .getByRole("combobox", { name: "Owner", exact: true })
      .locator("option"),
  ).toHaveText(["Alex"]);
  await page
    .getByRole("dialog")
    .getByRole("button", { name: "Close", exact: true })
    .click();
  state.economy.owners[2].permissions.can_build = true;
  await poll(page);
  await page
    .locator(".market-card")
    .filter({ hasText: "Cara field" })
    .getByRole("button", { name: "Buy", exact: true })
    .click();
  await expect(
    page
      .getByRole("dialog")
      .getByRole("combobox", { name: "Owner", exact: true })
      .locator("option"),
  ).toHaveText(["Alex", "Explorers team"]);
  state.economy.owners[2].permissions.can_build = false;
  await poll(page);
  await expect(page.getByRole("dialog")).toHaveCount(0);
  await page
    .locator(".market-card")
    .filter({ hasText: "Cara items" })
    .getByRole("button", { name: "Buy", exact: true })
    .click();
  const buyer = page
    .getByRole("dialog")
    .getByRole("combobox", { name: "Owner", exact: true });
  await expect(buyer.locator("option")).toHaveText(["Alex", "Explorers team"]);
  await expect(buyer).toHaveValue(aid);
  await page
    .getByRole("dialog")
    .getByRole("button", { name: "Close", exact: true })
    .click();
  await page
    .getByRole("button", { name: "Prepare a listing", exact: true })
    .click();
  await expect(
    page
      .getByRole("dialog")
      .getByRole("combobox", { name: "Owner", exact: true })
      .locator("option"),
  ).toHaveText(["Alex", "Explorers team"]);
  await expect(
    page
      .getByRole("dialog")
      .getByRole("combobox", { name: "Land", exact: true })
      .locator("option"),
  ).toHaveText(["Alex · Personal field", "Explorers team · Explorers field"]);
  await page
    .getByRole("dialog")
    .getByRole("combobox", { name: "Owner", exact: true })
    .selectOption(explorersId);
  await page
    .getByRole("dialog")
    .getByRole("combobox", { name: "Type", exact: true })
    .selectOption("building");
  await page
    .getByRole("dialog")
    .getByRole("textbox", { name: "Name", exact: true })
    .fill("Mismatch proof");
  await page
    .getByRole("dialog")
    .getByRole("combobox", { name: "Land", exact: true })
    .selectOption("personal-claim");
  await page
    .getByRole("dialog")
    .getByRole("button", { name: "Start deposit", exact: true })
    .click();
  await expect(page.getByRole("dialog").getByRole("alert")).toContainText(
    label("economy.claim_owner_mismatch"),
  );
  expect(
    state.commands.filter((command) => command.type === "asset_capture"),
  ).toHaveLength(0);
});
