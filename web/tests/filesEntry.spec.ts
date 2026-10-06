import { test, expect, type Page } from "@playwright/test";
import { mountFixture, sid } from "./fixture.mjs";
const url = (path: string) => "https://ux.fixture/#" + path;
const files = `/hosting/servers/${sid}/files`;
async function tick(page: Page, count = 7) {
  for (let i = 0; i < count; i++) {
    await page.clock.fastForward(2500);
    await page.waitForTimeout(40);
  }
}
async function setup(context: any, page: Page) {
  const state = await mountFixture(context);
  await page.clock.install({ time: new Date("2026-10-03T10:00:00Z") });
  state.server.inspection = null;
  state.server.status.actions.files = {
    allowed: false,
    reason: "files_closed",
  };
  // Match Core's projection as the durable inspection changes state.
  await context.route("**/api/v1/servers/**", async (route: any) => {
    state.server.status.actions.files =
      state.server.inspection?.state === "ready"
        ? { allowed: true, reason: null }
        : {
            allowed: false,
            reason: state.server.inspection ? "maintenance" : "files_closed",
          };
    await route.fallback();
  });
  return state;
}
const opens = (state: any) =>
  state.commands.filter((c: any) => c.type === "server_inspection" && c.open);

test("automatic Files admission is once per visit; Close and expiry do not wake it again", async ({
  context,
  page,
}) => {
  const state = await setup(context, page);
  await page.goto(url(files));
  await expect.poll(() => opens(state).length).toBe(1);
  await tick(page, 10);
  await expect(
    page.getByRole("link", { name: "notes.txt", exact: true }),
  ).toBeVisible();
  await page.getByRole("button", { name: "Close files", exact: true }).click();
  await tick(page, 15);
  await expect(
    page.getByRole("button", { name: "Open files", exact: true }),
  ).toBeVisible();
  expect(opens(state)).toHaveLength(1);
  await page.goto(url("/hosting/servers"));
  await page.goto(url(files));
  await expect.poll(() => opens(state).length).toBe(2);
  await tick(page, 10);
  state.server.inspection = null; // The host has ended the bounded inspection.
  await tick(page, 15);
  await expect(
    page.getByRole("button", { name: "Open files", exact: true }),
  ).toBeVisible();
  expect(opens(state)).toHaveLength(2);
  expect(state.commands.some((c: any) => c.type === "server_start")).toBe(
    false,
  );
  expect(state.server.desired).toBe("stopped");
});

test("preparation transport failure needs explicit retry with the same admission key", async ({
  context,
  page,
}) => {
  const state = await setup(context, page);
  const requests: any[] = [];
  await context.route("**/api/v1/commands", async (route) => {
    const body = route.request().postDataJSON();
    if (body.command.type === "server_inspection" && body.command.open) {
      requests.push(body);
      if (requests.length === 1)
        return route.fulfill({
          status: 503,
          json: {
            error: {
              message: { id: "text.request_failed_0", params: { "0": 503 } },
            },
          },
        });
    }
    return route.fallback();
  });
  await page.goto(url(files));
  await expect(page.getByRole("alert")).toContainText("503");
  await tick(page, 15);
  expect(requests).toHaveLength(1);
  await page.getByRole("button", { name: "Retry", exact: true }).click();
  await tick(page, 10);
  await expect(
    page.getByRole("link", { name: "notes.txt", exact: true }),
  ).toBeVisible();
  expect(requests).toHaveLength(2);
  expect(requests[1].request_id).toBe(requests[0].request_id);
  expect(opens(state)).toHaveLength(1);
});

test("failed preparation status read resumes its accepted job without another opening request", async ({
  context,
  page,
}) => {
  const state = await setup(context, page);
  state.failNext.job = "Status temporarily unavailable";
  await page.goto(url(files));
  await expect(page.getByRole("alert")).toBeVisible();
  expect(opens(state)).toHaveLength(1);
  const job = state.server.inspection.id;
  await page.getByRole("button", { name: "Retry", exact: true }).click();
  await tick(page, 10);
  await expect(
    page.getByRole("link", { name: "notes.txt", exact: true }),
  ).toBeVisible();
  expect(opens(state)).toHaveLength(1);
  expect(
    state.jobGets.filter((id: string) => id === job).length,
  ).toBeGreaterThan(1);
  await expect(page.getByRole("alert")).toHaveCount(0);
});

test("Files waits for a fresh stable observation before automatically preparing", async ({
  context,
  page,
}) => {
  const state = await setup(context, page);
  state.server.status.observation_fresh = false;
  await page.goto(url(files));
  await tick(page, 7);
  expect(opens(state)).toHaveLength(0);
  state.server.status.observation_fresh = true;
  await tick(page, 7);
  await expect.poll(() => opens(state).length).toBe(1);
  await tick(page, 7);
  await expect(
    page.getByRole("link", { name: "notes.txt", exact: true }),
  ).toBeVisible();
});

test("official stopped worlds and revoked administration never receive an inspection request", async ({
  context,
  page,
}) => {
  const state = await setup(context, page);
  state.server.kind = "official";
  await page.goto(url(files));
  await expect(page.locator("main")).toContainText(
    "File tools are available for custom servers only.",
  );
  await tick(page, 7);
  expect(opens(state)).toHaveLength(0);
  state.server.kind = "custom";
  state.server.can_administer = false;
  await page.reload();
  await tick(page, 7);
  expect(opens(state)).toHaveLength(0);
  await expect(page.locator(".files-workspace")).toHaveCount(0);
});

for (const width of [390, 1440]) {
  test(`hosting and file metadata cells are native link targets at ${width}px`, async ({
    context,
    page,
  }) => {
    await page.setViewportSize({ width, height: 900 });
    const state = await mountFixture(context);
    await page.clock.install({ time: new Date("2026-10-03T10:00:00Z") });
    await page.goto(url("/hosting/servers"));
    const serverRow = page
      .locator(".server-row")
      .filter({ hasText: "Workshop" });
    const target = serverRow.locator(".inventory-allocation");
    const box = (await target.boundingBox())!;
    await page.mouse.click(box.x + box.width - 4, box.y + box.height - 4);
    await expect(page).toHaveURL(url(`/hosting/servers/${sid}`));
    await page.goto(url(files));
    await tick(page);
    const row = page.locator(".files-table tbody tr").filter({
      has: page.getByRole("link", { name: "documents", exact: true }),
    });
    const cell = (await row.locator("td").nth(1).boundingBox())!;
    await page.mouse.click(cell.x + cell.width - 3, cell.y + cell.height / 2);
    await expect(page).toHaveURL(url(files + "?path=documents"));
    await tick(page);
    const file = page.getByRole("link", { name: "notes.txt", exact: true });
    await file.focus();
    await page.keyboard.press("Enter");
    await tick(page);
    await expect(page.getByLabel("File text", { exact: true })).toHaveValue(
      "enabled: true\n",
    );
    expect(state.commands.some((c: any) => c.type === "server_start")).toBe(
      false,
    );
  });
}

test("retrying a failed Close retries closing and never changes it into an opening", async ({
  context,
  page,
}) => {
  const state = await setup(context, page);
  await page.goto(url(files));
  await tick(page, 10);
  await expect(
    page.getByRole("button", { name: "Close files", exact: true }),
  ).toBeEnabled();
  state.failNext.server_inspection = {
    id: "text.request_failed_0",
    params: { "0": 503 },
  };
  await page.getByRole("button", { name: "Close files", exact: true }).click();
  await expect(page.getByRole("alert")).toContainText("503");
  await page.getByRole("button", { name: "Retry", exact: true }).click();
  await tick(page, 10);
  await expect(
    page.getByRole("button", { name: "Open files", exact: true }),
  ).toBeVisible();
  expect(
    state.commands
      .filter((c: any) => c.type === "server_inspection")
      .map((c: any) => c.open),
  ).toEqual([true, false, false]);
});

test("folder navigation during admission preserves the one pending preparation", async ({
  context,
  page,
}) => {
  const state = await setup(context, page);
  state.delays.server_inspection = 150;
  await page.goto(url(files));
  await expect.poll(() => opens(state).length).toBe(1);
  await page.evaluate((path) => {
    location.hash = path;
  }, files + "?path=documents");
  await page.waitForTimeout(200);
  await tick(page, 10);
  await expect(
    page.getByRole("link", { name: "notes.txt", exact: true }),
  ).toBeVisible();
  await expect(page.locator(".file-breadcrumbs")).toContainText("documents");
  await expect(page.getByRole("alert")).toHaveCount(0);
  expect(opens(state)).toHaveLength(1);
});

test("concurrent admission reuses another operator's session without reading their private job", async ({
  context,
  page,
}) => {
  const state = await setup(context, page);
  state.administrator = false;
  const foreignJob = "other-operator-inspection";
  await context.route("**/api/v1/commands", async (route) => {
    const c = route.request().postDataJSON().command;
    if (c.type !== "server_inspection") return route.fallback();
    state.commands.push(c);
    state.server.inspection = {
      id: foreignJob,
      actor: "another-operator",
      state: "opening",
      guest_ready: false,
    };
    return route.fulfill({
      json: {
        result: { job_id: foreignJob, inspection: state.server.inspection },
      },
    });
  });
  await page.goto(url(files));
  await expect.poll(() => opens(state).length).toBe(1);
  await tick(page);
  await expect(page.getByRole("alert")).toHaveCount(0);
  expect(state.jobGets).not.toContain(foreignJob);
  state.server.inspection = {
    ...state.server.inspection,
    state: "ready",
    guest_ready: true,
    expires_at: "2026-10-03T10:10:00Z",
  };
  await tick(page, 10);
  await expect(
    page.getByRole("link", { name: "notes.txt", exact: true }),
  ).toBeVisible();
  await expect(page.getByRole("alert")).toHaveCount(0);
  expect(state.jobGets).not.toContain(foreignJob);
  expect(opens(state)).toHaveLength(1);
});
