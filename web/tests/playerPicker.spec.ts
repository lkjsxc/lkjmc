import { test, expect, type Page } from "@playwright/test";
import { mountFixture } from "./fixture.mjs";
const url = "https://ux.fixture/#/timeline";
async function tick(page: Page) {
  await page.clock.fastForward(300);
  await page.waitForTimeout(60);
}
async function setup(context: any, page: Page, width = 1440) {
  await page.setViewportSize({ width, height: 900 });
  const state = await mountFixture(context);
  await page.clock.install({ time: new Date("2026-10-03T10:00:00Z") });
  await page.goto(url);
  await page
    .getByRole("button", { name: "New private conversation", exact: true })
    .click();
  return state;
}
for (const width of [390, 1440]) {
  test(`suggestions appear before name search and can be selected without typing at ${width}px`, async ({
    context,
    page,
  }) => {
    const state = await setup(context, page, width);
    await tick(page);
    const dialog = page.getByRole("dialog");
    const search = dialog.getByRole("combobox", {
      name: "Player",
      exact: true,
    });
    await expect(search).toHaveValue("");
    const candidate = dialog.getByRole("option", { name: /Bea/ });
    await expect(candidate).toBeVisible();
    expect((await candidate.boundingBox())!.y).toBeLessThan(
      (await search.boundingBox())!.y,
    );
    await candidate.click();
    await expect(search).toHaveValue("Bea");
    await dialog
      .getByRole("button", { name: "Open conversation", exact: true })
      .click();
    expect(state.commands.find((c: any) => c.type === "direct_room")).toEqual({
      type: "direct_room",
      target: "bea",
    });
    await expect(page.getByRole("dialog")).toHaveCount(0);
  });
}

test("manual search is optional and typing never submits a raw player name", async ({
  context,
  page,
}) => {
  const state = await setup(context, page);
  const dialog = page.getByRole("dialog");
  const search = dialog.getByRole("combobox", { name: "Player", exact: true });
  await search.fill("Not selected");
  await dialog
    .getByRole("button", { name: "Open conversation", exact: true })
    .click();
  await expect(dialog.getByRole("alert")).toContainText(
    "Choose a player from the search results",
  );
  expect(
    state.commands.filter((c: any) => c.type === "direct_room"),
  ).toHaveLength(0);
  await search.fill("Bea");
  await tick(page);
  await expect(dialog.getByRole("option", { name: /Bea/ })).toBeVisible();
  await search.press("Enter");
  await expect(search).toHaveValue("Bea");
  await dialog
    .getByRole("button", { name: "Open conversation", exact: true })
    .click();
  expect(
    state.commands.filter((c: any) => c.type === "direct_room"),
  ).toHaveLength(1);
});

test("late suggestion responses cannot replace a new query or a selected identity", async ({
  context,
  page,
}) => {
  await mountFixture(context);
  await page.clock.install({ time: new Date("2026-10-03T10:00:00Z") });
  const pending: any[] = [];
  await context.route("**/api/v1/players?*", async (route) => {
    const query = new URL(route.request().url()).searchParams.get("q");
    if (!query) {
      pending.push(route);
      return;
    }
    await route.fulfill({
      json: { players: [{ id: "bea", name: "Bea", rank: "Member" }] },
    });
  });
  await page.goto(url);
  await page
    .getByRole("button", { name: "New private conversation", exact: true })
    .click();
  await expect.poll(() => pending.length).toBe(1);
  const dialog = page.getByRole("dialog");
  const search = dialog.getByRole("combobox", { name: "Player", exact: true });
  await search.fill("Bea");
  await tick(page);
  await dialog.getByRole("option", { name: /Bea/ }).click();
  await pending[0]
    .fulfill({
      json: {
        players: [{ id: "old", name: "Stale private result", rank: "Member" }],
      },
    })
    .catch(() => {});
  await expect(search).toHaveValue("Bea");
  await expect(dialog.getByText("Stale private result")).toHaveCount(0);
  await expect(
    dialog.locator('input[type="hidden"][name="target"]'),
  ).toHaveValue("bea");
});

test("a bounded suggestion list scrolls without expanding the mobile dialog offscreen", async ({
  context,
  page,
}) => {
  await mountFixture(context);
  await page.setViewportSize({ width: 390, height: 700 });
  await context.route("**/api/v1/players?*", (route) =>
    route.fulfill({
      json: {
        players: Array.from({ length: 30 }, (_, n) => ({
          id: String(n),
          name: `Player ${n}`,
          rank: "Member",
        })),
      },
    }),
  );
  await page.goto(url);
  await page
    .getByRole("button", { name: "New private conversation", exact: true })
    .click();
  await expect(page.getByRole("option")).toHaveCount(30);
  const list = page.getByRole("listbox");
  const box = (await list.boundingBox())!;
  expect(box.height).toBeLessThanOrEqual(280);
  expect(await list.evaluate((e) => e.scrollHeight > e.clientHeight)).toBe(
    true,
  );
  const submit = page
    .getByRole("dialog")
    .getByRole("button", { name: "Open conversation", exact: true });
  await expect(submit).toBeInViewport();
  const search = page
    .getByRole("dialog")
    .getByRole("combobox", { name: "Player", exact: true });
  await search.focus();
  for (let n = 0; n < 19; n++) await search.press("ArrowDown");
  await expect(
    page.getByRole("option", { name: "Player 19 Member", exact: true }),
  ).toBeInViewport();
  await expect(search).toBeFocused();
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= innerWidth,
    ),
  ).toBe(true);
});

test("selecting a suggestion clears native required-field validation after an empty submit", async ({
  context,
  page,
}) => {
  const state = await setup(context, page);
  await tick(page);
  const dialog = page.getByRole("dialog");
  const search = dialog.getByRole("combobox", { name: "Player", exact: true });
  await dialog
    .getByRole("button", { name: "Open conversation", exact: true })
    .click();
  expect(state.commands.some((c: any) => c.type === "direct_room")).toBe(false);
  await dialog.getByRole("option", { name: /Bea/ }).click();
  expect(
    await search.evaluate((e) => (e as HTMLInputElement).checkValidity()),
  ).toBe(true);
  await dialog
    .getByRole("button", { name: "Open conversation", exact: true })
    .click();
  await expect(page.getByRole("dialog")).toHaveCount(0);
  expect(
    state.commands.filter((c: any) => c.type === "direct_room"),
  ).toHaveLength(1);
});
