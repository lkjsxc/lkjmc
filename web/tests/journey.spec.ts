import { test, expect } from "@playwright/test";
import { readFileSync } from "node:fs";
let session: { token: string };
test.beforeAll(() => {
  const file =
    process.env.LKJMC_TEST_SESSION ?? "../.local/browser-session.json";
  try {
    session = JSON.parse(readFileSync(file, "utf8"));
    if (!session.token) throw new Error("Missing token");
  } catch {
    throw new Error(
      "Integration requires a dedicated loopback service and test session at " +
        file +
        ". This lane never skips or falls back to mocks.",
    );
  }
});
test.beforeEach(async ({ context, baseURL }) => {
  const origin = baseURL ?? "http://127.0.0.1:18091";
  const target = new URL(origin);
  if (target.hostname !== "127.0.0.1")
    throw new Error(
      "Integration writes are restricted to the dedicated loopback test service.",
    );
  await context.addCookies([
    {
      name: "lkjmc_session",
      value: session.token,
      url: origin,
      httpOnly: true,
      sameSite: "Lax",
    },
  ]);
  const response = await context.request.get(origin + "/api/v1/me");
  expect(
    response.ok(),
    "Dedicated test session must authenticate",
  ).toBeTruthy();
  const me = await response.json();
  const saved = await context.request.post(origin + "/api/v1/commands", {
    headers: { "x-csrf-token": me.csrf, origin },
    data: {
      request_id: crypto.randomUUID(),
      command: { type: "language", language: "en" },
    },
  });
  expect(saved.ok()).toBeTruthy();
});
test("player navigation and official world tools load real authorized projections", async ({
  page,
}) => {
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  await page.goto("/#/play");
  await expect(page.locator("h1")).toHaveText("Play");
  for (const [path, title] of [
    ["/worlds", "Worlds"],
    ["/people", "Friends"],
    ["/timeline", "Timeline"],
    ["/expeditions", "Expeditions"],
    ["/account", "Account"],
  ]) {
    await page.goto("/#" + path);
    await expect(page.locator("h1")).toHaveText(title);
    await expect(page.getByText("Loading…", { exact: true })).toHaveCount(0);
    await expect(page.getByRole("alert")).toHaveCount(0);
  }
  const worlds = await (await page.request.get("/api/v1/view/play")).json();
  const official = worlds.servers.find(
    (world: any) => world.kind === "official",
  );
  expect(
    official,
    "The dedicated service needs an official world fixture",
  ).toBeTruthy();
  await page.goto("/#/worlds/" + official.id);
  await expect(page.locator("h1")).toHaveText(official.name);
  await expect(
    page
      .getByRole("navigation", { name: "Page menu" })
      .getByRole("link", { name: "World", exact: true }),
  ).toBeVisible();
  for (const route of [
    "/world?tab=land",
    "/world?tab=homes",
    "/economy?tab=wallet",
    "/economy?tab=storage",
  ]) {
    await page.goto("/#/worlds/" + official.id + route);
    await expect(page.locator("h1")).toBeVisible();
    await expect(page.getByText("Loading…", { exact: true })).toHaveCount(0);
    await expect(page.getByRole("alert")).toHaveCount(0);
    await page.reload();
    await expect(page.locator("h1")).toBeVisible();
  }
  await page.goto("/#/play");
  await page.screenshot({
    path: "../.local/player-real-desktop.png",
    fullPage: true,
  });
  expect(errors).toEqual([]);
});
test("an authorized conversation, message and report persist through reload", async ({
  page,
}) => {
  const name = `検証グループ ${Date.now()}`;
  const body = "日本語の利用者メッセージは英語の画面でもそのまま表示されます。";
  const reason = `Browser acceptance ${name}`;
  await page.goto("/#/timeline");
  await expect(page.locator(".timeline-composer")).toHaveCount(0);
  await page
    .getByRole("button", { name: "Create group chat", exact: true })
    .click();
  await page.getByLabel("Group name", { exact: true }).fill(name);
  await page
    .getByRole("dialog")
    .getByRole("button", { name: "Create", exact: true })
    .click();
  await expect(page.getByRole("dialog")).toHaveCount(0);
  await expect(
    page.locator(".conversation-link[aria-current=page]"),
  ).toContainText(name);
  await expect(page.locator("#timeline-pane-title")).toHaveText(name);
  await page.getByLabel("Message", { exact: true }).fill(body);
  await page.getByRole("button", { name: "Send", exact: true }).click();
  await expect(
    page.locator("article.message").getByText(body, { exact: true }),
  ).toBeVisible();
  await page.reload();
  await expect(page.locator("#timeline-pane-title")).toHaveText(name);
  await expect(
    page.locator("article.message").getByText(body, { exact: true }),
  ).toBeVisible();
  await page.locator(".timeline-toolbar details summary").click();
  await page
    .getByRole("button", { name: "Report messages", exact: true })
    .click();
  await page.getByRole("checkbox").first().check();
  await page
    .getByRole("button", { name: "Review submission", exact: true })
    .click();
  await expect(
    page.getByRole("dialog").getByText(body, { exact: true }),
  ).toBeVisible();
  await page.getByLabel("Reason for report", { exact: true }).fill(reason);
  await page
    .getByRole("dialog")
    .getByRole("button", { name: "Submit this report", exact: true })
    .click();
  await expect(page.getByRole("dialog")).toHaveCount(0);
  await page.goto("/#/account/reports");
  await expect(page.getByText(reason, { exact: true })).toBeVisible();
});
test("mobile world navigation and an authorized land form fit a narrow viewport", async ({
  page,
}) => {
  await page.setViewportSize({ width: 320, height: 844 });
  await page.goto("/#/play");
  await expect(page.locator("h1")).toHaveText("Play");
  await page
    .getByRole("navigation", { name: "Quick navigation" })
    .getByRole("link", { name: "Worlds", exact: true })
    .click();
  await expect(page.locator("h1")).toHaveText("Worlds");
  const response = await page.request.get("/api/v1/view/play");
  const data = await response.json();
  const official = data.servers.find((world: any) => world.kind === "official");
  expect(official).toBeTruthy();
  await page.goto("/#/worlds/" + official.id + "/world");
  await page
    .getByRole("button", { name: "Protect land now", exact: true })
    .click();
  await expect(page.getByRole("dialog")).toBeVisible();
  const size = await page.evaluate(() => ({
    scroll: document.documentElement.scrollWidth,
    client: document.documentElement.clientWidth,
  }));
  expect(size.scroll).toBeLessThanOrEqual(size.client);
  await page.screenshot({
    path: "../.local/player-real-land-mobile.png",
    fullPage: true,
  });
});

test("multiple teams keep explicit context, contribution selection and grouped progress", async ({
  page,
  baseURL,
}) => {
  test.setTimeout(90_000);
  const origin = baseURL!;
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  async function projection(path: string) {
    const response = await page.request.get(path);
    expect(response.ok(), "Real Core projection: " + path).toBeTruthy();
    return response.json();
  }
  const me = await projection("/api/v1/me");
  const initial = await projection("/api/v1/view/social?section=teams");
  expect(Array.isArray(initial.teams)).toBeTruthy();
  const initialIds = initial.teams.map((team: any) => team.id).sort();
  const originalContribution = initial.contribution_team_id ?? null;
  const prefix = "Browser teams " + crypto.randomUUID().slice(0, 8);
  const created: { id: string; name: string; room: string }[] = [];

  async function createTeam(name: string) {
    await page.goto("/#/people/teams");
    await expect(page.locator("h1")).toHaveText("Teams");
    await page
      .getByRole("button", { name: "Create team", exact: true })
      .click();
    await page.getByLabel("Team name", { exact: true }).fill(name);
    const receipt = page.waitForResponse(
      (response) =>
        response.url() === origin + "/api/v1/commands" &&
        response.request().method() === "POST" &&
        response.request().postDataJSON().command?.type === "team_create",
    );
    await page
      .getByRole("dialog")
      .getByRole("button", { name: "Create team", exact: true })
      .click();
    const response = await receipt;
    expect(response.ok()).toBeTruthy();
    const { result } = await response.json();
    const team = { id: result.team_id, name, room: result.room_id };
    // Record the accepted effect before UI assertions so a failure still cleans it up.
    created.push(team);
    await expect(page).toHaveURL(origin + "/#/people/teams/" + team.id);
    await expect(page.locator("h1")).toHaveText(name);
    await expect(page.getByRole("dialog")).toHaveCount(0);
    return team;
  }
  async function chooseTeam(team: (typeof created)[number]) {
    await page.goto("/#/people/teams/" + team.id);
    await expect(page.locator("h1")).toHaveText(team.name);
    await page
      .locator(".team-contribution")
      .getByRole("button", { name: "Select for contributions", exact: true })
      .click();
    await expect(
      page.getByRole("dialog").getByRole("heading", {
        name: "Contribute to " + team.name + "?",
        exact: true,
      }),
    ).toBeVisible();
    await page
      .getByRole("dialog")
      .getByRole("button", { name: "Select for contributions", exact: true })
      .click();
    await expect(page.getByRole("dialog")).toHaveCount(0);
    await page.reload();
    await expect(page.locator("h1")).toHaveText(team.name);
    await expect(
      page
        .locator(".team-contribution")
        .getByText("Selected for contributions", {
          exact: true,
        }),
    ).toBeVisible();
    const detail = await projection("/api/v1/teams/" + team.id);
    expect(detail.contribution_team_id).toBe(team.id);
  }

  try {
    const first = await createTeam(prefix + " A");
    const afterFirst = await projection("/api/v1/view/social?section=teams");
    expect(afterFirst.contribution_team_id).toBe(
      initialIds.length ? originalContribution : first.id,
    );
    const second = await createTeam(prefix + " B");
    const collection = await projection("/api/v1/view/social?section=teams");
    expect(collection.teams.map((team: any) => team.id).sort()).toEqual(
      [...initialIds, first.id, second.id].sort(),
    );
    // An additional membership must not silently redirect future contributions.
    expect(collection.contribution_team_id).toBe(
      afterFirst.contribution_team_id,
    );

    const land = await projection("/api/v1/view/life?section=land");
    for (const team of created) {
      const owner = land.owners.find((owner: any) => owner.id === team.id);
      expect(owner, "The new team's own land allowance exists").toBeTruthy();
      expect(owner.land.chunks).toBe(0);
      const detail = await projection("/api/v1/teams/" + team.id);
      expect(detail.team.id).toBe(team.id);
      expect(detail.team.name).toBe(team.name);
      expect(detail.team.room_id).toBe(team.room);
      expect(detail.team.leader).toBe(me.account.id);
      expect(detail.team.member_count).toBe(1);
      expect(detail.team.permissions.can_administer).toBe(true);
      await page.goto("/#/people/teams/" + team.id);
      await expect(page.locator("h1")).toHaveText(team.name);
      await expect(
        page.getByRole("link", { name: "Team chat", exact: true }),
      ).toHaveAttribute("href", "#/timeline?room=" + team.room);
    }
    if (collection.contribution_team_id !== first.id) await chooseTeam(first);
    await chooseTeam(second);
    // Selection B must not make explicit route A show B's private team context.
    await page.goto("/#/people/teams/" + first.id);
    await expect(page.locator("h1")).toHaveText(first.name);
    await expect(
      page.getByRole("link", { name: "Team chat", exact: true }),
    ).toHaveAttribute("href", "#/timeline?room=" + first.room);
    await expect(
      page.locator(".team-contribution").getByRole("button", {
        name: "Select for contributions",
        exact: true,
      }),
    ).toBeVisible();

    await page.goto("/#/people/teams");
    await expect(
      page.locator(".team-contribution").getByRole("link"),
    ).toHaveText(second.name);
    await page
      .getByRole("button", { name: "Clear selection", exact: true })
      .click();
    await page
      .getByRole("dialog")
      .getByRole("button", { name: "Clear selection", exact: true })
      .click();
    await expect(page.getByRole("dialog")).toHaveCount(0);
    await page.reload();
    await expect(page.locator(".team-contribution")).toContainText(
      "No team selected",
    );
    const cleared = await projection("/api/v1/view/social?section=teams");
    expect(cleared.contribution_team_id).toBeNull();
    expect(cleared.teams.map((team: any) => team.id).sort()).toEqual(
      collection.teams.map((team: any) => team.id).sort(),
    );

    const play = await projection("/api/v1/view/play");
    const official = play.servers.find(
      (world: any) => world.kind === "official",
    );
    expect(
      official,
      "Official world fixture for achievement navigation",
    ).toBeTruthy();
    const progress = await projection("/api/v1/view/life?section=achievements");
    const personal = progress.achievements.find(
      (group: any) => group.owner.id === me.account.id,
    );
    expect(personal.owner.kind).toBe("account");
    await page.goto("/#/worlds/" + official.id + "/world?tab=achievements");
    const chooser = page.getByLabel("Progress for", { exact: true });
    await expect(chooser).toHaveValue(me.account.id);
    for (const owner of [
      { id: me.account.id, name: "Personal", kind: "account" },
      ...created.map((team) => ({ ...team, kind: "team" })),
    ]) {
      const group = progress.achievements.find(
        (group: any) => group.owner.id === owner.id,
      );
      expect(group.owner.kind).toBe(owner.kind);
      expect(group.achievements.length).toBeGreaterThan(0);
      expect(
        group.achievements.every(
          (achievement: any) => achievement.team === (owner.kind === "team"),
        ),
      ).toBeTruthy();
      await chooser.selectOption(owner.id);
      await expect(page.locator(".achievement .eyebrow")).toHaveText(
        group.achievements.map(() => owner.name),
      );
      await expect(page.locator(".achievement progress")).toHaveCount(
        group.achievements.length,
      );
    }
    expect(errors).toEqual([]);
  } finally {
    // These teams have no game events/assets. Never modify pre-existing teams.
    for (const team of [...created].reverse()) {
      const response = await page.request.post("/api/v1/commands", {
        headers: { "x-csrf-token": me.csrf, origin },
        data: {
          request_id: crypto.randomUUID(),
          command: { type: "team_disband", team: team.id },
        },
      });
      expect(
        response.ok(),
        "Disband isolated browser team " + team.id,
      ).toBeTruthy();
    }
    const restored = await page.request.post("/api/v1/commands", {
      headers: { "x-csrf-token": me.csrf, origin },
      data: {
        request_id: crypto.randomUUID(),
        command: { type: "team_contribution_set", team: originalContribution },
      },
    });
    expect(
      restored.ok(),
      "Restore the fixture's contribution choice",
    ).toBeTruthy();
    const final = await projection("/api/v1/view/social?section=teams");
    expect(final.teams.map((team: any) => team.id).sort()).toEqual(initialIds);
    expect(final.contribution_team_id).toBe(originalContribution);
  }
});
