import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import path from "node:path";
const web = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "../web",
);
const args = process.argv.slice(2);
// Existing callers continue to check the real dedicated integration service.
// Fixtures are an explicit, separate frontend contract lane.
const fixture = args[0] === "--fixture";
if (fixture || args[0] === "--integration") args.shift();
const run = spawnSync(
  process.execPath,
  [
    path.join(web, "node_modules/@playwright/test/cli.js"),
    "test",
    `--project=${fixture ? "fixture" : "integration"}`,
    ...args,
  ],
  { cwd: web, stdio: "inherit" },
);
if (run.error) throw run.error;
process.exitCode = run.status ?? 1;
