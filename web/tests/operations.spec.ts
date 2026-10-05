import { test, expect } from "@playwright/test";
import { mountFixture, sid } from "./fixture.mjs";
import english from "../../locales/en.json" with { type: "json" };
import japanese from "../../locales/ja.json" with { type: "json" };

const url = (path: string) => "https://ux.fixture/#" + path;

for (const language of ["en", "ja"] as const) {
  for (const width of [390, 1440]) {
    test(`operations render structured messages and page filters at ${width}px in ${language}`, async ({
      context,
      page,
    }) => {
      const labels = language === "en" ? english : japanese;
      const state = await mountFixture(context, { language, width });
      const errors: string[] = [];
      page.on("pageerror", (error) => errors.push(error.message));
      const base = {
        server_id: sid,
        server_name: "Workshop",
        kind: "server.start",
        created_at: "2026-10-04T00:00:00Z",
        updated_at: "2026-10-04T00:05:00Z",
      };
      const active = {
        ...base,
        id: "active-operation",
        state: "queued",
        progress: { message: { id: "text.waiting_to_resume", params: {} } },
      };
      const failed = {
        ...base,
        id: "failed-operation",
        state: "failed",
        error: { id: "error.internal", params: { reference: "ops-reference" } },
      };
      const uncertain = {
        ...base,
        kind: "server.console",
        id: "uncertain-operation",
        state: "delivery_unknown",
        progress: {},
        result: {},
        gets: 2,
      };
      state.jobs.set(uncertain.id, uncertain);
      const requested: string[] = [];
      await context.route(
        /\/api\/v1\/admin\/operations(?:\?.*)?$/,
        async (route) => {
          const request = new URL(route.request().url());
          const filter = request.searchParams.get("filter") ?? "active";
          requested.push(request.search);
          const operations =
            filter === "active"
              ? [active]
              : filter === "failed"
                ? [failed, uncertain]
                : request.searchParams.has("cursor")
                  ? [
                      {
                        ...base,
                        id: "malformed-operation",
                        state: "failed",
                        error: { arbitrary: "diagnostic" },
                      },
                    ]
                  : [
                      {
                        ...base,
                        id: "completed-operation",
                        state: "succeeded",
                      },
                    ];
          await route.fulfill({
            json: {
              filter,
              operations,
              counts: { active: 1, failed: 2, history: 3 },
              next_cursor:
                filter === "history" && !request.searchParams.has("cursor")
                  ? "page/older:cursor"
                  : null,
            },
          });
        },
      );
      await context.route("**/api/v1/view/admin?section=overview", (route) =>
        route.fulfill({
          json: { counts: { operations: 1, reports: 0, backups: 0 } },
        }),
      );
      await page.goto(url("/admin"));
      await page
        .getByRole("link", { name: new RegExp(labels["text.operations"]) })
        .first()
        .click();
      await expect(page.locator("h1")).toHaveText(labels["text.operations"]);
      await expect(
        page
          .getByRole("main")
          .getByText(labels["text.waiting_to_resume"], { exact: true }),
      ).toBeVisible();
      const filters = page.getByRole("navigation", {
        name: labels["text.operations"],
        exact: true,
      });
      await expect(
        filters.getByRole("link", {
          name: labels["text.operations_in_progress"] + " · 1",
          exact: true,
        }),
      ).toHaveAttribute("aria-current", "page");
      await filters
        .getByRole("link", {
          name: labels["text.operations_failed_or_uncertain"] + " · 2",
          exact: true,
        })
        .click();
      await expect(
        page.getByText(
          labels["error.internal"].replace("{reference}", "ops-reference"),
          { exact: true },
        ),
      ).toBeVisible();
      const uncertainRow = page.locator(".list-row").filter({
        has: page.getByText(labels["text.delivery_unknown"], { exact: true }),
      });
      await uncertainRow
        .getByRole("button", { name: labels["text.view_details"], exact: true })
        .click();
      await expect(
        page
          .getByRole("dialog")
          .getByText(labels["text.delivery_unknown"], { exact: true }),
      ).toBeVisible();
      await expect(
        page
          .getByRole("dialog")
          .getByRole("button", { name: labels["text.retry"], exact: true }),
      ).toHaveCount(0);
      await page.keyboard.press("Escape");
      await filters
        .getByRole("link", {
          name: labels["text.operations_history"] + " · 3",
          exact: true,
        })
        .click();
      await page
        .getByRole("link", { name: labels["text.older"], exact: true })
        .click();
      await expect(page).toHaveURL(
        /filter=history&cursor=page%2Folder%3Acursor$/,
      );
      await expect(
        page.getByRole("link", { name: labels["text.latest"], exact: true }),
      ).toBeVisible();
      await expect(page.locator(".list-row")).toContainText(
        language === "en" ? "Reference" : "参照",
      );
      await page.goBack();
      await expect(
        page.getByRole("link", { name: labels["text.older"], exact: true }),
      ).toBeVisible();
      expect(
        requested.some((request) => request.includes("filter=failed")),
      ).toBe(true);
      expect(state.commands).toEqual([]);
      expect(errors).toEqual([]);
      expect(
        await page.evaluate(
          () => document.documentElement.scrollWidth <= innerWidth,
        ),
      ).toBe(true);
    });
  }
}
