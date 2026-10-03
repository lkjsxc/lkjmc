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
      "Integration lane requires a dedicated local test session at " +
        file +
        ". Provision the dedicated test service/session explicitly; this lane never skips or falls back to mocks.",
    );
  }
});
test.beforeEach(async ({ context }) => {
  await context.addCookies([
    {
      name: "lkjmc_session",
      value: session.token,
      url: "http://127.0.0.1:18091",
      httpOnly: true,
      sameSite: "Lax",
    },
  ]);
  const response = await context.request.get(
    "http://127.0.0.1:18091/api/v1/me",
  );
  expect(
    response.ok(),
    "Dedicated test session must authenticate /me",
  ).toBeTruthy();
  const me = await response.json();
  const language = await context.request.post(
    "http://127.0.0.1:18091/api/v1/commands",
    {
      headers: { "x-csrf-token": me.csrf, origin: "http://127.0.0.1:18091" },
      data: {
        request_id: crypto.randomUUID(),
        command: { type: "language", language: "ja" },
      },
    },
  );
  expect(language.ok()).toBeTruthy();
});

test("desktop pages load real API states and retain working navigation", async ({
  page,
}) => {
  const errors: string[] = [];
  page.on("pageerror", (e) => errors.push(e.message));
  await page.goto("/");
  await expect(
    page.getByRole("heading", { name: "ホーム", exact: true }),
  ).toBeVisible();
  for (const name of [
    "サーバー一覧",
    "フレンド",
    "タイムライン",
    "チーム",
    "パーティー",
    "サーバー管理",
    "運営管理",
  ]) {
    await page
      .getByRole("navigation", { name: "メインメニュー" })
      .getByRole("link", { name, exact: true })
      .click();
    const title =
      name === "チーム"
        ? ((
            await (
              await page.request.get("/api/v1/view/social?section=teams")
            ).json()
          ).team?.name ?? name)
        : name;
    await expect(
      page.getByRole("heading", { name: title, exact: true, level: 1 }),
    ).toBeVisible();
    await expect(
      page.getByText("読み込んでいます…", { exact: true }),
    ).not.toBeVisible();
    await expect(page.locator("[role=alert]")).toHaveCount(0);
  }
  await page.getByRole("link", { name: /のアカウント設定$/ }).click();
  await expect(
    page.getByRole("heading", {
      name: "アカウント設定",
      exact: true,
      level: 1,
    }),
  ).toBeVisible();
  await expect(
    page.getByText("読み込んでいます…", { exact: true }),
  ).not.toBeVisible();
  await expect(page.locator("[role=alert]")).toHaveCount(0);
  await page
    .getByRole("navigation", { name: "メインメニュー" })
    .getByRole("link", { name: "サーバー一覧", exact: true })
    .click();
  await page
    .locator(".server-row")
    .filter({ hasText: "official development" })
    .getByRole("link", { name: "official development", exact: true })
    .click();
  await expect(
    page.getByRole("heading", { name: "サーバーの詳細", exact: true }),
  ).toBeVisible();
  for (const name of [
    "土地・資産",
    "保護した土地",
    "マーケット",
    "プライベート End",
  ]) {
    await expect(
      page
        .getByRole("navigation", { name: "メインメニュー" })
        .getByRole("link", { name, exact: true }),
    ).toHaveCount(0);
    await page
      .getByRole("navigation", { name: "ページ内メニュー" })
      .getByRole("link", { name, exact: true })
      .click();
    await page.reload();
    await expect(
      page.getByRole("heading", { name, exact: true, level: 1 }),
    ).toBeVisible();
    await expect(
      page.getByText("読み込んでいます…", { exact: true }),
    ).not.toBeVisible();
    await expect(page.locator("[role=alert]")).toHaveCount(0);
    await expect(
      page
        .getByRole("navigation", { name: "現在の位置" })
        .getByRole("link", { name: "official development", exact: true }),
    ).toBeVisible();
  }
  await page
    .getByRole("navigation", { name: "メインメニュー" })
    .getByRole("link", { name: "ホーム", exact: true })
    .click();
  await page.screenshot({ path: "../.local/home-desktop.png", fullPage: true });
  expect(errors).toEqual([]);
});

test("a group and a message persist through a browser reload", async ({
  page,
}) => {
  const name = `検証グループ ${Date.now()}`;
  const reason = `ブラウザ受入検証 ${name}`;
  await page.goto("/#/timeline");
  await expect(
    page.getByRole("heading", { name: "タイムライン", exact: true }),
  ).toBeVisible();
  await page
    .getByRole("button", { name: "グループチャットを作る", exact: true })
    .click();
  await page.getByLabel("グループ名", { exact: true }).fill(name);
  await page
    .getByRole("dialog")
    .getByRole("button", { name: "作成する", exact: true })
    .click();
  await expect(page.getByRole("dialog")).toHaveCount(0);
  await expect(
    page.getByLabel("送信先", { exact: true }).locator("option:checked"),
  ).toHaveText(name);
  await page
    .getByLabel("メッセージ", { exact: true })
    .fill("これは実際のデータベースに保存されるメッセージです。");
  await page.getByRole("button", { name: "送信", exact: true }).click();
  await expect(
    page
      .locator("article.message")
      .getByText("これは実際のデータベースに保存されるメッセージです。", {
        exact: true,
      }),
  ).toBeVisible();
  await page.reload();
  await expect(
    page.getByLabel("送信先", { exact: true }).locator("option:checked"),
  ).toHaveText(name);
  await expect(
    page
      .locator("article.message")
      .getByText("これは実際のデータベースに保存されるメッセージです。", {
        exact: true,
      }),
  ).toBeVisible();
  await page
    .getByRole("button", { name: "メッセージを通報", exact: true })
    .click();
  await page.getByRole("checkbox").check();
  await page
    .getByRole("button", { name: "提出内容を確認", exact: true })
    .click();
  await expect(
    page
      .getByRole("dialog")
      .getByText("これは実際のデータベースに保存されるメッセージです。", {
        exact: true,
      }),
  ).toBeVisible();
  await page.getByLabel("通報の理由", { exact: true }).fill(reason);
  await page
    .getByRole("button", { name: "この内容で通報する", exact: true })
    .click();
  await expect(page.getByRole("dialog")).toHaveCount(0);
  await page.goto("/#/account/reports");
  await expect(page.getByText(reason, { exact: true })).toBeVisible();
});

test("mobile navigation and dialogs fit a narrow viewport", async ({
  page,
}) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto("/");
  await expect(
    page.getByRole("heading", { name: "ホーム", exact: true }),
  ).toBeVisible();
  await page.getByRole("button", { name: "メニューを開く" }).click();
  await page
    .getByRole("navigation", { name: "メインメニュー" })
    .getByRole("link", { name: "サーバー一覧", exact: true })
    .click();
  await page
    .locator(".server-row")
    .filter({ hasText: "official development" })
    .getByRole("link", { name: "official development", exact: true })
    .click();
  await page
    .getByRole("navigation", { name: "ページ内メニュー" })
    .getByRole("link", { name: "保護した土地", exact: true })
    .click();
  await page.getByRole("button", { name: "土地を保護", exact: true }).click();
  await expect(page.getByRole("dialog")).toBeVisible();
  const width = await page.evaluate(() => ({
    scroll: document.documentElement.scrollWidth,
    client: document.documentElement.clientWidth,
  }));
  expect(width.scroll).toBeLessThanOrEqual(width.client);
  await page.screenshot({ path: "../.local/land-mobile.png", fullPage: true });
});
