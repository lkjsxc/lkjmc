import { test } from "node:test";
import assert from "node:assert/strict";
import { mergeWindow } from "../src/timelineState.ts";

const empty = { items: [], cursor: null, loaded: false, scroll: 17 };
const item = (n) => ({
  id: `message:${n}`,
  type: "message",
  room_id: "room",
  created_at: new Date(n * 1000).toISOString(),
  before_cursor: `before:${n}`,
});

test("an exhausted first page never manufactures an older-page button from item cursors", () => {
  let state = mergeWindow(empty, {
    items: [item(2), item(1)],
    next_cursor: null,
  });
  assert.equal(state.cursor, null);
  assert.equal(state.exhaustedAt, "message:1");
  state = mergeWindow(state, {
    items: [item(3), item(2), item(1)],
    next_cursor: null,
  });
  assert.equal(state.cursor, null);
  assert.equal(state.scroll, 17);
  state = mergeWindow(state, {
    items: [item(4), item(3)],
    next_cursor: "tail:2",
  });
  assert.equal(
    state.cursor,
    null,
    "polling does not forget a retained beginning",
  );
});

test("an empty final older page remains exhausted during later polling", () => {
  let state = mergeWindow(empty, {
    items: [item(1), item(2)],
    next_cursor: "older",
  });
  assert.equal(state.cursor, "before:1");
  state = mergeWindow(state, { items: [], next_cursor: null }, true);
  assert.equal(state.cursor, null);
  state = mergeWindow(state, { items: [item(3)], next_cursor: "tail:2" });
  assert.equal(state.cursor, null);
});

test("evicting the beginning allows the bounded window to reload its real history", () => {
  let state = mergeWindow(empty, { items: [item(0)], next_cursor: null });
  state = mergeWindow(state, {
    items: Array.from({ length: 200 }, (_, n) => item(n + 1)),
    next_cursor: "tail:1",
  });
  assert.equal(state.items.length, 200);
  assert.equal(state.items[0].id, "message:1");
  assert.equal(state.cursor, "before:1");
  state = mergeWindow(state, { items: [item(0)], next_cursor: null }, true);
  assert.equal(state.items[0].id, "message:0");
  assert.equal(state.cursor, null);
});

test("trimming a complete response does not falsely mark the truncated history exhausted", () => {
  const state = mergeWindow(empty, {
    items: Array.from({ length: 201 }, (_, n) => item(n)),
    next_cursor: null,
  });
  assert.equal(state.exhaustedAt, "message:0");
  assert.equal(state.items[0].id, "message:1");
  assert.equal(state.cursor, "before:1");
});

test("an empty or fully revoked window cannot retain a stale older-page cursor", () => {
  assert.equal(
    mergeWindow(empty, { items: [], next_cursor: null }).cursor,
    null,
  );
  let state = mergeWindow(empty, { items: [item(1)], next_cursor: "older" });
  state = mergeWindow(state, {
    items: [],
    next_cursor: null,
    removed_room_ids: ["room"],
  });
  assert.equal(state.items.length, 0);
  assert.equal(state.cursor, null);
});
