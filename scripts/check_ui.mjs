import { createRequire } from "node:module";
import fs from "node:fs";
import assert from "node:assert/strict";
const require = createRequire(import.meta.url);
const { chromium } = require("../web/node_modules/@playwright/test");
const browser = await chromium.launch({ headless: true });
fs.mkdirSync(".local/production-ux", { recursive: true, mode: 0o700 });
const results = [];
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
];
const server = {
  id: "00000000-0000-0000-0000-000000000001",
  name: "SMP",
  kind: "official",
  maintenance: false,
  observed: "stopped",
  players: 0,
  version: "1.21.10",
  software: "paper",
  capabilities: { proxy_join: true, bedrock: true },
  memory_mib: 2048,
  cpu_millis: 2000,
  storage_mib: 10240,
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
        } else if (pathname.startsWith("/api/v1/view/"))
          body = {
            servers: [server],
            invitations: [],
            notifications: [],
            jobs: [],
            friends: [],
            rooms: [],
            communities: [],
            team: null,
            party: null,
            owners: [],
            claims: [],
            homes: [],
            achievements: [],
            ledger: [],
            listings: [],
            assets: [],
            prices: [],
            adventures: [],
            links: [],
            blocks: [],
            reports: [],
            ranks: [],
            audit: [],
            backups: [],
            backup_policy: { enabled: false },
            cost: { coins: 1000, ender_eyes: 12 },
          };
        await route.fulfill({ json: body });
      });
      const page = await context.newPage();
      const errors = [];
      page.on("pageerror", (error) => errors.push(error.message));
      for (const route of pages) {
        await page.goto("http://127.0.0.1:18194/#" + route);
        await page.locator("h1").waitFor();
        await page.waitForTimeout(75);
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
      const select = page
        .locator("main")
        .getByRole("combobox", {
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
