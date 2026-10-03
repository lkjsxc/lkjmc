import { defineConfig } from "@playwright/test";
export default defineConfig({
  testDir: "tests",
  testMatch: "ux.spec.ts",
  fullyParallel: false,
  workers: 1,
  outputDir: "../.local/browser-results",
  use: { baseURL: "https://ux.fixture", screenshot: "only-on-failure" },
  reporter: "list",
});
