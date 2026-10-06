import { identityEpoch, PrivateCache, unreadable } from "./identity";
import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { api, command, date, jobTitle, type Data } from "./api";
import { useApp } from "./App";
import { t, renderSystemMessage, message, messageError } from "./i18n";
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
  dm: "text.private_chat",
  group: "text.group_chat",
  team: "text.team",
  party: "text.party",
  community: "text.community",
};
function closeContextMenu(button: HTMLButtonElement) {
  const menu = button.closest("details");
  menu?.removeAttribute("open");
  menu?.querySelector("summary")?.focus();
}
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
  // The selected room is the single source of truth for reading and sending.
  // Account-wide updates are read-only, including the optional Activity view.
  const kind = roomFilter
    ? "messages"
    : params.get("kind") === "events"
      ? "events"
      : "all";
  const key = `${identityEpoch()}/${roomFilter}/${kind}`;
  const [window, setWindow] = useState<Window>(() => ({
    items: [],
    cursor: null,
    loaded: false,
    scroll: 0,
  }));
  const [rooms, setRooms] = useState<Data[]>([]);
  const roomsRef = useRef<Data[]>([]);
  roomsRef.current = rooms;
  const [roomsCursor, setRoomsCursor] = useState<string | null>(null);
  const [roomsBusy, setRoomsBusy] = useState(false);
  const [readError, setReadError] = useState<Error | null>(null);
  const [actionError, setActionError] = useState<Error | null>(null);
  const [loading, setLoading] = useState(false);
  const [draft, setDraft] = useState(drafts.get(draftKey(roomFilter)) ?? "");
  const [sending, setSending] = useState(false);
  const currentTarget = useRef(roomFilter);
  currentTarget.current = roomFilter;
  const lifetime = useRef(true);
  useEffect(() => {
    lifetime.current = true;
    return () => {
      lifetime.current = false;
    };
  }, []);
  const [newUpdates, setNewUpdates] = useState(false);
  const feed = useRef<HTMLDivElement>(null);
  const load = useRef<(before?: string) => Promise<void>>(async () => {});
  const scrollChange = useRef<{
    top: number;
    height: number;
    bottom: boolean;
    older: boolean;
    initial: boolean;
    anchor?: { id: string; offset: number };
  } | null>(null);
  const state = useRef(window);
  state.current = window;
  useLayoutEffect(() => {
    const view = feed.current,
      pending = scrollChange.current;
    if (!view || !pending) return;
    const anchor =
      pending.anchor &&
      [...view.querySelectorAll<HTMLElement>("[data-item-id]")].find(
        (e) => e.dataset.itemId === pending.anchor!.id,
      );
    if (anchor)
      view.scrollTop +=
        anchor.getBoundingClientRect().top -
        view.getBoundingClientRect().top -
        pending.anchor!.offset;
    else if (pending.older)
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
    setReadError(null);
    let alive = true,
      running = false;
    let conversationsLoaded = false;
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
        if (known.length) query.set("known", known.join(","));
        const response = await api("/api/v1/timeline?" + query, {
          signal: controller.signal,
        });
        if (!alive) return;
        const knownRooms = [
          ...new Set(
            [
              currentTarget.current,
              ...[...drafts.keys()].map((k) => k.split("/")[1]).filter(Boolean),
              ...roomsRef.current.map((r) => r.id),
              ...[...windows.keys()]
                .map((k) => k.split("/")[1])
                .filter(Boolean),
            ].filter(Boolean),
          ),
        ].slice(0, 200);
        const roomQuery = new URLSearchParams();
        if (knownRooms.length) roomQuery.set("known", knownRooms.join(","));
        if (currentTarget.current)
          roomQuery.set("selected", currentTarget.current);
        const membership = await api("/api/v1/rooms?" + roomQuery, {
          signal: controller.signal,
        });
        if (!alive) return;
        const revoked = new Set<string>(membership.removed_room_ids ?? []);
        const conversationMap = new Map(
          [...roomsRef.current, ...(membership.rooms ?? [])].map((r: Data) => [
            r.id,
            r,
          ]),
        );
        for (const id of revoked) conversationMap.delete(id);
        const conversations = [...conversationMap.values()].slice(-128);
        if (!conversationsLoaded) {
          setRoomsCursor(membership.rooms_next_cursor ?? null);
          conversationsLoaded = true;
        }
        const previous = state.current;
        const view = feed.current;
        const atBottom =
          !view || view.scrollHeight - view.scrollTop - view.clientHeight < 48;
        const anchor =
          view &&
          [...view.querySelectorAll<HTMLElement>("[data-item-id]")].find(
            (e) =>
              e.getBoundingClientRect().bottom >=
              view.getBoundingClientRect().top,
          );
        scrollChange.current = {
          anchor:
            before && anchor
              ? {
                  id: anchor.dataset.itemId!,
                  offset:
                    anchor.getBoundingClientRect().top -
                    view!.getBoundingClientRect().top,
                }
              : undefined,
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
            removed_room_ids: [...revoked],
            preserveHistory: !before && !atBottom,
          },
          Boolean(before),
        );
        next.scroll = view?.scrollTop ?? 0;
        state.current = next;
        windows.set(key, next);
        setWindow(next);
        for (const draft of drafts.keys())
          if (revoked.has(draft.split("/")[1])) drafts.delete(draft);
        for (const [cacheKey, cached] of windows) {
          const cachedRoom = cacheKey.split("/")[1];
          if (cachedRoom && revoked.has(cachedRoom)) windows.delete(cacheKey);
          else
            cached.items = cached.items.filter(
              (item) => item.type !== "message" || !revoked.has(item.room_id),
            );
        }
        if (revoked.has(currentTarget.current)) {
          setDraft("");
        }
        setRooms(conversations);
        setReadError(null);
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
            setDraft("");
            setActionError(null);
          }
          setReadError(e as Error);
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
    // Polling pauses while an action dialog covers the Timeline. Resume as soon
    // as it closes so confirmed actions do not leave their old content visible.
    if (panelsVisible) void load.current();
  }, [panelsVisible]);
  useEffect(() => {
    if (!roomFilter) return;
    const room = rooms.find((r) => r.id === roomFilter);
    if (room) {
      setDraft(drafts.get(draftKey(room.id)) ?? "");
    }
  }, [roomFilter, rooms.find((r) => r.id === roomFilter)?.id]);
  const selected = rooms.find((r) => r.id === roomFilter);
  const selectedLabel = selected ? roomLabel(selected, me.account.id) : "";
  const paneTitle = roomFilter
    ? selectedLabel || t("text.selected_conversation")
    : kind === "events"
      ? t("text.activity")
      : t("text.all_updates");
  const selectConversation = (id: string) => {
    go("/timeline?room=" + encodeURIComponent(id));
  };
  return (
    <div className="timeline-layout">
      <nav className="conversation-list" aria-label={t("text.conversations")}>
        <h2>{t("text.conversations")}</h2>
        <a
          className={`conversation-link${!roomFilter && kind === "all" ? " conversation-active" : ""}`}
          href="#/timeline"
          aria-current={!roomFilter && kind === "all" ? "page" : undefined}
        >
          <strong>{t("text.all_updates")}</strong>
          <small>{t("text.messages_and_activity")}</small>
        </a>
        <a
          className={`conversation-link${!roomFilter && kind === "events" ? " conversation-active" : ""}`}
          href="#/timeline?kind=events"
          aria-current={!roomFilter && kind === "events" ? "page" : undefined}
        >
          <strong>{t("text.activity")}</strong>
          <small>{t("text.notifications_and_operations")}</small>
        </a>
        {rooms.map((room) => (
          <a
            key={room.id}
            className={`conversation-link${room.id === roomFilter ? " conversation-active" : ""}`}
            href={`#/timeline?room=${encodeURIComponent(room.id)}`}
            aria-current={room.id === roomFilter ? "page" : undefined}
          >
            <strong>{roomLabel(room, me.account.id)}</strong>
            <small>
              {t(kindNames[room.kind] ?? room.kind)}
              {room.unread > 0 ? " · " + t("text.0_unread", room.unread) : ""}
            </small>
          </a>
        ))}
        {roomsCursor && (
          <button
            disabled={roomsBusy}
            onClick={async () => {
              setRoomsBusy(true);
              try {
                const page = await api(
                  "/api/v1/rooms?before=" + encodeURIComponent(roomsCursor),
                );
                if (!lifetime.current) return;
                setRooms((previous) =>
                  [
                    ...new Map(
                      [...previous, ...(page.rooms ?? [])].map((r: Data) => [
                        r.id,
                        r,
                      ]),
                    ).values(),
                  ].slice(-128),
                );
                setRoomsCursor(page.rooms_next_cursor ?? null);
              } catch (e) {
                if (lifetime.current) setReadError(e as Error);
              } finally {
                if (lifetime.current) setRoomsBusy(false);
              }
            }}
          >
            {roomsBusy ? t("text.loading") : t("text.load_more_conversations")}
          </button>
        )}
        <div className="actions">
          <button
            onClick={() =>
              open({
                title: message("text.start_private_conversation"),
                fields: [
                  {
                    name: "target",
                    label: message("text.player"),
                    type: "player",
                  },
                ],
                submit: message("text.open_conversation"),
                action: async (values) => {
                  const r = await send("direct_room", values);
                  selectConversation(r.room_id);
                },
              })
            }
          >
            {t("text.new_private_conversation")}
          </button>
        </div>
      </nav>
      <section className="timeline-pane" aria-labelledby="timeline-pane-title">
        <header className="timeline-toolbar">
          <div className="grow">
            <h2 id="timeline-pane-title">{paneTitle}</h2>
            {!selected && (
              <p>
                {roomFilter
                  ? t(
                      "text.conversation_access_is_checked_before_messages_are_shown",
                    )
                  : t("text.select_a_conversation_to_write_a_message")}
              </p>
            )}
          </div>
        </header>
        {selected && <RoomTools key={selected.id} room={selected} />}
        {readError && (
          <p role="alert" className="error">
            {window.items.length
              ? t(
                  "text.timeline_could_not_update_previously_loaded_items_are_still_shown",
                )
              : ""}{" "}
            {messageError(readError)}{" "}
            <button onClick={() => void load.current()} disabled={loading}>
              {t("text.retry")}
            </button>
          </p>
        )}
        <div role="status" aria-live="polite" className="sr-only">
          {newUpdates ? t("text.new_updates_are_available") : ""}
        </div>
        {newUpdates && (
          <button
            className="new-updates"
            onClick={() => {
              feed.current?.scrollTo({ top: feed.current.scrollHeight });
              setNewUpdates(false);
            }}
          >
            {t("text.show_new_updates")}
          </button>
        )}
        <div
          className="timeline-feed"
          ref={feed}
          role="region"
          tabIndex={0}
          aria-label={
            selected
              ? t("text.messages_in_0", selectedLabel)
              : t("text.timeline_items")
          }
          aria-busy={!window.loaded && loading}
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
              {loading ? t("text.loading") : t("text.load_earlier_items")}
            </button>
          )}
          {!window.loaded && !readError && (
            <p role="status">{t("text.loading")}</p>
          )}
          {window.loaded && !window.items.length && (
            <Empty>
              {roomFilter
                ? selected
                  ? t("text.no_messages_yet_start_the_conversation")
                  : t("text.this_conversation_is_no_longer_available")
                : t("text.you_re_up_to_date")}
            </Empty>
          )}
          {window.items.map((item) =>
            item.type === "message" ? (
              <article className="message" key={item.id} data-item-id={item.id}>
                <div className="message-main">
                  <div className="message-author">
                    <strong>{item.author_name}</strong>
                    {!roomFilter && (
                      <a
                        href={`#/timeline?room=${encodeURIComponent(item.room_id)}`}
                      >
                        {item.room_name}
                      </a>
                    )}
                    <time dateTime={item.created_at}>
                      {date(item.created_at)}
                    </time>
                    {!item.deleted_at && item.author === me.account.id && (
                      <details className="context-menu">
                        <summary
                          aria-label={t(
                            "text.message_options_for_0",
                            item.author_name,
                          )}
                        >
                          <span aria-hidden="true">⋯</span>
                        </summary>
                        <div className="menu-content">
                          {item.author === me.account.id && (
                            <button
                              onClick={(e) => {
                                closeContextMenu(e.currentTarget);
                                open({
                                  title: message("text.delete_message"),
                                  action: async () => {
                                    await send("message_delete", {
                                      id: item.message_id,
                                    });
                                    await load.current();
                                  },
                                  note: () => (
                                    <p>
                                      {t(
                                        "text.remove_this_message_from_the_conversation_copies_alread_957e0fbcf3",
                                      )}
                                    </p>
                                  ),
                                  submit: message("text.confirm_deletion"),
                                });
                              }}
                            >
                              {t("text.delete")}
                            </button>
                          )}
                        </div>
                      </details>
                    )}
                  </div>
                  <p>
                    {item.deleted_at ? (
                      <em>{t("text.deleted_message")}</em>
                    ) : (
                      item.body
                    )}
                  </p>
                </div>
              </article>
            ) : item.type === "job" ? (
              <article
                className="list-row"
                key={item.id}
                data-item-id={item.id}
              >
                <div className="grow">
                  <strong>
                    {jobTitle(item)}
                    {item.server_name ? " · " + item.server_name : ""}
                  </strong>
                  <small>{date(item.created_at)}</small>
                  {item.progress?.message && (
                    <p>{renderSystemMessage(item.progress.message)}</p>
                  )}
                  {item.error && (
                    <p className="error">{renderSystemMessage(item.error)}</p>
                  )}
                </div>
                <Status value={item.state} />
                <div className="actions">
                  {item.server_id && (
                    <a href={`#/servers/${item.server_id}`}>
                      {t("text.open_server")}
                    </a>
                  )}
                  <button onClick={() => showJob(item.job_id, item)}>
                    {t("text.view_details")}
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
        {selected && (
          <form
            className="timeline-composer"
            aria-label={t("text.write_a_message_143ec689")}
            onSubmit={async (e) => {
              e.preventDefault();
              if (sending || !draft.trim()) return;
              setSending(true);
              setActionError(null);
              const roomId = roomFilter,
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
                    if (currentTarget.current === roomId) setDraft("");
                  }
                  setActionError(e as Error);
                }
              } finally {
                if (lifetime.current) setSending(false);
              }
            }}
          >
            <label className="field">
              {t("text.message")}
              <textarea
                aria-label={t("text.message")}
                placeholder={t("text.write_a_message")}
                value={draft}
                rows={3}
                maxLength={4000}
                onChange={(e) => {
                  setDraft(e.target.value);
                  drafts.set(draftKey(roomFilter), e.target.value);
                }}
              />
            </label>
            <button className="primary" disabled={sending || !draft.trim()}>
              {sending ? t("text.sending") : t("text.send")}
            </button>
          </form>
        )}
        {actionError && (
          <p role="alert" className="error">
            {messageError(actionError)}
          </p>
        )}
      </section>
    </div>
  );
}
