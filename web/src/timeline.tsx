import { identityEpoch, PrivateCache, unreadable } from "./identity";
import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { api, command, date, jobTitle, type Data } from "./api";
import { useApp } from "./App";
import { t, translateError } from "./i18n";
import { Empty, Status } from "./ui";
import { NotificationItem } from "./jobs";
import { RoomTools } from "./roomTools";
import { mergeWindow, type TimelineWindow } from "./timelineState";

const drafts = new PrivateCache<string>(32);
type Window = TimelineWindow;
const windows = new PrivateCache<Window>(8);
const roomLabel = (room: Data, account: string) =>
  room.kind === "dm"
    ? room.members
        ?.filter((m: Data) => m.id !== account)
        .map((m: Data) => m.name)
        .join(", ") || room.name
    : room.name;
const kindNames: Record<string, string> = {
  dm: "Private chat",
  group: "Group chat",
  team: "Team",
  party: "Party",
  community: "Community",
};
export function Timeline() {
  const { route } = useApp();
  return <ScopedTimeline key={route.path} />;
}
function ScopedTimeline() {
  const { me, route, go, open, send, showJob, panelsVisible } = useApp();
  const visible = useRef(panelsVisible);
  visible.current = panelsVisible;
  const draftKey = (room: string) => `${identityEpoch()}/${room}`;
  const params = new URLSearchParams(route.path.split("?")[1]);
  const roomFilter = params.get("room") ?? "";
  const kind = ["all", "messages", "events"].includes(params.get("kind") ?? "")
    ? params.get("kind")!
    : "all";
  const key = `${identityEpoch()}/${roomFilter}/${kind}`;
  const [window, setWindow] = useState<Window>(() => ({
    items: [],
    cursor: null,
    loaded: false,
    scroll: 0,
  }));
  const [rooms, setRooms] = useState<Data[]>([]);
  const [readError, setReadError] = useState("");
  const [actionError, setActionError] = useState("");
  const [loading, setLoading] = useState(false);
  const [composerKind, setComposerKind] = useState("dm");
  const [target, setTarget] = useState(roomFilter);
  const [draft, setDraft] = useState(drafts.get(draftKey(roomFilter)) ?? "");
  const [sending, setSending] = useState(false);
  const currentTarget = useRef(target);
  currentTarget.current = target;
  const lifetime = useRef(true);
  useEffect(() => {
    lifetime.current = true;
    return () => {
      lifetime.current = false;
    };
  }, []);
  const [reportMode, setReportMode] = useState(false);
  const [chosen, setChosen] = useState<number[]>([]);
  const [reportBusy, setReportBusy] = useState(false);
  const [newUpdates, setNewUpdates] = useState(false);
  const feed = useRef<HTMLDivElement>(null);
  const load = useRef<(before?: string) => Promise<void>>(async () => {});
  const scrollChange = useRef<{
    top: number;
    height: number;
    bottom: boolean;
    older: boolean;
    initial: boolean;
  } | null>(null);
  const state = useRef(window);
  state.current = window;
  useLayoutEffect(() => {
    const view = feed.current,
      pending = scrollChange.current;
    if (!view || !pending) return;
    if (pending.older)
      view.scrollTop = pending.top + view.scrollHeight - pending.height;
    else if (pending.bottom) view.scrollTop = view.scrollHeight;
    else if (pending.initial)
      view.scrollTop = windows.get(key)?.scroll ?? view.scrollHeight;
    else view.scrollTop = pending.top;
    scrollChange.current = null;
  }, [window]);
  useEffect(() => {
    const cached = windows.get(key);
    const initial = cached ?? {
      items: [],
      cursor: null,
      loaded: false,
      scroll: 0,
    };
    state.current = initial;
    scrollChange.current = {
      top: cached?.scroll ?? 0,
      height: 0,
      bottom: !cached,
      older: false,
      initial: true,
    };
    setWindow({ items: [], cursor: null, loaded: false, scroll: 0 });
    setReadError("");
    let alive = true,
      running = false;
    const controller = new AbortController();
    let timer: ReturnType<typeof setTimeout>;
    async function fetchWindow(before?: string) {
      if (!alive || running) return;
      clearTimeout(timer);
      if (document.hidden || !visible.current) {
        timer = setTimeout(() => void fetchWindow(), 8000);
        return;
      }
      running = true;
      setLoading(true);
      try {
        const query = new URLSearchParams({ kind });
        if (roomFilter) query.set("room", roomFilter);
        if (before) query.set("before", before);
        const known = state.current.items.map((item) => item.id);
        if (known.length) query.set("known_ids", known.join(","));
        const response = await api("/api/v1/timeline?" + query, {
          signal: controller.signal,
        });
        if (!alive) return;
        const previous = state.current;
        const view = feed.current;
        const atBottom =
          !view || view.scrollHeight - view.scrollTop - view.clientHeight < 48;
        scrollChange.current = {
          top: view?.scrollTop ?? 0,
          height: view?.scrollHeight ?? 0,
          bottom:
            !before &&
            (!previous.loaded ||
              (atBottom && !view?.contains(document.activeElement))),
          older: Boolean(before),
          initial: false,
        };
        const added = (response.items ?? []).some(
          (item: Data) => !previous.items.some((old) => old.id === item.id),
        );
        const next = mergeWindow(
          previous,
          {
            items: response.items ?? [],
            next_cursor: response.next_cursor ?? null,
            updates: response.updates,
            removed_ids: response.removed_ids,
            room_ids: (response.rooms ?? []).map((room: Data) => room.id),
          },
          Boolean(before),
        );
        next.scroll = view?.scrollTop ?? 0;
        state.current = next;
        windows.set(key, next);
        setWindow(next);
        const permitted = new Set<string>(
          (response.rooms ?? []).map((room: Data) => room.id),
        );
        for (const draft of drafts.keys())
          if (!permitted.has(draft.split("/")[1])) drafts.delete(draft);
        for (const [cacheKey, cached] of windows) {
          const cachedRoom = cacheKey.split("/")[1];
          if (cachedRoom && !permitted.has(cachedRoom))
            windows.delete(cacheKey);
          else
            cached.items = cached.items.filter(
              (item) => item.type !== "message" || permitted.has(item.room_id),
            );
        }
        if (currentTarget.current && !permitted.has(currentTarget.current)) {
          setDraft("");
          setTarget("");
        }
        setChosen((ids) =>
          ids.filter((id) =>
            next.items.some(
              (item) => item.message_id === id && !item.deleted_at,
            ),
          ),
        );
        setRooms(response.rooms ?? []);
        setReadError("");
        if (previous.loaded && added && !before && !atBottom)
          setNewUpdates(true);
      } catch (e) {
        if (alive) {
          if (unreadable(e)) {
            windows.clear();
            drafts.clear();
            const empty = { items: [], cursor: null, loaded: false, scroll: 0 };
            state.current = empty;
            setWindow(empty);
            setRooms([]);
            setChosen([]);
            setDraft("");
            setTarget("");
            setActionError("");
          }
          setReadError((e as Error).message);
        }
      } finally {
        running = false;
        if (alive) {
          setLoading(false);
          timer = setTimeout(() => void fetchWindow(), 8000);
        }
      }
    }
    load.current = fetchWindow;
    void fetchWindow();
    return () => {
      alive = false;
      clearTimeout(timer);
      controller.abort();
      const cached = windows.get(key);
      if (cached && feed.current) cached.scroll = feed.current.scrollTop;
    };
  }, [key]);
  useEffect(() => {
    if (!roomFilter) return;
    const room = rooms.find((r) => r.id === roomFilter);
    if (room) {
      setTarget(room.id);
      setComposerKind(room.kind);
      setDraft(drafts.get(draftKey(room.id)) ?? "");
    }
  }, [roomFilter, rooms.find((r) => r.id === roomFilter)?.id]);
  const selected = rooms.find((r) => r.id === target);
  const kinds = [...new Set(rooms.map((room) => room.kind))];
  const changeTarget = (id: string) => {
    setTarget(id);
    setDraft(drafts.get(draftKey(id)) ?? "");
  };
  const filter = (nextRoom: string, nextKind: string) => {
    const q = new URLSearchParams();
    if (nextRoom) q.set("room", nextRoom);
    if (nextKind !== "all") q.set("kind", nextKind);
    setNewUpdates(false);
    go("/timeline" + (q.size ? "?" + q : ""));
  };
  async function report() {
    setReportBusy(true);
    setActionError("");
    try {
      const result = await api("/api/v1/reports/preview", {
        method: "POST",
        body: JSON.stringify({ message_ids: chosen }),
      });
      if (!lifetime.current) return;
      open({
        title: t("Review your submission"),
        type: "report",
        values: { target: null, message_ids: chosen },
        fields: [
          { name: "reason", label: t("Reason for report"), type: "textarea" },
        ],
        note: (
          <>
            <p>
              {t("Only the following ")}
              {result.evidence.length}
              {t(
                " messages will be submitted. Go back and select more if surrounding context is needed.",
              )}
            </p>
            {result.evidence.map((m: Data) => (
              <blockquote key={m.id}>
                <strong>{m.author_name}</strong>
                <p>{m.body}</p>
                <small>{date(m.created_at)}</small>
              </blockquote>
            ))}
          </>
        ),
        submit: t("Submit this report"),
      });
    } catch (e) {
      setActionError((e as Error).message);
    } finally {
      setReportBusy(false);
    }
  }
  return (
    <>
      <div className="section-toolbar">
        <div className="actions">
          <button
            onClick={() =>
              open({
                title: t("Start private conversation"),
                fields: [
                  { name: "target", label: t("Player"), type: "player" },
                ],
                submit: t("Open conversation"),
                action: async (values) => {
                  const r = await send("direct_room", values);
                  filter(r.room_id, "messages");
                },
              })
            }
          >
            {t("New private conversation")}
          </button>
          <button
            onClick={() =>
              open({
                title: t("Create group chat"),
                fields: [{ name: "name", label: t("Group name"), max: 80 }],
                submit: t("Create"),
                action: async (values) => {
                  const r = await send("room_create", values);
                  if (r.room_id) filter(r.room_id, "messages");
                  else await load.current();
                },
              })
            }
          >
            {t("Create group chat")}
          </button>
        </div>
      </div>
      <div className="timeline-filters">
        <label className="field">
          {t("Show")}
          <select
            aria-label={t("Show")}
            value={kind}
            onChange={(e) => filter(roomFilter, e.target.value)}
          >
            <option value="all">{t("Messages and events")}</option>
            <option value="messages">{t("Messages only")}</option>
            <option value="events">{t("Events only")}</option>
          </select>
        </label>
        <label className="field">
          {t("Conversation filter")}
          <select
            aria-label={t("Conversation filter")}
            value={roomFilter}
            onChange={(e) => filter(e.target.value, kind)}
          >
            <option value="">{t("All conversations")}</option>
            {roomFilter && !rooms.some((r) => r.id === roomFilter) && (
              <option value={roomFilter}>{t("Selected conversation")}</option>
            )}
            {rooms.map((room) => (
              <option key={room.id} value={room.id}>
                {roomLabel(room, me.account.id)}
              </option>
            ))}
          </select>
        </label>
        <button
          onClick={() => {
            setReportMode((v) => !v);
            setChosen([]);
          }}
        >
          {reportMode ? t("Finish selecting") : t("Report messages")}
        </button>
      </div>
      {readError && (
        <p role="alert" className="error">
          {window.items.length
            ? t(
                "Timeline could not update. Previously loaded items are still shown.",
              )
            : ""}{" "}
          {readError}{" "}
          <button onClick={() => void load.current()} disabled={loading}>
            {t("Retry")}
          </button>
        </p>
      )}
      {newUpdates && (
        <button
          className="new-updates"
          onClick={() => {
            feed.current?.scrollTo({ top: feed.current.scrollHeight });
            setNewUpdates(false);
          }}
        >
          {t("Show new updates")}
        </button>
      )}
      <div
        className="timeline-feed"
        ref={feed}
        aria-label={t("Timeline items")}
        onScroll={() => {
          const view = feed.current;
          if (
            view &&
            view.scrollHeight - view.scrollTop - view.clientHeight < 48
          )
            setNewUpdates(false);
        }}
      >
        {window.cursor && (
          <button
            disabled={loading}
            onClick={() => void load.current(window.cursor!)}
          >
            {loading ? t("Loading…") : t("Load earlier items")}
          </button>
        )}
        {!window.loaded && !readError && <p role="status">{t("Loading…")}</p>}
        {window.loaded && !window.items.length && (
          <Empty>{t("No items match this filter.")}</Empty>
        )}
        {window.items.map((item) =>
          item.type === "message" ? (
            <article className="message" key={item.id} data-item-id={item.id}>
              {reportMode && !item.deleted_at && (
                <input
                  type="checkbox"
                  checked={chosen.includes(item.message_id)}
                  aria-label={t(
                    "Include message by {0} in report",
                    item.author_name,
                  )}
                  onChange={(e) =>
                    setChosen((ids) =>
                      e.target.checked
                        ? [...ids, item.message_id]
                        : ids.filter((id) => id !== item.message_id),
                    )
                  }
                />
              )}
              <div className="message-main">
                <div className="message-author">
                  <strong>{item.author_name}</strong>
                  <a href={`#/timeline?room=${item.room_id}&kind=messages`}>
                    {item.room_name}
                  </a>
                  <time dateTime={item.created_at}>
                    {date(item.created_at)}
                  </time>
                  {item.author === me.account.id && !item.deleted_at && (
                    <button
                      onClick={() =>
                        open({
                          title: t("Delete message"),
                          action: async () => {
                            await send("message_delete", {
                              id: item.message_id,
                            });
                            await load.current();
                          },
                          note: (
                            <p>
                              {t(
                                "Remove this message from the conversation. Copies already submitted as report evidence may remain.",
                              )}
                            </p>
                          ),
                          submit: t("Confirm deletion"),
                        })
                      }
                    >
                      {t("Delete")}
                    </button>
                  )}
                </div>
                <p>
                  {item.deleted_at ? (
                    <em>{t("Deleted message")}</em>
                  ) : (
                    item.body
                  )}
                </p>
              </div>
            </article>
          ) : item.type === "job" ? (
            <article className="list-row" key={item.id} data-item-id={item.id}>
              <div className="grow">
                <strong>
                  {jobTitle(item)}
                  {item.server_name ? " · " + item.server_name : ""}
                </strong>
                <small>{date(item.created_at)}</small>
                {item.progress?.message && (
                  <p>{translateError(item.progress.message)}</p>
                )}
                {item.error && (
                  <p className="error">{translateError(item.error)}</p>
                )}
              </div>
              <Status value={item.state} />
              <div className="actions">
                {item.server_id && (
                  <a href={`#/servers/${item.server_id}`}>{t("Open server")}</a>
                )}
                <button onClick={() => showJob(item.job_id, item)}>
                  {t("View details")}
                </button>
              </div>
            </article>
          ) : (
            <article key={item.id} data-item-id={item.id}>
              <NotificationItem notice={item} />
            </article>
          ),
        )}
      </div>
      {reportMode && (
        <div className="section-toolbar">
          <span>
            {chosen.length}
            {t(" selected")}
          </span>
          <button
            disabled={!chosen.length || chosen.length > 30 || reportBusy}
            onClick={() => void report()}
          >
            {t("Review submission")}
          </button>
        </div>
      )}
      <section className="timeline-composer" aria-label={t("Write a message")}>
        <h2>{t("Write a message")}</h2>
        <div className="timeline-filters">
          <label className="field">
            {t("Conversation kind")}
            <select
              aria-label={t("Conversation kind")}
              disabled={sending}
              value={composerKind}
              onChange={(e) => {
                setComposerKind(e.target.value);
                changeTarget("");
              }}
            >
              {[...new Set([composerKind, ...kinds])].map((k) => (
                <option key={k} value={k}>
                  {t(kindNames[k] ?? k)}
                </option>
              ))}
            </select>
          </label>
          <label className="field">
            {t("Send to")}
            <select
              aria-label={t("Send to")}
              disabled={sending}
              value={target}
              onChange={(e) => changeTarget(e.target.value)}
            >
              <option value="">{t("Choose a conversation")}</option>
              {rooms
                .filter((r) => r.kind === composerKind)
                .map((r) => (
                  <option key={r.id} value={r.id}>
                    {roomLabel(r, me.account.id)}
                  </option>
                ))}
            </select>
          </label>
        </div>
        {selected ? (
          <>
            <RoomTools key={selected.id} room={selected} />
            <form
              onSubmit={async (e) => {
                e.preventDefault();
                if (sending || !draft.trim()) return;
                setSending(true);
                setActionError("");
                const roomId = target,
                  submitted = draft;
                try {
                  await command("message_send", {
                    room: roomId,
                    body: submitted,
                  });
                  if (!lifetime.current) return;
                  if (drafts.get(draftKey(roomId)) === submitted) {
                    drafts.set(draftKey(roomId), "");
                    if (currentTarget.current === roomId) setDraft("");
                  }
                  await load.current();
                } catch (e) {
                  if (lifetime.current) {
                    if (unreadable(e)) {
                      drafts.delete(draftKey(roomId));
                      for (const cached of windows.values())
                        cached.items = cached.items.filter(
                          (item) => item.room_id !== roomId,
                        );
                      const next = {
                        ...state.current,
                        items: state.current.items.filter(
                          (item) => item.room_id !== roomId,
                        ),
                      };
                      state.current = next;
                      setWindow(next);
                      setRooms((rooms) =>
                        rooms.filter((room) => room.id !== roomId),
                      );
                      if (currentTarget.current === roomId) {
                        setTarget("");
                        setDraft("");
                      }
                    }
                    setActionError((e as Error).message);
                  }
                } finally {
                  if (lifetime.current) setSending(false);
                }
              }}
            >
              <label className="field">
                {t("Message")}
                <textarea
                  aria-label={t("Message")}
                  value={draft}
                  rows={3}
                  maxLength={4000}
                  onChange={(e) => {
                    setDraft(e.target.value);
                    drafts.set(draftKey(target), e.target.value);
                  }}
                />
              </label>
              <button className="primary" disabled={sending || !draft.trim()}>
                {sending ? t("Sending…") : t("Send")}
              </button>
            </form>
          </>
        ) : (
          <p>
            {t(
              "Choose an authorized conversation, or create a private or group conversation above.",
            )}
          </p>
        )}
        {actionError && (
          <p role="alert" className="error">
            {actionError}
          </p>
        )}
      </section>
    </>
  );
}
