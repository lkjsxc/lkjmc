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
  await page.getByRole("button", { name: "Protect land now", exact: true }).click();
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
