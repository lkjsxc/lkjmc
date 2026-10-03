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
};
export function mergeWindow(
  previous: TimelineWindow,
  incoming: {
    items: TimelineItem[];
    next_cursor: string | null;
    updates?: TimelineItem[];
    removed_ids?: string[];
    room_ids?: string[];
  },
  older = false,
): TimelineWindow {
  const merged = new Map(previous.items.map((item) => [item.id, item]));
  for (const item of [...incoming.items, ...(incoming.updates ?? [])])
    merged.set(item.id, item);
  for (const id of incoming.removed_ids ?? []) merged.delete(id);
  if (incoming.room_ids)
    for (const [id, item] of merged) {
      if (item.type === "message" && !incoming.room_ids.includes(item.room_id))
        merged.delete(id);
    }
  // Stable sort preserves the Core order at equal timestamps, including an
  // older page's order before the existing boundary. IDs are opaque.
  const order = older
    ? [
        ...incoming.items.map((item) => item.id),
        ...previous.items.map((item) => item.id),
      ]
    : [...merged.keys()];
  const items = [...new Set(order)]
    .filter((id) => merged.has(id))
    .map((id) => merged.get(id)!)
    .sort((a, b) => Date.parse(a.created_at) - Date.parse(b.created_at));
  return {
    items: older ? items.slice(0, 300) : items.slice(-300),
    cursor: older || !previous.loaded ? incoming.next_cursor : previous.cursor,
    loaded: true,
    scroll: previous.scroll,
  };
}
