import fs from "node:fs";
import { test, expect } from "@playwright/test";
import { mountFixture, sid, otherSid, aid, rid, teamId } from "./fixture.mjs";

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

for (const language of ["en", "ja"]) {
  for (const width of [390, 768, 1440]) {
    test(`hierarchy stays first without duplicate headings at ${width}px in ${language}`, async ({
      context,
      page,
    }, info) => {
      await page.setViewportSize({ width, height: 900 });
      const state = await mountFixture(context, { language });
      state.server.name = "長いワールド名とVeryLongUnbrokenServerName".repeat(
        3,
      );
      await page.goto(url(`/hosting/servers/${sid}/console`));
      const crumbs = page.locator(".topbar .breadcrumbs");
      await expect(crumbs).toHaveCount(1);
      await expect(crumbs).toContainText(catalogs[language]["text.hosting"]);
      await expect(crumbs).toContainText(state.server.name);
      await expect(crumbs.locator('[aria-current="page"]')).toHaveText(
        catalogs[language]["text.console"],
      );
      await expect(page.locator(".page-heading")).toHaveCount(0);
      await expect(page.locator("main > h1")).toHaveClass("sr-only");
      const boxes = await page.evaluate(() => ({
        top: document
          .querySelector(".topbar")!
          .getBoundingClientRect()
          .toJSON(),
        crumbs: document
          .querySelector(".breadcrumbs")!
          .getBoundingClientRect()
          .toJSON(),
        main: document.querySelector("main")!.getBoundingClientRect().toJSON(),
        width: document.documentElement.scrollWidth,
      }));
      expect(boxes.crumbs.y).toBeGreaterThanOrEqual(0);
      expect(boxes.crumbs.bottom).toBeLessThanOrEqual(boxes.top.bottom + 1);
      expect(boxes.main.y).toBeGreaterThanOrEqual(boxes.top.bottom - 1);
      expect(boxes.width).toBeLessThanOrEqual(width + 1);
      await page.locator("main").evaluate((main) => {
        main.style.minHeight = "2000px";
      });
      await page.evaluate(() => window.scrollTo(0, 500));
      expect((await page.locator(".topbar").boundingBox())!.y).toBe(0);
      await page.evaluate(() => window.scrollTo(0, 0));
      await page.screenshot({ path: info.outputPath("hierarchy.png") });
    });
  }
}

test("Play and Worlds show the lobby and use the whole native card as a link", async ({
  context,
  page,
}, info) => {
  const state = await mountFixture(context);
  state.worlds = [
    state.server,
    { ...state.server, id: otherSid, name: "Lobby", kind: "lobby" },
  ];
  await page.goto(url("/play"));
  await expect(
    page.locator(".play-hero,.world-detail-hero,.join-guidance,.page-heading"),
  ).toHaveCount(0);
  await expect(page.locator("main")).not.toContainText("Welcome back");
  await expect(
    page.getByRole("link", { name: "Lobby", exact: true }),
  ).toHaveAttribute("href", `#/worlds/${otherSid}`);
  await expect(page.locator(".world-card")).toHaveCount(2);
  const world = page.locator(`a.world-card[href="#/worlds/${sid}"]`);
  const box = (await world.boundingBox())!;
  await world.click({ position: { x: box.width - 6, y: box.height - 6 } });
  await expect(page).toHaveURL(url(`/worlds/${sid}`));
  await expect(page.locator(".breadcrumbs")).toContainText("Workshop");
  await expect(page.locator(".world-detail-hero")).toHaveCount(0);
  await page.goto(url("/worlds"));
  await expect(page.locator(".world-card")).toHaveCount(2);
  await page.screenshot({
    path: info.outputPath("worlds.png"),
    fullPage: true,
  });
});

test("teams open from their padding and keyboard, with creation after the existing list", async ({
  context,
  page,
}, info) => {
  await mountFixture(context);
  await page.goto(url("/people/teams"));
  const team = page.getByRole("link", { name: "Builders team", exact: true });
  const create = page.getByRole("button", { name: "Create team", exact: true });
  await expect(team).toHaveAttribute("href", `#/people/teams/${teamId}`);
  const row = (await team.boundingBox())!;
  expect((await create.boundingBox())!.y).toBeGreaterThanOrEqual(
    row.y + row.height,
  );
  await expect(
    page.locator(".team-contribution,.team-contribution-label"),
  ).toHaveCount(0);
  await team.click({ position: { x: row.width - 6, y: row.height - 6 } });
  await expect(page).toHaveURL(url(`/people/teams/${teamId}`));
  await expect(page.locator(".breadcrumbs")).toContainText("People");
  await expect(page.locator(".breadcrumbs")).toContainText("Teams");
  await expect(page.locator(".breadcrumbs")).toContainText("Builders team");
  await expect(page.locator(".team-contribution")).toHaveCount(0);
  await page.goto(url("/people/teams"));
  await team.focus();
  await page.keyboard.press("Enter");
  await expect(page).toHaveURL(url(`/people/teams/${teamId}`));
  await page.screenshot({ path: info.outputPath("team.png"), fullPage: true });
});

test("an exhausted Timeline has no phantom pagination, reporting, or unavailable-service promotion", async ({
  context,
  page,
}) => {
  const state = await mountFixture(context);
  await page.clock.install({ time: new Date("2026-10-03T10:00:00Z") });
  state.items = [
    {
      ...state.message(1, "My retained message", rid),
      author: aid,
      before_cursor: "before:1",
    },
  ];
  await context.route("**/api/v1/timeline**", async (route) => {
    await route.fulfill({
      contentType: "application/json",
      body: JSON.stringify({
        items: state.items,
        rooms: state.rooms,
        next_cursor: null,
        updates: [],
        removed_ids: [],
      }),
    });
  });
  await page.goto(url(`/timeline?room=${rid}`));
  await expect(
    page.getByText("My retained message", { exact: true }),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Load earlier items", exact: true }),
  ).toHaveCount(0);
  await expect(
    page.getByRole("button", { name: "Create group chat", exact: true }),
  ).toHaveCount(0);
  await expect(
    page.getByRole("button", { name: /Conversation options|Timeline options/ }),
  ).toHaveCount(0);
  await expect(
    page.getByText("Voice service is being set up", { exact: true }),
  ).toHaveCount(0);
  await expect(page.getByRole("button", { name: /Report/ })).toHaveCount(0);
  await page.clock.fastForward(9000);
  await expect(
    page.getByRole("button", { name: "Load earlier items", exact: true }),
  ).toHaveCount(0);
  await expect(
    page.getByRole("button", { name: "New private conversation", exact: true }),
  ).toBeVisible();
});

test("report pages are absent from account and administration navigation", async ({
  context,
  page,
}) => {
  await mountFixture(context);
  for (const path of ["/account", "/admin"]) {
    await page.goto(url(path));
    await expect(page.locator("main")).toBeVisible();
    await expect(page.locator('a[href*="/reports"]')).toHaveCount(0);
  }
});
