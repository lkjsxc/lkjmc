import { defineConfig } from "@playwright/test";
export default defineConfig({
  testDir: "tests",
  fullyParallel: false,
  workers: 1,
  outputDir: "../.local/browser-results",
  use: { screenshot: "only-on-failure" },
  projects: [
    {
      name: "fixture",
      testMatch: [
        "ux.spec.ts",
        "player.spec.ts",
        "teams.spec.ts",
        "operations.spec.ts",
        "directNavigation.spec.ts",
        "filesEntry.spec.ts",
      ],
      use: { baseURL: "https://ux.fixture" },
    },
    {
      name: "integration",
      testMatch: "journey.spec.ts",
      use: { baseURL: "http://127.0.0.1:18091" },
    },
  ],
  reporter: "list",
});
