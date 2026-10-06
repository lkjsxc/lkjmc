import fs from "node:fs";
import { test, expect, type Page } from "@playwright/test";
import { mountFixture, aid, sid, groupId } from "./fixture.mjs";
const pid = "00000000-0000-0000-0000-000000000052";
const url = (path: string) => "https://ux.fixture/#" + path;
const labels = Object.fromEntries(
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
async function tick(page: Page, count = 7) {
  for (let n = 0; n < count; n++) {
    await page.clock.fastForward(2500);
    await page.waitForTimeout(35);
  }
}
async function setup(context: any, page: Page, language = "en") {
  const state = await mountFixture(context, { language });
  await page.clock.install({ time: new Date("2026-10-03T10:00:00Z") });
  state.party = null;
  state.partyDelay = 0;
  state.newParty = (name = "Our party") => ({
    id: pid,
    room_id: groupId,
    name,
    leader: aid,
    members: [
      { account_id: aid, name: "Alex", ready: false },
      { account_id: "bea", name: "Bea", ready: false },
    ],
  });
  await context.route("**/api/v1/view/social?section=parties", (route: any) =>
    route.fulfill({ json: { party: state.party } }),
  );
  await context.route("**/api/v1/commands", async (route: any) => {
    const c = route.request().postDataJSON().command;
    if (!c.type.startsWith("party_")) return route.fallback();
    state.commands.push(c);
    if (state.partyDelay)
      await new Promise((resolve) => setTimeout(resolve, state.partyDelay));
    if (c.type === "party_create")
      state.party ??= state.newParty(labels[language]["text.party"]);
    else if (!state.party || c.party !== state.party.id)
      return route.fulfill({
        status: 403,
        json: { error: { message: { id: "error.forbidden", params: {} } } },
      });
    if (c.type === "party_rename") state.party.name = c.name;
    if (c.type === "party_transfer") state.party.leader = c.target;
    if (c.type === "party_ready")
      state.preparation.participants.find(
        (p: any) => p.account_id === aid,
      ).ready = c.ready;
    if (c.type === "party_leave") state.party = null;
    return route.fulfill({
      json: { result: { party_id: state.party?.id, room_id: groupId } },
    });
  });
  return state;
}
for (const language of ["en", "ja"]) {
  test(`party creation asks for no name and opens the single membership in ${language}`, async ({
    context,
    page,
  }) => {
    const state = await setup(context, page, language);
    await page.setViewportSize({ width: 390, height: 844 });
    state.partyDelay = 150;
    await page.goto(url("/people/parties"));
    const create = page.getByRole("button", {
      name: labels[language]["text.create_party"],
      exact: true,
    });
    await create.click();
    await expect(create).toBeDisabled();
    await expect(page.getByRole("dialog")).toHaveCount(0);
    await expect(page.locator(".party-workspace")).toBeVisible();
    expect(
      state.commands.filter((c: any) => c.type === "party_create"),
    ).toEqual([{ type: "party_create" }]);
    await expect(page.locator(".party-workspace .team-member-row")).toHaveCount(
      2,
    );
    await expect(page.locator(".breadcrumbs [aria-current=page]")).toHaveText(
      labels[language]["text.party"],
    );
    await expect(page.locator(".section-nav")).toHaveCount(0);
    await expect(page.locator(".party-workspace .team-member-list")).toHaveCSS(
      "list-style-type",
      "none",
    );
    await expect(
      page.locator(".party-workspace .team-member-row").first(),
    ).toHaveCSS("display", "flex");

    await expect(
      page.locator(".party-workspace h2,.party-workspace h3,.page-heading"),
    ).toHaveCount(0);
    await expect(page.locator('a[href$="/parties/ready"]')).toHaveCount(0);
    expect(
      await page.evaluate(
        () => document.documentElement.scrollWidth <= innerWidth,
      ),
    ).toBe(true);
  });
}
test("renaming preserves the draft across polling and submits the expected party ID", async ({
  context,
  page,
}) => {
  const state = await setup(context, page);
  state.party = state.newParty();
  await page.goto(url("/people/parties"));
  await page.getByRole("button", { name: "Party name", exact: true }).click();
  const input = page
    .getByRole("dialog")
    .getByLabel("Party name", { exact: true });
  await expect(input).toHaveValue("Our party");
  await input.fill("新しい名前");
  await tick(page);
  await expect(input).toHaveValue("新しい名前");
  await page
    .getByRole("dialog")
    .getByRole("button", { name: "Save", exact: true })
    .click();
  await expect(page.getByRole("dialog")).toHaveCount(0);
  await expect(page.locator(".breadcrumbs [aria-current=page]")).toHaveText(
    "新しい名前",
  );
  await expect(page).toHaveTitle("新しい名前 · lkjmc");
  expect(state.commands.at(-1)).toEqual({
    type: "party_rename",
    party: pid,
    name: "新しい名前",
  });
  await page.reload();
  await expect(page.locator(".party-workspace .team-member-row")).toHaveCount(
    2,
  );
  await expect(
    page.getByRole("link", { name: "Chat room", exact: true }),
  ).toHaveAttribute("href", "#/timeline?room=" + groupId);
});
for (const action of ["Party name", "Invite member"]) {
  test(`${action} dialog closes when leadership or membership changes`, async ({
    context,
    page,
  }) => {
    const state = await setup(context, page);
    state.party = state.newParty();
    await page.goto(url("/people/parties"));
    await page.getByRole("button", { name: action, exact: true }).click();
    await expect(page.getByRole("dialog")).toBeVisible();
    state.party.leader = "bea";
    await tick(page);
    await expect(page.getByRole("dialog")).toHaveCount(0);
    await expect(
      page.getByRole("button", { name: action, exact: true }),
    ).toHaveCount(0);
    state.party.leader = aid;
    await tick(page);
    await page.getByRole("button", { name: action, exact: true }).click();
    state.party = null;
    await tick(page);
    await expect(page.getByRole("dialog")).toHaveCount(0);
    await expect(
      page.getByRole("button", { name: "Create party", exact: true }),
    ).toBeVisible();
    expect(state.commands).toEqual([]);
  });
}
test("members see no leader controls or expedition consent and can leave only their displayed party", async ({
  context,
  page,
}) => {
  const state = await setup(context, page);
  state.party = { ...state.newParty(), leader: "bea" };
  await page.goto(url("/people/parties/members"));
  await expect(page.locator(".party-workspace .team-member-row")).toHaveCount(
    2,
  );
  await expect(
    page.getByRole("button", {
      name: /Party name|Invite member|Make leader|Ready|Cancel ready/,
    }),
  ).toHaveCount(0);
  await page.getByRole("button", { name: "Leave party", exact: true }).click();
  await expect(
    page.getByRole("button", { name: "Create party", exact: true }),
  ).toBeVisible();
  expect(state.commands.at(-1)).toEqual({ type: "party_leave", party: pid });
});
test("SMP expedition preparation owns consent and no longer sends users to a common party readiness page", async ({
  context,
  page,
}) => {
  const state = await setup(context, page);
  state.party = state.newParty();
  state.server.kind = "official";
  state.preparation.party_id = pid;
  state.preparation.participants[0].ready = false;
  state.preparation.can_prepare = false;
  await page.goto(url(`/worlds/${sid}/expeditions`));
  await expect(page.locator(".expedition-hero,.world-art")).toHaveCount(0);
  await expect(page.locator('a[href$="/parties/ready"]')).toHaveCount(0);
  await page.getByRole("button", { name: "Ready", exact: true }).click();
  await expect(
    page.getByRole("button", { name: "Cancel ready status", exact: true }),
  ).toBeVisible();
  expect(state.commands.at(-1)).toEqual({
    type: "party_ready",
    party: pid,
    ready: true,
  });
  await page
    .getByRole("button", { name: "Cancel ready status", exact: true })
    .click();
  await expect(
    page.getByRole("button", { name: "Ready", exact: true }),
  ).toBeVisible();
  expect(state.commands.at(-1)).toEqual({
    type: "party_ready",
    party: pid,
    ready: false,
  });
  await page.goto(url("/people/parties"));
  await expect(
    page.getByRole("button", { name: /^Ready$|Cancel ready/ }),
  ).toHaveCount(0);
});
