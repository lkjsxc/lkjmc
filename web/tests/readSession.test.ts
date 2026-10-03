import { ReadSession, type ReadTransport } from "../src/readSession.ts";
import {
  acceptIdentity,
  identityEpoch,
  identitySignal,
  assertIdentity,
  resetIdentity,
  PrivateCache,
} from "../src/identity.ts";

// The runner supplies node:test/assert. This typed suite needs no extra Node
// type package in the frontend's fixed dependency installation.
export function registerReadSessionTests(
  test: (name: string, run: () => void | Promise<void>) => void,
  assert: {
    equal(actual: unknown, expected: unknown): void;
    notEqual(actual: unknown, expected: unknown): void;
    deepEqual(actual: unknown, expected: unknown): void;
    throws(run: () => unknown, expected: { name: string }): void;
  },
) {
  const failure = (status: number) =>
    Object.assign(new Error(`HTTP ${status}`), { status });
  for (const status of [401, 403, 404]) {
    test(`READ ${status} drops output/job/key and only explicit retry admits fresh work`, async () => {
      let clock = 1,
        submits = 0,
        gets = 0;
      const keys: string[] = [];
      const transport: ReadTransport = {
        submit: async (_type, _values, requestId) => {
          keys.push(requestId);
          return { job_id: `job-${++submits}` };
        },
        job: async () => {
          if (++gets === 2) throw failure(status);
          return { state: "succeeded", result: { text: "private output" } };
        },
      };
      const read = new ReadSession("server_file_read", {
        id: "server",
        path: "notes.txt",
      });
      await read.advance(transport, () => clock);
      clock += 1500;
      await read.advance(transport, () => clock);
      assert.equal(read.snapshot().result?.text, "private output");
      read.retry();
      await read.advance(transport, () => clock);
      clock += 121000; // Includes pruned, two-minute ephemeral job 404.
      await read.advance(transport, () => clock);
      assert.equal(read.snapshot().result, undefined);
      assert.equal(read.snapshot().progress, undefined);
      assert.equal(read.snapshot().revoked, true);
      for (let i = 0; i < 3; i++)
        await read.advance(transport, () => clock, true);
      assert.equal(submits, 2);
      assert.equal(gets, 2);
      assert.equal(read.delay(clock, true), null);
      read.retry();
      await read.advance(transport, () => clock);
      assert.equal(submits, 3);
      assert.equal(read.snapshot().revoked, true); // Actions stay gated until permission succeeds.
      assert.notEqual(keys[1], keys[2]);
      clock += 1500;
      await read.advance(transport, () => clock);
      assert.equal(read.snapshot().result?.text, "private output");
      assert.equal(read.snapshot().revoked, false);
    });
  }

  test("transport/5xx retry resumes original job and reopens hide results until reauthorized", async () => {
    let clock = 1,
      submits = 0,
      gets = 0;
    const transport: ReadTransport = {
      submit: async () => ({ job_id: `job-${++submits}` }),
      job: async (id) => {
        if (++gets === 2) throw failure(503);
        assert.equal(id, `job-${submits}`);
        return { state: "succeeded", result: { text: "verified read" } };
      },
    };
    const read = new ReadSession("server_file_read", { id: "server" });
    await read.advance(transport, () => clock);
    clock += 1500;
    await read.advance(transport, () => clock);
    read.retry();
    await read.advance(transport, () => clock);
    clock += 1500;
    await read.advance(transport, () => clock);
    assert.equal(read.snapshot().result?.text, "verified read");
    assert.equal(read.snapshot().revoked, false);
    read.retry();
    await read.advance(transport, () => clock);
    assert.equal(submits, 2);
    read.revalidate();
    assert.equal(read.snapshot().result, undefined);
    await read.advance(transport, () => clock);
    assert.equal(submits, 3);
    clock += 1500;
    await read.advance(transport, () => clock);
    assert.equal(read.snapshot().result?.text, "verified read");
    clock += 120000;
    assert.equal(read.snapshot().result, undefined);
  });

  test("disposed session ignores a late private response", async () => {
    let resolve!: (value: Record<string, unknown>) => void;
    const read = new ReadSession("server_logs", { id: "server" });
    const pending = read.advance({
      submit: () =>
        new Promise((r) => {
          resolve = r;
        }),
      job: async () => ({}),
    });
    read.dispose();
    resolve({ text: "old-session secret" });
    await pending;
    assert.equal(read.snapshot().result, undefined);
  });

  test("identity epoch distinguishes same-account logins, aborts old work and bounds private caches", () => {
    resetIdentity();
    acceptIdentity("same-account", "session-a");
    const epoch = identityEpoch();
    const signal = identitySignal();
    const cache = new PrivateCache<string>(2);
    cache.set("a", "secret a");
    cache.set("b", "secret b");
    cache.set("c", "secret c");
    assert.deepEqual([...cache.keys()], ["b", "c"]);
    acceptIdentity("same-account", "session-a");
    assert.equal(identityEpoch(), epoch);
    assert.equal(cache.size, 2);
    acceptIdentity("same-account", "session-b");
    assert.equal(signal.aborted, true);
    assert.equal(cache.size, 0);
    assert.throws(() => assertIdentity(epoch), { name: "AbortError" });
    cache.set("new", "private");
    acceptIdentity("other-account", "session-b");
    assert.equal(cache.size, 0);
  });
  test("reopening a pending read ignores its old response and preserves uncertain admission idempotency", async () => {
    const releases: ((value: Record<string, unknown>) => void)[] = [];
    const keys: string[] = [];
    const read = new ReadSession("server_file_read", { id: "server" });
    const transport: ReadTransport = {
      submit: (_type, _values, key) => {
        keys.push(key);
        return new Promise((resolve) => releases.push(resolve));
      },
      job: async () => ({ state: "succeeded", result: { text: "current" } }),
    };
    const old = read.advance(transport);
    read.revalidate();
    const current = read.advance(transport);
    assert.equal(keys[0], keys[1]);
    releases[0]({ text: "before close" });
    await old;
    assert.equal(read.snapshot().result, undefined);
    releases[1]({ text: "after authorization" });
    await current;
    assert.equal(read.snapshot().result?.text, "after authorization");
  });
}
