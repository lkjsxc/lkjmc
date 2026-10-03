import { createRequire } from "node:module";
import fs from "node:fs";
import assert from "node:assert/strict";
const require = createRequire(import.meta.url);
const { chromium } = require("../web/node_modules/@playwright/test");
const browser = await chromium.launch({ headless: true });
fs.mkdirSync(".local/production-ux", { recursive: true, mode: 0o700 });
const results = [];
const sid = "00000000-0000-0000-0000-000000000001";
const aid = "00000000-0000-0000-0000-000000000002";
const pages = [
  "home",
  "play",
  "smp",
  "social",
  "life",
  "market",
  "adventure",
  "servers",
  "settings",
  "admin",
  ...["notifications", "invitations", "activity"].map((p) => "/home/" + p),
  "/friends/incoming",
  "/friends/outgoing",
  "/chat",
  "/teams",
  "/teams/members",
  "/teams/settings",
  "/parties",
  "/parties/members",
  "/parties/ready",
  ...["privacy", "linking", "blocks", "reports"].map((p) => "/account/" + p),
  ...["reports", "ranks", "backups", "jobs", "audit"].map((p) => "/admin/" + p),
  "/manage/servers/new",
  "/manage/communities",
  ...[
    "overview",
    "console",
    "files",
    "backups",
    "members",
    "settings",
    "activity",
  ].map((p) => "/manage/servers/" + sid + (p === "overview" ? "" : "/" + p)),
  ...[
    "overview",
    "land",
    "homes",
    "coins",
    "coin-history",
    "achievements",
    "meetup",
    "stored-assets",
    "materials",
  ].map((p) => "/servers/" + sid + (p === "overview" ? "" : "/" + p)),
];
const server = {
  id: "00000000-0000-0000-0000-000000000001",
  name: "SMP",
  kind: "official",
  maintenance: false,
  observed: "stopped",
  players: 0,
  version: "1.21.11",
  software: "paper",
  capabilities: { proxy_join: true, bedrock: true },
  memory_mib: 2048,
  cpu_millis: 2000,
  storage_mib: 10240,
  can_manage: true,
  can_administer: true,
  last_observed_at: new Date().toISOString(),
  desired: "stopped",
  members: [],
  artifacts: [],
  backups: [],
};
try {
  for (const width of [320, 360, 390, 768, 1024, 1440]) {
    for (const language of ["en", "ja"]) {
      const context = await browser.newContext({
        viewport: { width, height: 1000 },
        locale: "ja-JP",
      });
      let savedLanguage = language;
      await context.route("**/api/**", async (route) => {
        const pathname = new URL(route.request().url()).pathname;
        let body = {};
        if (pathname === "/api/v1/me")
          body = {
            account: {
              id: "00000000-0000-0000-0000-000000000002",
              name: "VeryLongPlayerName_ABCDEFGHIJKLMNOPQRSTUVWXYZ",
              administrator: true,
              language: savedLanguage,
              rank: {
                name: "Member",
                server_count: 2,
                concurrent_servers: 1,
                memory_mib: 2048,
                cpu_millis: 2000,
                storage_mib: 10240,
              },
              identities: [],
              dm_policy: "friends",
              activity_policy: "friends",
            },
            csrf: "fixture",
            game_address: "lkjsxc.com:25591",
            development: false,
          };
        else if (pathname === "/api/v1/commands") {
          const command = route.request().postDataJSON().command;
          if (command.type === "language") savedLanguage = command.language;
          body = { result: { language: savedLanguage } };
        } else if (pathname.startsWith("/api/v1/"))
          body = {
            servers: [
              server,
              {
                ...server,
                id: "00000000-0000-0000-0000-000000000003",
                name: "Second server",
              },
            ],
            server,
            presets: [
              { software: "paper", version: "1.21.11", java: 21 },
              { software: "paper", version: "26.2", java: 25 },
            ],
            counts: { notifications: 67, invitations: 8, jobs: 12 },
            invitations: [
              {
                id: crypto.randomUUID(),
                sender_name: "A friend",
                kind: "team",
                created_at: new Date().toISOString(),
              },
            ],
            notifications: Array.from(
              { length: pathname.includes("history") ? 25 : 3 },
              (_, n) => ({
                id: 100 - n,
                kind: "message",
                created_at: new Date().toISOString(),
                body: {},
              }),
            ),
            jobs: [
              {
                id: crypto.randomUUID(),
                kind: "server.create",
                state: "waiting",
                progress: { message: "Waiting to resume" },
                created_at: new Date().toISOString(),
              },
            ],
            friends: [
              {
                id: crypto.randomUUID(),
                name: "A friend",
                requester: aid,
                state: "accepted",
              },
              {
                id: crypto.randomUUID(),
                name: "Incoming request",
                requester: sid,
                state: "pending",
              },
              {
                id: crypto.randomUUID(),
                name: "Outgoing request",
                requester: aid,
                state: "pending",
              },
            ],
            rooms: [
              {
                id: sid,
                name: "Test conversation",
                kind: "group",
                unread: 2,
                members: [{ id: aid, name: "Player" }],
              },
            ],
            communities: [],
            team: {
              id: sid,
              name: "Test team",
              leader: aid,
              room_id: sid,
              members: [
                { account_id: aid, name: "Player" },
                { account_id: sid, name: "Teammate" },
              ],
            },
            party: {
              id: sid,
              name: "Test party",
              leader: aid,
              room_id: sid,
              members: [{ account_id: aid, name: "Player", ready: true }],
            },
            owners: [
              {
                id: aid,
                name: "Player",
                kind: "account",
                wallet: { balance: 2000, reserved: 0 },
                land: { chunks: 4 },
                used_chunks: 2,
              },
            ],
            claims: [],
            homes: [],
            achievements: [],
            ledger: [],
            listings: [],
            assets: [],
            prices: [],
            adventures: [],
            links: [],
            blocks: [{ id: sid, name: "Blocked player" }],
            reports: [
              {
                id: sid,
                reason: "Example report",
                status: "open",
                created_at: new Date().toISOString(),
              },
            ],
            ranks: [
              {
                id: 1,
                name: "Player",
                server_count: 2,
                concurrent_servers: 1,
                memory_mib: 2048,
                cpu_millis: 1000,
                storage_mib: 10240,
              },
            ],
            audit: [],
            backups: [],
            backup_policy: { enabled: false },
            cost: { coins: 1000, ender_eyes: 12 },
          };
        if (pathname.includes("history")) body.next_cursor = "50";
        if (pathname.includes("/messages")) body.messages = [];
        await route.fulfill({ json: body });
      });
      const page = await context.newPage();
      const errors = [];
      page.on("pageerror", (error) => errors.push(error.message));
      for (const route of pages) {
        await page.goto("http://127.0.0.1:18194/#" + route);
        await page.locator("h1").waitFor();
        await page
          .getByText(language === "en" ? "Loading…" : "読み込んでいます…", {
            exact: true,
          })
          .waitFor({ state: "hidden" });
        await page.waitForTimeout(40);
        const contrast = await page
          .locator(
            "main input:not([type=checkbox]):not([type=file]), main select, main textarea",
          )
          .evaluateAll((nodes) => {
            const luminance = (rgb) =>
              rgb
                .slice(0, 3)
                .map((v) => v / 255)
                .map((v) =>
                  v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4,
                )
                .reduce(
                  (sum, v, i) => sum + v * [0.2126, 0.7152, 0.0722][i],
                  0,
                );
            return nodes
              .filter((n) => n.getClientRects().length)
              .map((n) => {
                const c = getComputedStyle(n);
                const rgb = (s) => (s.match(/[\d.]+/g) || []).map(Number);
                const fg = luminance(rgb(c.color)),
                  bg = luminance(rgb(c.backgroundColor));
                return {
                  type: n.type,
                  ratio: (Math.max(fg, bg) + 0.05) / (Math.min(fg, bg) + 0.05),
                  background: c.backgroundColor,
                };
              });
          });
        for (const c of contrast)
          assert.ok(
            c.ratio >= 4.5,
            `${route}: input contrast ${JSON.stringify(c)}`,
          );
        const columns = await page
          .locator("main .grid.two, main .grid.three")
          .evaluateAll((nodes) =>
            nodes.map(
              (n) => getComputedStyle(n).gridTemplateColumns.split(" ").length,
            ),
          );
        assert.ok(
          columns.every((n) => n === 1),
          `${route}: main content must use one column`,
        );
        if (route === "social")
          assert.equal(await page.locator(".chat-layout").count(), 0);
        if (route === "/chat")
          assert.equal(await page.locator(".friend-row").count(), 0);
        if (route === "home")
          assert.ok((await page.locator("main .list-row").count()) <= 9);
        if (route === "/manage/servers/new")
          assert.equal(
            await page
              .getByRole("option", { name: "paper 1.21.11 · Java 21" })
              .count(),
            1,
          );
        assert.equal(await page.locator("html").getAttribute("lang"), language);
        const size = await page.evaluate(() => ({
          width: document.documentElement.clientWidth,
          scroll: document.documentElement.scrollWidth,
        }));
        if (size.scroll > size.width)
          throw Error(
            `${width}/${language}/${route}: overflow ${JSON.stringify(size)}`,
          );
        assert.deepEqual(errors, [], `${width}/${language}/${route}`);
        if (["home", "settings", "smp"].includes(route))
          await page.screenshot({
            path: `.local/production-ux/${language}-${width}-${route}.png`,
            fullPage: true,
          });
        results.push({
          width,
          language,
          page: route,
          overflow: false,
          errors: 0,
        });
      }
      if (width <= 850) {
        await page
          .getByRole("button", {
            name: language === "en" ? "Open menu" : "メニューを開く",
            exact: true,
          })
          .click();
        await page.keyboard.press("Escape");
        assert.equal(await page.locator("aside").getAttribute("inert"), "");
      }
      await page.goto("http://127.0.0.1:18194/#settings");
      await page.locator("h1").waitFor();
      const select = page.locator("main").getByRole("combobox", {
        name: language === "en" ? "Language" : "言語",
        exact: true,
      });
      await select.selectOption(language === "en" ? "ja" : "en");
      await page.waitForFunction(
        (expected) => document.documentElement.lang === expected,
        language === "en" ? "ja" : "en",
      );
      await page.reload();
      await page.locator("h1").waitFor();
      assert.equal(
        await page.locator("html").getAttribute("lang"),
        language === "en" ? "ja" : "en",
      );
      await context.close();
    }
  }
  const context = await browser.newContext({
    locale: "ja-JP",
    viewport: { width: 320, height: 800 },
  });
  await context.route("**/api/v1/me", (route) =>
    route.fulfill({
      status: 401,
      json: { error: { message: "Please sign in." } },
    }),
  );
  await context.route("**/health/ready", (route) =>
    route.fulfill({ json: { login_configured: true } }),
  );
  const page = await context.newPage();
  await page.goto("http://127.0.0.1:18194/");
  await page.getByRole("link", { name: "Sign up / Sign in" }).waitFor();
  assert.equal(await page.locator("html").getAttribute("lang"), "en");
  await page.screenshot({
    path: ".local/production-ux/landing-320.png",
    fullPage: true,
  });
  await page
    .getByRole("combobox", { name: "Language", exact: true })
    .selectOption("ja");
  await page.reload();
  await page.getByRole("link", { name: "登録・ログイン" }).waitFor();
  await context.close();
  fs.writeFileSync(
    ".local/production-ux/browser-results.json",
    JSON.stringify(results, null, 2),
  );
  console.log(
    `PASS: ${results.length} page/locale/viewport checks, 12 saved language reloads, mobile navigation, English-first landing and anonymous persistence.`,
  );
} finally {
  await browser.close();
}
