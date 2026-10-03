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
  incoming: { items: TimelineItem[]; next_cursor: string | null },
  older = false,
): TimelineWindow {
  const merged = new Map(previous.items.map((item) => [item.id, item]));
  for (const item of incoming.items) merged.set(item.id, item);
  // Stable sort preserves the Core order at equal timestamps, including an
  // older page's order before the existing boundary. IDs are opaque.
  const order = older
    ? [
        ...incoming.items.map((item) => item.id),
        ...previous.items.map((item) => item.id),
      ]
    : [...merged.keys()];
  const items = [...new Set(order)]
    .map((id) => merged.get(id)!)
    .sort((a, b) => Date.parse(a.created_at) - Date.parse(b.created_at));
  return {
    items,
    cursor: older || !previous.loaded ? incoming.next_cursor : previous.cursor,
    loaded: true,
    scroll: previous.scroll,
  };
}
