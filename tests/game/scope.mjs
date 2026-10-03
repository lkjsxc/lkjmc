// Protocol fixtures may alter combat and readiness state. Require a dedicated
// loopback-only test database, never the shared developer database or production.
import fs from "node:fs/promises";
import assert from "node:assert/strict";
import path from "node:path";
import { execFile } from "node:child_process";
import { promisify } from "node:util";

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
// CI has a private PostgreSQL process, with no Docker/host socket. Both paths
// use exactly the same database guard; credentials never become process arguments.
export async function protocolSql(root, sql) {
  const config = JSON.parse(await fs.readFile(path.join(root, ".local/dev.json"), "utf8"));
  const database = checkedDatabase(config);
  if (!process.env.LKJMC_TEST_PSQL) {
    return promisify(execFile)("docker", ["exec", "lkjmc-rebuild-dev-postgres", "psql", "-U", "lkjmc", "-d", database, "-X", "-q", "-v", "ON_ERROR_STOP=1", "-c", sql]);
  }
  const url = new URL(config.database_url);
  const env = Object.fromEntries(Object.entries(process.env).filter(([name]) => !name.startsWith("PG")));
  return promisify(execFile)(process.env.LKJMC_TEST_PSQL, ["-X", "-q", "-v", "ON_ERROR_STOP=1", "-c", sql], {
    env: { ...env, PGHOST: url.hostname, PGPORT: url.port, PGDATABASE: database,
      PGUSER: url.username, PGPASSWORD: decodeURIComponent(url.password) },
  });
}
