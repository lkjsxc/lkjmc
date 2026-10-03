// Protocol fixtures may alter combat and readiness state. Require a dedicated
// loopback-only test database, never the shared developer database or production.
import fs from "node:fs/promises";
import assert from "node:assert/strict";
import path from "node:path";

export function checkedDatabase(config) {
  assert.equal(config.scope, "isolated-protocol-test", "Prepare a dedicated protocol-test config first");
  const url = new URL(config.database_url);
  assert(["postgres:", "postgresql:"].includes(url.protocol));
  assert(["127.0.0.1", "localhost", "[::1]"].includes(url.hostname), "The fixture database must be loopback-only");
  assert.equal(url.port, "16543");
  assert.equal(url.username, "lkjmc");
  const database = url.pathname.slice(1);
  assert(/^lkjmc_test_protocol_[a-z0-9_]{8,48}$/.test(database), "Do not run state-changing fixtures against a non-test database");
  assert.equal(database, config.test_database);
  assert(!url.search && !url.hash);
  return database;
}
export async function protocolDatabase(root) {
  return checkedDatabase(JSON.parse(await fs.readFile(path.join(root, ".local/dev.json"), "utf8")));
}
