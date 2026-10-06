import { test, expect } from "@playwright/test";
import { mountFixture, sid } from "./fixture.mjs";
const url = (path: string) => "https://ux.fixture/#" + path;

test("SMP offers both teleport directions with player-list-first selection", async ({
  context,
  page,
}) => {
  const state = await mountFixture(context);
  state.server.kind = "official";
  await page.goto(url(`/worlds/${sid}/world?tab=meetup`));
  for (const [label, here] of [
    ["Go to a player", false],
    ["Ask a player to come here", true],
  ] as const) {
    await page.getByRole("button", { name: label, exact: true }).click();
    const dialog = page.getByRole("dialog");
    await expect(dialog.getByRole("heading")).toHaveText(label);
    await dialog.getByRole("option", { name: /Bea/ }).click();
    await dialog
      .getByRole("button", { name: "Send request", exact: true })
      .click();
    await expect(page.getByRole("dialog")).toHaveCount(0);
    expect(
      state.commands.filter((c: any) => c.type === "teleport_request").at(-1),
    ).toEqual({ type: "teleport_request", here, target: "bea" });
  }
  await expect(
    page.getByRole("link", { name: "Teleport requests", exact: true }),
  ).toHaveAttribute("href", "#/play/invitations");
});

test("invitation decisions identify the direction and exact request before acceptance", async ({
  context,
  page,
}) => {
  const state = await mountFixture(context);
  const invitations = [false, true].map((teleport_here, index) => ({
    id: `request-${index}`,
    kind: "teleport",
    teleport_here,
    sender_name: "Bea",
    created_at: "2026-10-03T10:00:00Z",
    expires_at: "2026-10-03T10:02:00Z",
  }));
  await context.route("**/api/v1/history/invitations**", (route) =>
    route.fulfill({ json: { invitations } }),
  );
  await page.goto(url("/play/invitations"));
  const to = page
    .locator(".list-row")
    .filter({ hasText: "wants to teleport to you" });
  const here = page
    .locator(".list-row")
    .filter({ hasText: "asks you to teleport to them" });
  await expect(to).toBeVisible();
  await expect(here).toBeVisible();
  await expect(to).toContainText("Expires:");
  await expect(here).toContainText("Expires:");
  expect(
    state.commands.filter((c: any) => c.type === "invite_respond"),
  ).toHaveLength(0);
  await here.getByRole("button", { name: "Accept", exact: true }).click();
  await expect
    .poll(() => state.commands.filter((c: any) => c.type === "invite_respond"))
    .toEqual([{ type: "invite_respond", id: "request-1", accept: true }]);
});
