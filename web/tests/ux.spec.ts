import { test, expect, type Page } from "@playwright/test";
import { mountFixture, sid, otherSid, rid, groupId } from "./fixture.mjs";
const url = (path: string) => "https://ux.fixture/#" + path;
async function chooseConversation(page: Page, room: string) {
  const link = page
    .getByRole("navigation", { name: "Conversations" })
    .locator(`a[href="#/timeline?room=${room}"]`);
  await link.click();
  await expect(link).toHaveAttribute("aria-current", "page");
  await expect(page.getByLabel("Message", { exact: true })).toBeVisible();
}
async function reportMessages(page: Page) {
  await page.locator(".timeline-toolbar .context-menu > summary").click();
  await page
    .getByRole("button", { name: "Report messages", exact: true })
    .click();
}
async function tick(page: Page, count = 3) {
  for (let i = 0; i < count; i++) {
    await page.clock.fastForward(2500);
    await page.waitForTimeout(50);
  }
}
async function setup(context: any, page: Page, options = {}) {
  const state = await mountFixture(context, options);
  await page.clock.install({ time: new Date("2026-10-03T10:00:00Z") });
  return state;
}
test("navigation has working destinations and account settings at the bottom", async ({
  context,
  page,
}) => {
  await setup(context, page);
  await page.goto(url("/play"));
  const nav = page.getByRole("navigation", { name: "Main menu" });
  await expect(
    nav.getByRole("link", { name: "Timeline", exact: true }),
  ).toBeVisible();
  await expect(
    nav.getByRole("link", { name: "Account", exact: true }),
  ).toHaveCount(0);
  await expect(page.locator("aside select, footer, .job-tray")).toHaveCount(0);
  await expect(
    page.getByRole("button", { name: "Refresh", exact: true }),
  ).toHaveCount(0);
  await expect(page.locator(".play-hero")).toBeVisible();
  await expect(page.locator(".message")).toHaveCount(0);
  await page.getByRole("link", { name: "Account settings for Alex" }).click();
  await expect(
    page.getByRole("combobox", { name: "Language", exact: true }),
  ).toBeVisible();
  await page
    .getByRole("combobox", { name: "Language", exact: true })
    .selectOption("ja");
  await expect(page.locator("html")).toHaveAttribute("lang", "ja");
  await page.goto(url("/hosting/servers"));
  await expect(page.locator("h1")).toHaveText("あなたのサーバー");
  await expect(
    page.getByRole("link", { name: "詳細", exact: true }),
  ).toHaveCount(0);
  await page
    .locator(".server-row")
    .getByRole("link", { name: "Workshop", exact: true })
    .click();
  await expect(
    page
      .locator("main .metric")
      .filter({ has: page.getByText("メモリ", { exact: true }) })
      .locator("dd"),
  ).toHaveText(/2\s*GiB/);
  await expect(
    page
      .locator("main .metric")
      .filter({ has: page.getByText("CPU", { exact: true }) })
      .locator("dd"),
  ).toHaveText(/2\s*vCPU/);
  await expect(page.locator("main .output")).toHaveCount(0);
});
test("team tabs use the team name and direct Members/Settings contents", async ({
  context,
  page,
}) => {
  await setup(context, page);
  await page.goto(url("/people/teams"));
  const nav = page.getByRole("navigation", { name: "Page menu" });
  await expect(
    nav.getByRole("link", { name: "Builders team", exact: true }),
  ).toBeVisible();
  await expect(nav.getByRole("link", { name: "Overview" })).toHaveCount(0);
  await nav.getByRole("link", { name: "Members", exact: true }).click();
  await expect(page.getByText("Bea", { exact: true })).toBeVisible();
  await nav.getByRole("link", { name: "Settings", exact: true }).click();
  await expect(
    page.getByRole("button", { name: "Leave team", exact: true }),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Invite member", exact: true }),
  ).toHaveCount(0);
});
test("Timeline aligns reading and sending, retains isolated drafts and exposes message report evidence", async ({
  context,
  page,
}) => {
  const state = await setup(context, page);
  await page.goto(url("/timeline"));
  await expect(page.locator(".timeline-composer")).toHaveCount(0);
  await chooseConversation(page, rid);
  await expect(
    page.getByRole("region", { name: "Messages in Bea" }),
  ).toBeVisible();
  await page.getByLabel("Message", { exact: true }).fill("Private draft");
  await chooseConversation(page, groupId);
  await expect(page.locator(".timeline-feed")).not.toContainText("Message 10");
  await page.getByLabel("Message", { exact: true }).fill("Group draft");
  await chooseConversation(page, rid);
  await expect(page.getByLabel("Message", { exact: true })).toHaveValue(
    "Private draft",
  );
  state.sendFailure = true;
  await page.getByRole("button", { name: "Send", exact: true }).click();
  await expect(page.getByRole("alert")).toContainText("Request failed (503)");
  await page.getByLabel("Message", { exact: true }).focus();
  await tick(page, 4);
  await expect(page.getByLabel("Message", { exact: true })).toBeFocused();
  await expect(page.getByRole("alert")).toContainText("Request failed (503)");
  state.sendFailure = false;
  await page.getByRole("button", { name: "Send", exact: true }).click();
  await expect(page.getByLabel("Message", { exact: true })).toHaveValue("");
  await expect(
    page.locator(".timeline-feed").getByText("Private draft", { exact: true }),
  ).toBeVisible();
  expect(
    state.commands.filter((c: any) => c.type === "message_send").at(-1),
  ).toMatchObject({ room: rid, body: "Private draft" });
  await reportMessages(page);
  await page.getByRole("checkbox").first().check();
  await page.getByRole("button", { name: "Review submission" }).click();
  await expect(page.getByRole("dialog")).toContainText("Message 10");
  await page
    .getByRole("dialog")
    .getByLabel("Reason for report")
    .fill("Fixture evidence");
  await page.getByRole("button", { name: "Submit this report" }).click();
  await expect(page.getByRole("dialog")).toHaveCount(0);
  expect(
    state.commands.some(
      (c: any) => c.type === "report" && c.message_ids.length === 1,
    ),
  ).toBeTruthy();
});
test("Timeline deletion updates immediately after confirmation without a polling tick", async ({
  context,
  page,
}) => {
  await setup(context, page);
  await page.goto(url("/timeline"));
  const article = page
    .locator("article.message")
    .filter({
      has: page.getByRole("button", {
        name: "Delete",
        exact: true,
        includeHidden: true,
      }),
    })
    .first();
  await expect(article).toBeVisible();
  const id = await article.getAttribute("data-item-id");
  await article.getByLabel("Message options for Alex").click();
  await article.getByRole("button", { name: "Delete", exact: true }).click();
  await page
    .getByRole("dialog")
    .getByRole("button", { name: "Confirm deletion", exact: true })
    .click();
  await expect(page.getByRole("dialog")).toHaveCount(0);
  await expect(page.locator(`[data-item-id="${id}"]`)).toContainText(
    "Deleted message",
  );
});
test("Timeline older pagination, equal-time updates, scroll and selection survive tail polling", async ({
  context,
  page,
}) => {
  const state = await setup(context, page);
  await page.goto(url("/timeline"));
  await expect(page.locator(".timeline-feed [data-item-id]")).toHaveCount(26);
  await page.getByRole("button", { name: "Load earlier items" }).click();
  await expect(page.locator(".timeline-feed [data-item-id]")).toHaveCount(36);
  await reportMessages(page);
  await page.getByRole("checkbox").first().check();
  await page.locator(".timeline-feed").evaluate((e: any) => (e.scrollTop = 20));
  const feedRegion = page.getByRole("region", { name: "Timeline items" });
  await expect(feedRegion).toHaveAttribute("tabindex", "0");
  await feedRegion.focus();
  const before = await page
    .locator(".timeline-feed")
    .evaluate((e: any) => e.scrollTop);
  state.items[0].deleted_at = "2026-10-03T10:00:00Z";
  state.items.push(state.message(120, "New tail"));
  await tick(page, 4);
  await expect(page.locator(".timeline-feed [data-item-id]")).toHaveCount(37);
  await expect(page.getByRole("checkbox").first()).toBeChecked();
  await expect(feedRegion).toBeFocused();
  await expect(
    page.getByRole("status").filter({ hasText: "New updates are available." }),
  ).toBeAttached();
  await expect(
    page.locator('.timeline-feed [data-item-id="message:0"]'),
  ).toContainText("Earlier 0");
  expect(
    await page.locator(".timeline-feed").evaluate((e: any) => e.scrollTop),
  ).toBe(before);
  await expect(
    page.getByRole("button", { name: "Show new updates" }),
  ).toBeVisible();
  await page
    .getByRole("navigation", { name: "Conversations" })
    .getByRole("link", { name: /^Activity / })
    .click();
  await expect(page.locator(".timeline-feed .message")).toHaveCount(0);
  await expect
    .poll(() =>
      state.timelineRequests.some((q: string) => q.includes("kind=events")),
    )
    .toBeTruthy();
});
test("Timeline rejects read failures explicitly without discarding loaded history", async ({
  context,
  page,
}) => {
  const state = await setup(context, page);
  await page.goto(url("/timeline"));
  await expect(page.locator(".timeline-feed [data-item-id]")).toHaveCount(26);
  state.timelineFailure = true;
  await tick(page, 4);
  await expect(page.getByRole("alert")).toContainText(
    "Timeline could not update",
  );
  await expect(page.locator(".timeline-feed [data-item-id]")).toHaveCount(26);
  state.timelineFailure = false;
  await page.getByRole("button", { name: "Retry", exact: true }).click();
  await expect(page.getByRole("alert")).toHaveCount(0);
});
test("private and group creation open usable conversations and job/notification details show final response", async ({
  context,
  page,
}) => {
  const state = await setup(context, page);
  await page.goto(url("/timeline"));
  await page.getByRole("button", { name: "New private conversation" }).click();
  await page
    .getByRole("dialog")
    .getByRole("combobox", { name: "Player", exact: true })
    .fill("Bea");
  await tick(page, 1);
  await expect(page.getByRole("option", { name: /Bea/ })).toBeVisible();
  await page
    .getByRole("combobox", { name: "Player", exact: true })
    .press("ArrowDown");
  await page
    .getByRole("combobox", { name: "Player", exact: true })
    .press("Enter");
  await page
    .getByRole("button", { name: "Open conversation", exact: true })
    .click();
  await expect(page).toHaveURL(url("/timeline?room=" + rid));
  expect(
    state.commands.find((c: any) => c.type === "direct_room"),
  ).toMatchObject({ target: "bea" });
  await expect(
    page.getByRole("region", { name: "Messages in Bea" }),
  ).toBeVisible();
  await expect(page.getByLabel("Message", { exact: true })).toBeVisible();
  await page
    .getByRole("button", { name: "Create group chat", exact: true })
    .click();
  await page.getByLabel("Group name").fill("Builders");
  await page
    .getByRole("dialog")
    .getByRole("button", { name: "Create", exact: true })
    .click();
  await expect(page).toHaveURL(url("/timeline?room=" + groupId));
  await expect(
    page.getByRole("region", { name: "Messages in Builders" }),
  ).toBeVisible();
  await expect(page.getByLabel("Message", { exact: true })).toBeVisible();
  await page.goto(url("/play/notifications"));
  await page
    .locator(".list-row")
    .filter({ hasText: "Create backup · Workshop" })
    .getByRole("button", { name: "View details" })
    .click();
  await tick(page);
  await expect(page.getByRole("dialog")).toContainText("backup-123");
  await page
    .getByRole("dialog")
    .getByRole("button", { name: "Close", exact: true })
    .click();
  await page
    .locator(".list-row")
    .filter({ hasText: "Coins received · Alex" })
    .getByRole("button", { name: "View details" })
    .click();
  await expect(page.getByRole("dialog").locator(".job-response")).toContainText(
    "Coins received",
  );
});
test("job detail leads with understandable progress and keeps technical data optional", async ({
  context,
  page,
}) => {
  const state = await setup(context, page);
  Object.assign(state.jobs.get("completed"), {
    state: "leased",
    gets: 2,
    result: null,
  });
  state.jobs.get("completed").progress = {
    message: { id: "text.verifying", params: {} },
    percent: 75,
    stage: "archive_verification",
  };
  await page.goto(url("/play/notifications"));
  await page
    .locator(".list-row")
    .filter({ hasText: "Create backup · Workshop" })
    .getByRole("button", { name: "View details" })
    .click();
  const dialog = page.getByRole("dialog");
  await expect(dialog.getByRole("heading")).toHaveText(
    "Create backup · Workshop",
  );
  await expect(dialog.getByRole("status")).toHaveText("Verifying");
  await expect(
    dialog.getByRole("progressbar", { name: "Progress" }),
  ).toHaveAttribute("value", "75");
  const technical = dialog
    .locator("pre")
    .filter({ hasText: "archive_verification" });
  await expect(technical).not.toBeVisible();
  await dialog.getByText("Progress", { exact: true }).click();
  await expect(technical).toBeVisible();
  await dialog.getByText("Progress", { exact: true }).click();
  await expect(technical).not.toBeVisible();
});
test("file-close notification retains its operation while details are loading", async ({
  context,
  page,
}) => {
  const state = await setup(context, page);
  state.notices = [
    {
      id: 1,
      kind: "job_finished",
      body: {
        id: "completed",
        job_kind: "server.inspection",
        open: false,
        server_id: sid,
        server_name: "Workshop",
      },
      created_at: "2026-10-03T08:00:00Z",
    },
  ];
  Object.assign(state.jobs.get("completed"), {
    kind: "server.inspection",
    open: false,
  });
  state.delays.job = 1500;
  await page.goto(url("/play/notifications"));
  await page
    .locator(".list-row")
    .filter({ hasText: "Close files · Workshop" })
    .getByRole("button", { name: "View details" })
    .click();
  const dialog = page.getByRole("dialog");
  await expect(dialog.getByRole("status")).toHaveText("Loading…");
  expect(await dialog.getByRole("heading").textContent()).toBe(
    "Close files · Workshop",
  );
  await expect(dialog.getByText("Loading…", { exact: true })).toHaveCount(0);
  await expect(dialog.getByRole("heading")).toHaveText(
    "Close files · Workshop",
  );
});
test("Console waits for durable output, throttles pending reads, retains command errors and stops when inactive", async ({
  context,
  page,
}) => {
  const state = await setup(context, page);
  state.server.observed = state.server.desired = "running";
  state.pausedJobs = true;
  await page.goto(url(`/hosting/servers/${sid}/console`));
  await expect(page.getByText("Reading from the server…")).toBeVisible();
  await page.getByLabel("Console command", { exact: true }).fill("say hello");
  await tick(page, 5);
  expect(
    state.commands.filter((c: any) => c.type === "server_logs"),
  ).toHaveLength(1);
  await expect(page.getByLabel("Console command", { exact: true })).toHaveValue(
    "say hello",
  );
  state.pausedJobs = false;
  await tick(page, 4);
  await expect(page.getByLabel("Server output")).toContainText(
    "Fixture server stdout",
  );
  state.failNext.server_console = { id: "error.forbidden", params: {} };
  await page.getByRole("button", { name: "Send command", exact: true }).click();
  await expect(page.locator(".action-form [role=alert]")).toContainText(
    "You do not have permission to do this.",
  );
  await tick(page, 7);
  await expect(page.locator(".action-form [role=alert]")).toContainText(
    "You do not have permission to do this.",
  );
  const count = state.commands.filter(
    (c: any) => c.type === "server_logs",
  ).length;
  await page.goto(url("/play"));
  await tick(page, 9);
  expect(
    state.commands.filter((c: any) => c.type === "server_logs"),
  ).toHaveLength(count);
});
test("Logs uses an explicit UTC date, preserves history and never submits live reads", async ({
  context,
  page,
}) => {
  const state = await setup(context, page);
  await page.goto(url(`/hosting/servers/${sid}/logs`));
  await page.getByLabel("Log date (UTC)").fill("2026-10-01");
  await tick(page, 4);
  await expect(page.getByLabel("Server output")).toContainText(
    "Historical October 1",
  );
  await page.locator(".output").focus();
  await tick(page, 8);
  await expect(page.getByLabel("Log date (UTC)")).toHaveValue("2026-10-01");
  await expect(page.locator(".output")).toBeFocused();
  expect(
    state.commands
      .filter((c: any) => c.type === "server_logs")
      .every((c: any) => c.date),
  ).toBeTruthy();
  await page.getByRole("button", { name: "2026-10-02", exact: true }).click();
  await tick(page, 4);
  await expect(page.getByLabel("Server output")).toContainText(
    "Historical October 2",
  );
});
test("Files explores folders, guards stale text saves, targets uploads and confirms single-file deletion", async ({
  context,
  page,
}) => {
  const state = await setup(context, page);
  await page.goto(url(`/hosting/servers/${sid}/files`));
  await tick(page);
  await page
    .getByRole("button", { name: "Folder: documents", exact: true })
    .click();
  await tick(page);
  await page.getByRole("button", { name: "notes.txt", exact: true }).click();
  await tick(page);
  await expect(page.getByLabel("File text")).toHaveValue("enabled: true\n");
  await page.getByLabel("File text").fill("enabled: false\n");
  state.files.get("documents/notes.txt").sha = "sha-other-editor";
  await page.getByRole("button", { name: "Save file", exact: true }).click();
  await tick(page);
  await expect(page.locator(".file-editor [role=alert]").last()).toContainText(
    "File changed",
  );
  await expect(page.getByLabel("File text")).toHaveValue("enabled: false\n");
  expect(
    state.commands.find((c: any) => c.type === "server_file_write")
      .expected_sha256,
  ).toBe("sha-original");
  await page.getByRole("button", { name: "Close editor" }).click();
  await page.getByRole("button", { name: "notes.txt", exact: true }).click();
  await tick(page); // Reopen must finish a fresh authorization read.
  await expect(page.getByLabel("File text")).toHaveValue("enabled: false\n");
  await page.getByRole("button", { name: "Close editor" }).click();
  await page.locator("input[type=file]").setInputFiles({
    name: "attachment.txt",
    mimeType: "text/plain",
    buffer: Buffer.from("fixture"),
  });
  await expect(
    page.getByRole("dialog").getByLabel("Destination in server"),
  ).toHaveValue("documents/attachment.txt");
  await page
    .getByRole("dialog")
    .getByRole("button", { name: "Close", exact: true })
    .click();
  state.files.get("documents/notes.txt").sha = "sha-original";
  await page.getByRole("button", { name: "notes.txt", exact: true }).click();
  await tick(page);
  await page.getByRole("button", { name: "Delete file", exact: true }).click();
  await expect(page.getByRole("dialog")).toContainText("documents/notes.txt");
  expect(
    state.commands.filter((c: any) => c.type === "server_file_delete"),
  ).toHaveLength(0);
  await page.getByRole("button", { name: "Confirm deletion" }).click();
  await tick(page, 5);
  expect(
    state.commands.filter((c: any) => c.type === "server_file_delete"),
  ).toHaveLength(1);
});
test("direct settings keep typed names during refresh and isolates server command drafts", async ({
  context,
  page,
}) => {
  const state = await setup(context, page);
  await page.goto(url(`/hosting/servers/${sid}/settings`));
  await page.getByLabel("Name", { exact: true }).fill("Typed workshop");
  await page.getByLabel("Name", { exact: true }).focus();
  await tick(page, 7);
  await expect(page.getByLabel("Name", { exact: true })).toHaveValue(
    "Typed workshop",
  );
  await expect(page.getByLabel("Name", { exact: true })).toBeFocused();
  await page.getByRole("button", { name: "Save", exact: true }).click();
  expect(state.server.name).toBe("Typed workshop");
  await page.goto(url(`/hosting/servers/${sid}/console`));
  await page
    .getByLabel("Console command", { exact: true })
    .fill("first server draft");
  await page.goto(url(`/hosting/servers/${otherSid}/console`));
  await expect(page.getByLabel("Console command", { exact: true })).toHaveValue(
    "",
  );
  await page.goto(url(`/hosting/servers/${sid}/console`));
  await expect(page.getByLabel("Console command", { exact: true })).toHaveValue(
    "first server draft",
  );
});
test("native OP is separate from hosting roles and exposes pending/applied outcome", async ({
  context,
  page,
}) => {
  const state = await setup(context, page);
  await page.goto(url(`/hosting/servers/${sid}/members`));
  await expect(
    page.getByText(
      "No operator change recorded. Hosting roles do not grant Minecraft OP.",
    ),
  ).toBeVisible();
  await page
    .getByRole("button", { name: "Grant operator", exact: true })
    .click();
  await page
    .getByRole("dialog")
    .getByRole("button", { name: "Grant operator", exact: true })
    .click();
  await tick(page, 6);
  expect(
    state.commands.find((c: any) => c.type === "server_operator"),
  ).toMatchObject({
    id: sid,
    member: state.server.members[0].account_id,
    operator: true,
  });
  await expect(
    page.getByText("Operator grant saved; effective on next start."),
  ).toBeVisible();
});
for (const language of ["en", "ja"])
  for (const width of [320, 360, 390, 768, 1024, 1440])
    test(`task routes fit ${width}px in ${language}, mobile drawer restores focus`, async ({
      context,
      page,
    }, testInfo) => {
      await page.setViewportSize({ width, height: 900 });
      await setup(context, page, { language });
      const errors: string[] = [];
      page.on("pageerror", (e) => errors.push(e.message));
      for (const path of [
        "/play",
        "/worlds",
        `/worlds/${sid}`,
        "/people",
        "/people/teams/members",
        "/expeditions",
        "/timeline",
        "/timeline?room=" + rid,
        "/account",
        `/hosting/servers/${sid}`,
        `/hosting/servers/${sid}/console`,
        `/hosting/servers/${sid}/logs`,
        `/hosting/servers/${sid}/files`,
        `/hosting/servers/${sid}/members`,
        `/hosting/servers/${sid}/settings`,
      ]) {
        await page.goto(url(path));
        await expect(page.locator("h1")).toBeVisible();
        await expect(
          page.getByText(language === "en" ? "Loading…" : "読み込んでいます…", {
            exact: true,
          }),
        ).toHaveCount(0);
        await tick(page);
        const size = await page.evaluate(() => ({
          client: document.documentElement.clientWidth,
          scroll: document.documentElement.scrollWidth,
        }));
        expect(size.scroll, `${path} overflow`).toBeLessThanOrEqual(
          size.client,
        );
        const controls = await page
          .locator(
            "main input:not([type=checkbox]):not([type=file]),main select,main textarea",
          )
          .evaluateAll((nodes) =>
            nodes
              .filter((n) => (n as HTMLElement).getClientRects().length)
              .map((n) => ({
                bg: getComputedStyle(n).backgroundColor,
                fg: getComputedStyle(n).color,
              })),
          );
        for (const c of controls) expect(c.bg).not.toBe("rgb(255, 255, 255)");
        if ([390, 1440].includes(width))
          await page.screenshot({
            path: testInfo.outputPath(path.replaceAll("/", "_") + ".png"),
            fullPage: true,
          });
      }
      if (width <= 850) {
        const button = page.getByRole("button", {
          name: language === "en" ? "Open menu" : "メニューを開く",
          exact: true,
        });
        await button.click();
        await page.keyboard.press("Escape");
        await expect(button).toBeFocused();
        await expect(page.locator("aside")).toHaveAttribute("inert", "");
      }
      expect(errors).toEqual([]);
    });

test("server permissions and stopped/unsupported boundaries suppress unavailable mutations", async ({
  context,
  page,
}) => {
  const state = await setup(context, page);
  state.server.kind = "official";
  await page.goto(url(`/hosting/servers/${sid}/files`));
  await expect(
    page.getByText("File tools are available for custom servers only."),
  ).toBeVisible();
  await expect(page.locator("input[type=file]")).toHaveCount(0);
  state.server.kind = "custom";
  state.server.observed = state.server.desired = "running";
  // Same-hash navigation does not reload. Observe the real 15-second poll.
  const refreshed = page.waitForResponse((r) =>
    r.url().includes(`/api/v1/servers/${sid}`),
  );
  await tick(page, 7);
  expect((await (await refreshed).json()).server.kind).toBe("custom");
  await expect(
    page.getByRole("button", { name: "Create folder", exact: true }),
  ).toBeDisabled();
  await expect(page.locator("input[type=file]")).toBeDisabled();
  await page.goto(url(`/hosting/servers/${sid}/members`));
  await expect(
    page.getByRole("button", { name: "Grant operator", exact: true }),
  ).toBeDisabled();
  state.server.can_administer = false;
  await page.goto(url(`/hosting/servers/${sid}/settings`));
  await expect(
    page.getByText("Administrator permission is required for this page."),
  ).toBeVisible();
  await expect(
    page.getByRole("textbox", { name: "Name", exact: true }),
  ).toHaveCount(0);
  state.server.can_manage = false;
  await page.goto(url(`/hosting/servers/${sid}/console`));
  await expect(
    page.getByText("You no longer have permission to manage this server."),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Send command", exact: true }),
  ).toHaveCount(0);
  expect(
    state.commands.some((c: any) =>
      [
        "server_file_write",
        "server_install",
        "server_directory_create",
        "server_operator",
      ].includes(c.type),
    ),
  ).toBeFalsy();
});
test("confirmed file save does not report a stale read as a conflict or repeat completed actions", async ({
  context,
  page,
}) => {
  const state = await setup(context, page);
  await page.goto(url(`/hosting/servers/${sid}/files`));
  await tick(page);
  await page.getByRole("button", { name: "notes.txt", exact: true }).click();
  await tick(page);
  state.delays.server_file_read = 1500;
  await page.getByLabel("File text").fill("saved without a false conflict\n");
  await page.getByRole("button", { name: "Save file", exact: true }).click();
  await tick(page, 4);
  await expect(
    page.getByText("Saved notes.txt.", { exact: true }),
  ).toBeVisible();
  await expect(page.locator(".file-editor [role=alert]")).toHaveCount(0);
  await expect(page.locator(".route-progress")).toHaveCount(0);
});
test("a successful explorer save and new folder/file creation use the selected directory", async ({
  context,
  page,
}) => {
  const state = await setup(context, page);
  await page.goto(url(`/hosting/servers/${sid}/files`));
  await tick(page);
  await page
    .getByRole("button", { name: "Folder: documents", exact: true })
    .click();
  await tick(page);
  await page.getByRole("button", { name: "notes.txt", exact: true }).click();
  await tick(page);
  await page.getByLabel("File text").fill("enabled: false\n");
  await page.getByRole("button", { name: "Save file", exact: true }).click();
  await tick(page, 5);
  await expect(
    page.getByText("Saved documents/notes.txt.", { exact: true }),
  ).toBeVisible();
  expect(state.files.get("documents/notes.txt").text).toBe("enabled: false\n");
  await page.getByRole("button", { name: "Close editor" }).click();
  await page
    .getByRole("button", { name: "Create folder", exact: true })
    .click();
  await page.getByLabel("Folder name", { exact: true }).fill("data");
  await page
    .getByRole("dialog")
    .getByRole("button", { name: "Create", exact: true })
    .click();
  await tick(page, 5);
  expect(state.files.get("documents/data").kind).toBe("directory");
  await page.getByRole("button", { name: "New text file" }).click();
  await page.getByLabel("File name", { exact: true }).fill("new.yml");
  await page.getByLabel("File text").fill("new: true\n");
  await page.getByRole("button", { name: "Save file", exact: true }).click();
  await tick(page, 5);
  expect(
    state.commands.find(
      (c: any) =>
        c.type === "server_file_write" && c.path === "documents/new.yml",
    ).expected_sha256,
  ).toBeNull();
});
test("delayed actions disable repeated clicks and sleeping-server join shows true transfer result", async ({
  context,
  page,
}) => {
  const state = await setup(context, page);
  state.delays.server_join = 200;
  await page.goto(url("/worlds/" + sid));
  const join = page.locator(".world-detail-hero .hero-actions button");
  await expect(join).toHaveText("Wake and join");
  await join.click();
  await expect(join).toBeDisabled();
  await page.waitForTimeout(250);
  expect(
    state.commands.filter((c: any) => c.type === "server_join"),
  ).toHaveLength(1);
  const toast = page.locator(".toast");
  await expect(toast).toContainText("Join server · Workshop");
  await toast.getByRole("button", { name: "View details" }).click();
  await tick(page);
  await expect(page.getByRole("dialog")).toContainText("Transfer completed.");
  await expect(
    page.getByRole("dialog").getByRole("link", { name: "Open server" }),
  ).toHaveAttribute("href", `#/worlds/${sid}`);
});
test("building placement retains preview result and explicit confirmation without the old Activity tray", async ({
  context,
  page,
}) => {
  const state = await setup(context, page);
  state.server.kind = "official";
  await page.goto(url(`/worlds/${sid}/economy?tab=storage`));
  await page.getByRole("button", { name: "Place", exact: true }).click();
  await page
    .getByRole("dialog")
    .getByRole("button", { name: "Preview", exact: true })
    .click();
  await tick(page, 4);
  await expect(
    page.getByText("The placement area is clear.", { exact: true }),
  ).toBeVisible();
  await expect(page.locator(".job-tray")).toHaveCount(0);
  await page
    .getByRole("button", { name: "Place building", exact: true })
    .click();
  await page
    .getByRole("button", { name: "Confirm placement", exact: true })
    .click();
  expect(
    state.commands.filter((c: any) => c.type === "asset_place")[1].placement,
  ).toMatchObject({ preview: false, preview_hash: "preview-safe" });
});

for (const status of [403, 404]) {
  test(`current server ${status} removes contents, controls and dialogs`, async ({
    context,
    page,
  }) => {
    const state = await setup(context, page);
    await page.goto(url(`/hosting/servers/${sid}/files`));
    await tick(page);
    await expect(
      page.getByRole("button", { name: "notes.txt", exact: true }),
    ).toBeVisible();
    await page
      .getByRole("button", { name: "Create folder", exact: true })
      .click();
    await page.getByLabel("Folder name", { exact: true }).fill("private draft");
    state.failures[`/api/v1/servers/${sid}`] = status;
    await tick(page, 7);
    await expect(page.getByRole("dialog")).toHaveCount(0);
    await expect(
      page.getByRole("button", { name: "notes.txt", exact: true }),
    ).toHaveCount(0);
    await expect(
      page.getByRole("button", { name: "Create folder", exact: true }),
    ).toHaveCount(0);
    await expect(
      page.getByText("Previously loaded data is still shown."),
    ).toHaveCount(0);
    await expect(page.getByRole("alert")).toContainText(
      status === 403
        ? "You do not have permission to do this."
        : "The requested item could not be found.",
    );
  });
}

test("same-account new CSRF session clears private drafts, dialogs, jobs and delayed old responses", async ({
  context,
  page,
}) => {
  const state = await setup(context, page);
  await page.goto(url(`/hosting/servers/${sid}/console`));
  state.server.desired = state.server.observed = "running";
  await tick(page, 7);
  await page.getByLabel("Console command").fill("private command");
  state.delays.server_console = 1000;
  await page.getByRole("button", { name: "Send command", exact: true }).click();
  await expect
    .poll(() => state.commands.some((c: any) => c.type === "server_console"))
    .toBeTruthy();
  state.csrf = "second-login-same-account";
  await page.evaluate(() => window.dispatchEvent(new Event("focus")));
  await expect(page.getByLabel("Console command")).toHaveValue("");
  await page.waitForTimeout(1200);
  await expect(page.locator(".toast, .route-progress")).toHaveCount(0);
  await page.goto(url(`/hosting/servers/${sid}/files`));
  await expect(
    page.getByRole("button", { name: "New text file" }),
  ).toBeVisible();
  state.server.desired = state.server.observed = "stopped";
  await tick(page, 7);
  await page
    .getByRole("button", { name: "Create folder", exact: true })
    .click();
  await page.getByLabel("Folder name", { exact: true }).fill("secret folder");
  state.csrf = "third-login";
  await page.evaluate(() => window.dispatchEvent(new Event("focus")));
  await expect(page.getByRole("dialog")).toHaveCount(0);
});

test("401 during a delayed read removes private output immediately and late responses cannot restore it", async ({
  context,
  page,
}) => {
  const state = await setup(context, page);
  await page.goto(url("/timeline?room=" + rid));
  await expect(page.getByText("Message 10", { exact: true })).toBeVisible();
  await page.getByLabel("Message", { exact: true }).fill("secret draft");
  state.responseDelays["/api/v1/timeline"] = 1000;
  await tick(page, 4);
  state.failures["/api/v1/me"] = 401;
  await page.evaluate(() => window.dispatchEvent(new Event("focus")));
  await expect(
    page.getByRole("link", { name: "Join the community" }),
  ).toBeVisible();
  await expect(page.getByText("Message 10", { exact: true })).toHaveCount(0);
  await page.waitForTimeout(1200);
  await expect(
    page.locator(".timeline-feed, .timeline-composer, .toast"),
  ).toHaveCount(0);
  delete state.failures["/api/v1/me"];
  state.csrf = "fresh-login";
  await page.evaluate(() => window.dispatchEvent(new Event("focus")));
  await expect(page.getByLabel("Message", { exact: true })).toHaveValue("");
});

for (const status of [403, 404]) {
  test(`Timeline ${status} clears cached pages and composer on revisit`, async ({
    context,
    page,
  }) => {
    const state = await setup(context, page);
    await page.goto(url("/timeline?room=" + rid));
    await expect(page.getByText("Message 10", { exact: true })).toBeVisible();
    await page.getByLabel("Message", { exact: true }).fill("private text");
    await page.goto(url("/play"));
    state.failures["/api/v1/timeline"] = status;
    await page.goto(url("/timeline?room=" + rid));
    await expect(page.getByRole("alert")).toContainText(
      status === 403
        ? "You do not have permission to do this."
        : "The requested item could not be found.",
    );
    await expect(page.locator(".timeline-feed .message")).toHaveCount(0);
    await expect(page.getByLabel("Message", { exact: true })).toHaveCount(0);
    delete state.failures["/api/v1/timeline"];
    await page.getByRole("button", { name: "Retry", exact: true }).click();
    await expect(page.getByLabel("Message", { exact: true })).toHaveValue("");
  });
}

test("known-id refresh updates and removes older pages and prunes revoked rooms", async ({
  context,
  page,
}) => {
  const state = await setup(context, page);
  await page.goto(url("/timeline?room=" + rid));
  await page.getByRole("button", { name: "Load earlier items" }).click();
  await expect(page.getByText("Earlier 0", { exact: true })).toBeVisible();
  state.updates = [
    { ...state.message(0, ""), deleted_at: "2026-10-03T10:00:00Z" },
  ];
  state.removedIds = ["message:1"];
  await tick(page, 4);
  await expect(page.getByText("Earlier 0", { exact: true })).toHaveCount(0);
  await expect(page.locator('[data-item-id="message:0"]')).toContainText(
    "Deleted message",
  );
  await expect(page.locator('[data-item-id="message:1"]')).toHaveCount(0);
  expect(
    state.timelineRequests.some((q: string) =>
      new URLSearchParams(q).get("known")?.includes("message:0"),
    ),
  ).toBeTruthy();
  await page.getByLabel("Message", { exact: true }).fill("revoked room draft");
  state.rooms = state.rooms.filter((room: any) => room.id !== rid);
  await tick(page, 4);
  await expect(page.locator(".timeline-feed .message")).toHaveCount(0);
  await expect(page.getByLabel("Message", { exact: true })).toHaveCount(0);
});

test("late send on another route does not erase that room's draft", async ({
  context,
  page,
}) => {
  const state = await setup(context, page);
  await page.goto(url("/timeline?room=" + groupId));
  await page.getByLabel("Message", { exact: true }).fill("group draft");
  await page.goto(url("/timeline?room=" + rid));
  await page.getByLabel("Message", { exact: true }).fill("sent private draft");
  state.delays.message_send = 1000;
  await page.getByRole("button", { name: "Send", exact: true }).click();
  await expect
    .poll(() => state.commands.some((c: any) => c.type === "message_send"))
    .toBeTruthy();
  await page.goto(url("/timeline?room=" + groupId));
  await page.waitForTimeout(1200);
  await expect(page.getByLabel("Message", { exact: true })).toHaveValue(
    "group draft",
  );
  await expect(page.locator(".toast")).toHaveCount(0);
});

test("file reopen reauthorizes and a refused read removes the editor draft", async ({
  context,
  page,
}) => {
  const state = await setup(context, page);
  await page.goto(url(`/hosting/servers/${sid}/files`));
  await tick(page);
  await page.getByRole("button", { name: "notes.txt", exact: true }).click();
  await tick(page);
  await page.getByLabel("File text").fill("unsaved private text");
  await page.getByRole("button", { name: "Close editor" }).click();
  const reads = state.commands.filter(
    (c: any) => c.type === "server_file_read",
  ).length;
  state.failures["/api/v1/commands"] = 403;
  await page.getByRole("button", { name: "notes.txt", exact: true }).click();
  await expect(page.locator(".file-editor [role=alert]")).toContainText(
    "You do not have permission to do this.",
  );
  await expect(page.getByLabel("File text")).toHaveCount(0);
  await expect(
    page.getByRole("button", { name: "Save file", exact: true }),
  ).toHaveCount(0);
  delete state.failures["/api/v1/commands"];
  await page
    .locator(".file-editor")
    .getByRole("button", { name: "Retry", exact: true })
    .click();
  await tick(page);
  expect(
    state.commands.filter((c: any) => c.type === "server_file_read").length,
  ).toBeGreaterThan(reads);
  await expect(page.getByLabel("File text")).toHaveValue("server notes\n");
});

test("job detail rechecks authorization on reopen and never retains a refused result", async ({
  context,
  page,
}) => {
  const state = await setup(context, page);
  await page.goto(url("/timeline"));
  await page
    .locator('[data-item-id="job:completed"]')
    .getByRole("button", { name: "View details" })
    .click();
  await tick(page);
  await expect(page.getByRole("dialog")).toContainText("backup-123");
  await page.getByRole("button", { name: "Close", exact: true }).click();
  state.failures["/api/v1/jobs/completed"] = 403;
  await page
    .locator('[data-item-id="job:completed"]')
    .getByRole("button", { name: "View details" })
    .click();
  await expect(page.getByRole("dialog")).toContainText(
    "You do not have permission to do this.",
  );
  await expect(page.getByRole("dialog")).not.toContainText("backup-123");
});

test("host refusal remains visible and never becomes simulated sleeping-VM read success", async ({
  context,
  page,
}) => {
  const state = await setup(context, page);
  state.hostRefusals = true;
  await page.goto(url(`/hosting/servers/${sid}/files`));
  await expect(page.getByRole("alert")).toContainText(
    "You do not have permission to do this.",
  );
  await expect(
    page.getByRole("button", { name: "notes.txt", exact: true }),
  ).toHaveCount(0);
  await expect(
    page.getByRole("button", { name: "Create folder", exact: true }),
  ).toBeDisabled();
  await tick(page, 12);
  expect(
    state.commands.filter((c: any) => c.type === "server_files"),
  ).toHaveLength(1);
});

test("account change clears a file draft while harmless status refresh preserves it", async ({
  context,
  page,
}) => {
  const state = await setup(context, page);
  await page.goto(url(`/hosting/servers/${sid}/files`));
  await tick(page);
  await page.getByRole("button", { name: "notes.txt", exact: true }).click();
  await tick(page);
  await page.getByLabel("File text").fill("private draft for Alex");
  await page.getByLabel("File text").focus();
  state.server.players = 2;
  state.server.observed = state.server.desired = "running";
  await tick(page, 7);
  await expect(page.getByLabel("File text")).toHaveValue(
    "private draft for Alex",
  );
  await expect(page.getByLabel("File text")).toBeFocused();
  state.accountId = "another-account";
  await page.evaluate(() => window.dispatchEvent(new Event("focus")));
  await expect(page.getByLabel("File text")).toHaveCount(0);
  await tick(page);
  await page.getByRole("button", { name: "notes.txt", exact: true }).click();
  await tick(page);
  await expect(page.getByLabel("File text")).toHaveValue("server notes\n");
});

test("a pruned READ job requires explicit retry with a new admission", async ({
  context,
  page,
}) => {
  const state = await setup(context, page);
  state.pausedJobs = true;
  await page.goto(url(`/hosting/servers/${sid}/files`));
  await expect
    .poll(
      () => state.commands.filter((c: any) => c.type === "server_files").length,
    )
    .toBe(1);
  state.jobs.delete("fixture-1");
  await tick(page, 1);
  await expect(page.getByRole("alert")).toContainText(
    "The requested item could not be found.",
  );
  await tick(page, 8);
  expect(
    state.commands.filter((c: any) => c.type === "server_files"),
  ).toHaveLength(1);
  state.pausedJobs = false;
  await page.getByRole("button", { name: "Retry", exact: true }).click();
  await tick(page);
  expect(
    state.commands.filter((c: any) => c.type === "server_files"),
  ).toHaveLength(2);
  await expect(
    page.getByRole("button", { name: "notes.txt", exact: true }),
  ).toBeVisible();
});

test("temporary server failure labels retained state as stale and preserves typed settings", async ({
  context,
  page,
}) => {
  const state = await setup(context, page);
  await page.goto(url(`/hosting/servers/${sid}/settings`));
  await page.getByLabel("Name", { exact: true }).fill("unsaved server name");
  state.failures[`/api/v1/servers/${sid}`] = 503;
  await tick(page, 7);
  await expect(page.getByRole("alert")).toContainText(
    "Previously loaded data is still shown.",
  );
  await expect(page.getByLabel("Name", { exact: true })).toHaveValue(
    "unsaved server name",
  );
});

test("a reopened file draft retains an exact accessible editor label", async ({
  context,
  page,
}) => {
  await setup(context, page);
  await page.goto(url(`/hosting/servers/${sid}/files`));
  await tick(page);
  await page.getByRole("button", { name: "notes.txt", exact: true }).click();
  await tick(page);
  await page
    .getByLabel("File text", { exact: true })
    .fill("unsaved personal notes");
  await page.getByRole("button", { name: "Close editor" }).click();
  await page.getByRole("button", { name: "notes.txt", exact: true }).click();
  await tick(page);
  await expect(page.getByLabel("File text", { exact: true })).toHaveValue(
    "unsaved personal notes",
  );
});

test("text editor rejects oversized UTF-8 before submitting an impossible save", async ({
  context,
  page,
}) => {
  const state = await setup(context, page);
  await page.goto(url(`/hosting/servers/${sid}/files`));
  await tick(page);
  await page.getByRole("button", { name: "notes.txt", exact: true }).click();
  await tick(page);
  await page.getByLabel("File text", { exact: true }).fill("界".repeat(21846));
  await page.getByRole("button", { name: "Save file", exact: true }).click();
  await expect(page.locator(".file-editor [role=alert]")).toContainText(
    "Text files must be at most 64 KiB.",
  );
  expect(
    state.commands.filter(
      (command: any) => command.type === "server_file_write",
    ),
  ).toHaveLength(0);
});

test("sleeping Files require explicit opening and keep Minecraft stopped", async ({
  context,
  page,
}) => {
  const state = await setup(context, page);
  state.server.inspection = null;
  await page.goto(url(`/hosting/servers/${sid}/files`));
  await expect(
    page.getByRole("button", { name: "Open files", exact: true }),
  ).toBeVisible();
  expect(
    state.commands.some((c: any) => c.type === "server_files"),
  ).toBeFalsy();
  await page.getByRole("button", { name: "Open files", exact: true }).click();
  await expect(page.locator(".toast")).toContainText("Open files · Workshop");
  await tick(page, 10);
  await expect(
    page.getByRole("button", { name: "notes.txt", exact: true }),
  ).toBeVisible();
  expect(state.server.desired).toBe("stopped");
  expect(
    state.commands.filter((c: any) => c.type === "server_inspection" && c.open),
  ).toHaveLength(1);
  await page.getByRole("button", { name: "Close files", exact: true }).click();
  await expect(page.locator(".toast")).toContainText("Close files · Workshop");
  await tick(page, 10);
  await expect(
    page.getByRole("button", { name: "Open files", exact: true }),
  ).toBeVisible();
  expect(
    state.commands.some((c: any) => c.type === "server_start"),
  ).toBeFalsy();
});

test("owner membership is immutable and an unverified identity has no native OP controls", async ({
  context,
  page,
}) => {
  const state = await setup(context, page);
  state.server.members[0].is_owner = true;
  state.server.members[0].role = "administrator";
  state.server.members[0].minecraft_identity = {
    ready: false,
    reason: {
      id: "text.link_and_verify_a_java_account_before_changing_minecraft_op",
      params: {},
    },
  };
  await page.goto(url(`/hosting/servers/${sid}/members`));
  await expect(
    page.getByText("Owner · Administrator", { exact: true }),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Remove member", exact: true }),
  ).toHaveCount(0);
  await expect(
    page.getByRole("button", { name: "Save role", exact: true }),
  ).toHaveCount(0);
  await expect(
    page.getByRole("button", { name: "Grant operator", exact: true }),
  ).toHaveCount(0);
  await expect(page.getByText(/Link and verify a Java account/)).toBeVisible();
});
