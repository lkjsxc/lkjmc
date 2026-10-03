import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import path from "node:path";
const web = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "../web",
);
// All pages and API calls are fulfilled by the scoped fixture. No live service,
// browser-session file, credentials, local port or production access is used.
const run = spawnSync(
  process.execPath,
  [
    path.join(web, "node_modules/@playwright/test/cli.js"),
    "test",
    "tests/ux.spec.ts",
    ...process.argv.slice(2),
  ],
  { cwd: web, stdio: "inherit" },
);
if (run.error) throw run.error;
process.exitCode = run.status ?? 1;
