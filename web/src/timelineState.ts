export type TimelineItem = {
  id: string;
  created_at: string;
  [key: string]: any;
};
export type TimelineWindow = {
  items: TimelineItem[];
  cursor: string | null;
  loaded: boolean;
  scroll: number;
  exhaustedAt?: string;
};
export function mergeWindow(
  previous: TimelineWindow,
  incoming: {
    items: TimelineItem[];
    next_cursor: string | null;
    updates?: TimelineItem[];
    removed_ids?: string[];
    removed_room_ids?: string[];
    preserveHistory?: boolean;
  },
  older = false,
): TimelineWindow {
  const merged = new Map(previous.items.map((item) => [item.id, item]));
  for (const item of [...incoming.items, ...(incoming.updates ?? [])])
    merged.set(item.id, item);
  for (const id of incoming.removed_ids ?? []) merged.delete(id);
  for (const [id, item] of merged)
    if (item.type === "message" && incoming.removed_room_ids?.includes(item.room_id)) merged.delete(id);
  // Core and the client share the same chronological tuple and ordinal tie-breaker.
  const fraction = (at: string) => (at.match(/\.(\d+)/)?.[1] ?? "").padEnd(9, "0");
  const items = [...merged.values()].sort((a, b) => {
    const ms = Date.parse(a.created_at) - Date.parse(b.created_at);
    if (ms) return ms;
    const af = fraction(a.created_at), bf = fraction(b.created_at);
    return af < bf ? -1 : af > bf ? 1 : a.id < b.id ? -1 : a.id > b.id ? 1 : 0;
  });
  const retained = older || incoming.preserveHistory ? items.slice(0, 200) : items.slice(-200);
  const exhaustedAt = older && !incoming.next_cursor ? retained[0]?.id : previous.exhaustedAt;
  const cursor = retained[0]?.before_cursor
    ? (retained[0].id === exhaustedAt ? null : retained[0].before_cursor)
    : older || !previous.loaded ? incoming.next_cursor : previous.cursor;
  return {
    items: retained,
    cursor,
    exhaustedAt,
    loaded: true,
    scroll: previous.scroll,
  };
}
