import { t } from "./i18n";
import { useEffect, useRef, useState } from "react";
import type { Room } from "livekit-client";
import { api, date, type Data } from "./api";
import { useApp, PageBlock } from "./App";
import { ActionForm, Card, Empty, Icon, PlayerPicker } from "./ui";

export function Social({ data }: { data: Data }) {
  const { me, open, act, send, route, go } = useApp();
  const [room, setRoom] = useState<Data | null>(null);
  const [voice, setVoice] = useState<Room | null>(null);
  const [voiceName, setVoiceName] = useState("");
  const [muted, setMuted] = useState(false);
  const [error, setError] = useState("");
  const [connecting, setConnecting] = useState(false);
  const audio = useRef<HTMLDivElement>(null);
  const currentVoice = useRef<Room | null>(null);
  useEffect(
    () => () => {
      void currentVoice.current?.disconnect();
    },
    [],
  );
  useEffect(() => {
    setRoom((data.rooms ?? []).find((r: Data) => r.id === route.id) ?? null);
  }, [data.rooms, route.id]);
  async function joinVoice(room: Data) {
    setConnecting(true);
    setError("");
    try {
      const { Room, RoomEvent, Track } = await import("livekit-client");
      await currentVoice.current?.disconnect();
      const token = await api(`/api/v1/voice/${room.id}`, {
        method: "POST",
        body: "{}",
      });
      const next = new Room({ adaptiveStream: true, dynacast: true });
      currentVoice.current = next;
      next.on(RoomEvent.TrackSubscribed, (track) => {
        if (track.kind === Track.Kind.Audio) {
          const el = track.attach();
          audio.current?.appendChild(el);
        }
      });
      next.on(RoomEvent.TrackUnsubscribed, (track) =>
        track.detach().forEach((el) => el.remove()),
      );
      next.on(RoomEvent.Disconnected, () => {
        if (currentVoice.current === next) {
          setVoice(null);
          setVoiceName("");
        }
      });
      await next.connect(token.url, token.token);
      await next.localParticipant.setMicrophoneEnabled(true);
      setVoice(next);
      setVoiceName(room.name);
      setMuted(false);
    } catch (e) {
      await currentVoice.current?.disconnect();
      setError((e as Error).message);
    } finally {
      setConnecting(false);
    }
  }
  const friends = (data.friends ?? []).filter((f: Data) =>
    route.section === "incoming"
      ? f.state === "pending" && f.requester !== me.account.id
      : route.section === "outgoing"
        ? f.state === "pending" && f.requester === me.account.id
        : f.state === "accepted",
  );
  const invite = (kind: string, resource: string) =>
    open({
      title: t("Send an invitation"),
      type: "invite",
      values: { kind, resource },
      fields: [{ name: "target", label: t("Invite a player"), type: "player" }],
      submit: t("Invite"),
    });
  return (
    <>
      <div ref={audio} className="remote-audio" />
      {error && (
        <p role="alert" className="error">
          {error}
        </p>
      )}
      {voice && (
        <div className="voice-bar">
          <Icon name="voice" />
          <strong>{voiceName}</strong>
          <span>{t("In voice chat · Not recorded")}</span>
          <button
            onClick={async () => {
              try {
                await voice.localParticipant.setMicrophoneEnabled(muted);
                setMuted(!muted);
              } catch (e) {
                setError((e as Error).message);
              }
            }}
          >
            {muted ? t("Unmute microphone") : t("Mute microphone")}
          </button>
          <button onClick={() => void voice.disconnect()}>
            {t("Leave voice chat")}
          </button>
        </div>
      )}
      <div className="grid two">
        <PageBlock id={["friends", "incoming", "outgoing"]}>
          <Card
            title={t("Friends")}
            action={
              <button
                onClick={() =>
                  open({
                    title: t("Friend request"),
                    type: "friend_request",
                    fields: [
                      { name: "target", label: t("Player"), type: "player" },
                    ],
                    submit: t("Send friend request"),
                  })
                }
              >
                <Icon name="plus" />
                {t("Add")}
              </button>
            }
          >
            {!friends.length ? (
              <Empty>
                {t("No friends yet. Choose “Add” to find a player.")}
              </Empty>
            ) : (
              friends.map((f: Data) => (
                <div className="friend-row" key={f.id}>
                  <span className="avatar">{f.name.slice(0, 1)}</span>
                  <div className="grow">
                    <strong>{f.name}</strong>
                    <small>
                      {f.state === "accepted"
                        ? f.server_id
                          ? t("Online in-game")
                          : t("Friends")
                        : f.requester === me.account.id
                          ? t("Awaiting response")
                          : t("Incoming friend request")}
                    </small>
                  </div>
                  <div className="actions">
                    {f.state === "pending" && f.requester !== me.account.id ? (
                      <>
                        <button
                          onClick={() =>
                            act("friend_respond", {
                              target: f.id,
                              accept: true,
                            })
                          }
                        >
                          {t("Accept")}
                        </button>
                        <button
                          className="quiet"
                          onClick={() =>
                            act("friend_respond", {
                              target: f.id,
                              accept: false,
                            })
                          }
                        >
                          {t("Decline")}
                        </button>
                      </>
                    ) : f.state === "accepted" ? (
                      <button
                        onClick={() =>
                          void send("direct_room", { target: f.id })
                            .then((r) => {
                              go("/chat/" + r.room_id);
                            })
                            .catch((e) => setError(e.message))
                        }
                      >
                        {t("Chat")}
                      </button>
                    ) : null}
                    <button
                      className="quiet"
                      aria-label={t("Manage friendship with {0}", f.name)}
                      onClick={() =>
                        open({
                          title: f.name,
                          fields: [
                            {
                              name: "action",
                              label: t("Actions"),
                              type: "select",
                              options: [
                                {
                                  value: "remove",
                                  label: t("Remove friend or request"),
                                },
                                { value: "block", label: t("Block player") },
                              ],
                            },
                          ],
                          action: async (v) =>
                            send(
                              v.action === "block" ? "block" : "friend_remove",
                              {
                                target: f.id,
                                ...(v.action === "block"
                                  ? { blocked: true }
                                  : {}),
                              },
                            ),
                        })
                      }
                    >
                      {t("Settings")}
                    </button>
                  </div>
                </div>
              ))
            )}
          </Card>
        </PageBlock>
        <PageBlock id={["team", "team-members", "team-settings"]}>
          <Card title={t("Teams")}>
            <div className="group-section">
              <span className="eyebrow">{t("Team")}</span>
              {data.team ? (
                <>
                  <h3>{data.team.name}</h3>
                  <p>{t("Share land, coins, and buildings with your team.")}</p>
                  <div className="actions">
                    <button onClick={() => invite("team", data.team.id)}>
                      {t("Invite member")}
                    </button>
                    <button onClick={() => go("/chat/" + data.team.room_id)}>
                      {t("Team chat")}
                    </button>
                  </div>
                  <PageBlock id="team-members">
                    {" "}
                    {data.team.members?.map((m: Data) => (
                      <div className="list-row" key={m.account_id}>
                        <div>
                          <strong>{m.name}</strong>
                          {m.account_id === data.team.leader && (
                            <small>{t("Leader")}</small>
                          )}
                        </div>
                        {me.account.id === data.team.leader &&
                          m.account_id !== data.team.leader && (
                            <div className="actions">
                              <button
                                className="quiet"
                                onClick={() =>
                                  open({
                                    title: t("Permissions for {0}", m.name),
                                    type: "team_permissions",
                                    values: {
                                      team: data.team.id,
                                      member: m.account_id,
                                    },
                                    fields: [
                                      ["build", t("Build"), m.can_build],
                                      ["sell", t("Sell"), m.can_sell],
                                      [
                                        "spend",
                                        t("Spend shared coins"),
                                        m.can_spend,
                                      ],
                                      [
                                        "members",
                                        t("Manage members"),
                                        m.can_manage_members,
                                      ],
                                      [
                                        "administer",
                                        t("Manage team"),
                                        m.can_administer,
                                      ],
                                    ].map(([name, label, value]) => ({
                                      name: String(name),
                                      label: String(label),
                                      type: "checkbox",
                                      value: Boolean(value),
                                    })),
                                  })
                                }
                              >
                                {t("Role")}
                              </button>
                              <button
                                className="quiet"
                                onClick={() =>
                                  open({
                                    title: t("Transfer leadership"),
                                    type: "team_transfer",
                                    values: {
                                      team: data.team.id,
                                      target: m.account_id,
                                    },
                                    note: (
                                      <p>
                                        {m.name}
                                        {t(" will become your team’s leader.")}
                                      </p>
                                    ),
                                    submit: t("Transfer leadership now"),
                                  })
                                }
                              >
                                {t("Transfer")}
                              </button>
                            </div>
                          )}
                      </div>
                    ))}
                  </PageBlock>
                  <PageBlock id="team-settings">
                    {" "}
                    <div className="actions">
                      <button
                        className="quiet"
                        onClick={() =>
                          open({
                            title: t("Leave team"),
                            type: "team_leave",
                            note: (
                              <p>
                                {t(
                                  "Land and shared assets stay with the team. Transfer leadership first if you are the leader.",
                                )}
                              </p>
                            ),
                            submit: t("Leave team"),
                          })
                        }
                      >
                        {t("Leave team")}
                      </button>
                      {me.account.id === data.team.leader && (
                        <button
                          className="quiet danger"
                          onClick={() =>
                            open({
                              title: t("Disband team"),
                              type: "team_disband",
                              values: { team: data.team.id },
                              note: (
                                <p>
                                  {t(
                                    "Disband a team after disposing of its land, stored assets, and shared balance.",
                                  )}
                                </p>
                              ),
                              submit: t("Confirm disbanding"),
                            })
                          }
                        >
                          {t("Disband")}
                        </button>
                      )}
                    </div>
                  </PageBlock>
                </>
              ) : (
                <>
                  <p>
                    {t(
                      "You can belong to one team. The team’s own achievements increase its land allowance.",
                    )}
                  </p>
                  <button
                    onClick={() =>
                      open({
                        title: t("Create team"),
                        type: "team_create",
                        fields: [
                          { name: "name", label: t("Team name"), max: 64 },
                        ],
                        submit: t("Create team"),
                      })
                    }
                  >
                    {t("Create team")}
                  </button>
                </>
              )}
            </div>
          </Card>
        </PageBlock>
        <PageBlock id={["party", "party-members", "party-ready"]}>
          <Card title={t("Parties")}>
            <div className="group-section">
              <span className="eyebrow">{t("Party")}</span>
              {data.party ? (
                <>
                  <h3>{data.party.name}</h3>
                  <div className="actions">
                    <button onClick={() => invite("party", data.party.id)}>
                      {t("Invite member")}
                    </button>
                    <button onClick={() => go("/chat/" + data.party.room_id)}>
                      {t("Chat room")}
                    </button>
                    <button
                      onClick={() =>
                        act("party_ready", {
                          ready: !data.party.members.find(
                            (m: Data) => m.account_id === me.account.id,
                          )?.ready,
                        })
                      }
                    >
                      {data.party.members.find(
                        (m: Data) => m.account_id === me.account.id,
                      )?.ready
                        ? t("Cancel ready status")
                        : t("Ready")}
                    </button>
                    <button
                      className="quiet"
                      onClick={() => act("party_leave")}
                    >
                      {t("Leave")}
                    </button>
                  </div>
                  <PageBlock id={["party-members", "party-ready"]}>
                    {" "}
                    {data.party.members?.map((m: Data) => (
                      <p key={m.account_id}>
                        {m.ready ? "✓" : "○"} {m.name}
                        {m.account_id === data.party.leader
                          ? t(" · Leader")
                          : ""}
                        {me.account.id === data.party.leader &&
                          m.account_id !== me.account.id && (
                            <button
                              className="quiet"
                              onClick={() =>
                                act("party_transfer", { target: m.account_id })
                              }
                            >
                              {t("Make leader")}
                            </button>
                          )}
                      </p>
                    ))}
                  </PageBlock>
                </>
              ) : (
                <>
                  <p>{t("A temporary group for adventures and meeting up.")}</p>
                  <button
                    onClick={() =>
                      open({
                        title: t("Create party"),
                        type: "party_create",
                        fields: [
                          { name: "name", label: t("Party name"), max: 80 },
                        ],
                        submit: t("Create party"),
                      })
                    }
                  >
                    {t("Create party")}
                  </button>
                </>
              )}
            </div>
          </Card>
        </PageBlock>
      </div>
      <PageBlock id="chat">
        <div className="chat-layout">
          <Card
            title={t("Conversations")}
            action={
              <button
                className="quiet"
                onClick={() =>
                  open({
                    title: t("Create group chat"),
                    type: "room_create",
                    fields: [{ name: "name", label: t("Group name"), max: 80 }],
                    submit: t("Create"),
                  })
                }
              >
                <Icon name="plus" />
              </button>
            }
          >
            {!data.rooms?.length ? (
              <Empty>
                {t(
                  "No conversations yet. Start a private chat with a friend or create a group chat.",
                )}
              </Empty>
            ) : (
              data.rooms.map((r: Data) => (
                <button
                  className={`room-option ${room?.id === r.id ? "selected" : ""}`}
                  key={r.id}
                  onClick={() => {
                    go("/chat/" + r.id);
                    act("room_read", { room: r.id });
                  }}
                >
                  <div>
                    <strong>
                      {r.kind === "dm"
                        ? r.members
                            ?.filter((m: Data) => m.id !== me.account.id)
                            .map((m: Data) => m.name)
                            .join(", ") || r.name
                        : r.name}
                    </strong>
                    <small>
                      {r.members?.length ?? 0} {t(" people")}
                    </small>
                  </div>
                  {r.unread > 0 && <b>{r.unread}</b>}
                </button>
              ))
            )}
          </Card>
          {room ? (
            <Chat
              key={room.id}
              room={room}
              onVoice={() => void joinVoice(room)}
              voiceAvailable={me.voice_available && !connecting}
              onInvite={() => invite("room", room.id)}
            />
          ) : (
            <div className="card">
              <Empty>{t("Select a conversation to see messages.")}</Empty>
            </div>
          )}
        </div>
      </PageBlock>
      <PageBlock id="communities">
        <Card
          title={t("Server communities")}
          action={
            <button
              onClick={() =>
                open({
                  title: t("Create community"),
                  type: "community_create",
                  fields: [
                    { name: "name", label: t("Community name"), max: 64 },
                  ],
                  submit: t("Create"),
                })
              }
            >
              {t("Create")}
            </button>
          }
        >
          <p>
            {t(
              "Manage servers together. Each server owner’s tier determines their allowance.",
            )}
          </p>
          {data.communities?.map((c: Data) => (
            <div className="list-row" key={c.id}>
              <strong>{c.name}</strong>
              <button onClick={() => invite("community", c.id)}>
                {t("Invite members")}
              </button>
            </div>
          ))}
        </Card>
      </PageBlock>
    </>
  );
}

function Chat({
  room,
  onVoice,
  voiceAvailable,
  onInvite,
}: {
  room: Data;
  onVoice: () => void;
  voiceAvailable: boolean;
  onInvite: () => void;
}) {
  const { me, send, act, open } = useApp();
  const [messages, setMessages] = useState<Data[]>([]);
  const [query, setQuery] = useState("");
  const [draft, setDraft] = useState("");
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const [chosen, setChosen] = useState<number[]>([]);
  const [reportMode, setReportMode] = useState(false);
  const [more, setMore] = useState(true);
  async function load(before?: number) {
    try {
      const result = await api(
        `/api/v1/rooms/${room.id}/messages?q=${encodeURIComponent(query)}${before ? `&before=${before}` : ""}`,
      );
      setMessages((previous) =>
        before ? [...result.messages, ...previous] : result.messages,
      );
      setMore(result.messages.length === 100);
      setError("");
    } catch (e) {
      setError((e as Error).message);
    }
  }
  useEffect(() => {
    let alive = true;
    const fetchMessages = () =>
      api(`/api/v1/rooms/${room.id}/messages?q=${encodeURIComponent(query)}`)
        .then((v) => {
          if (alive) {
            setMessages(v.messages);
            setMore(v.messages.length === 100);
            setError("");
          }
        })
        .catch((e) => {
          if (alive) setError(e.message);
        });
    void fetchMessages();
    const timer = setInterval(() => {
      if (document.hasFocus() && !reportMode) void fetchMessages();
    }, 8000);
    return () => {
      alive = false;
      clearInterval(timer);
    };
  }, [room.id, query, reportMode]);
  async function report() {
    try {
      const result = await api("/api/v1/reports/preview", {
        method: "POST",
        body: JSON.stringify({ message_ids: chosen }),
      });
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
      setError((e as Error).message);
    }
  }
  return (
    <section className="card chat">
      <div className="card-head">
        <div>
          <h2>{room.name}</h2>
          <small>
            {room.kind === "dm"
              ? t("Private chat")
              : t("Visible to group members")}
          </small>
        </div>
        <div className="actions">
          {room.kind === "group" && (
            <button className="quiet" onClick={onInvite}>
              {t("Invite member")}
            </button>
          )}
          <button
            onClick={onVoice}
            disabled={!voiceAvailable}
            title={
              voiceAvailable
                ? t("Join voice room")
                : t("Voice service is being set up")
            }
          >
            <Icon name="voice" />
            {t("Voice chat")}
          </button>
          <button
            className="quiet"
            onClick={() => {
              setReportMode(!reportMode);
              setChosen([]);
            }}
          >
            {reportMode ? t("Finish selecting") : t("Report")}
          </button>
        </div>
      </div>
      <label className="chat-search">
        <span className="sr-only">{t("Search this conversation")}</span>
        <input
          type="search"
          placeholder={t("Search this conversation")}
          value={query}
          onChange={(e) => setQuery(e.target.value)}
        />
      </label>
      {error && (
        <p role="alert" className="error">
          {error}
        </p>
      )}
      <div className="messages" aria-live="polite">
        {more && messages.length > 0 && (
          <button className="quiet" onClick={() => void load(messages[0].id)}>
            {t("Earlier messages")}
          </button>
        )}
        {!messages.length ? (
          <Empty>
            {query ? t("No matching messages.") : t("No messages yet.")}
          </Empty>
        ) : (
          messages.map((m) => (
            <article
              className={`message ${m.author === me.account.id ? "own" : ""}`}
              key={m.id}
            >
              {reportMode && !m.deleted_at && (
                <input
                  type="checkbox"
                  aria-label={t(
                    "Include message by {0} in report",
                    m.author_name,
                  )}
                  checked={chosen.includes(m.id)}
                  onChange={(e) =>
                    setChosen((ids) =>
                      e.target.checked
                        ? [...ids, m.id]
                        : ids.filter((id) => id !== m.id),
                    )
                  }
                />
              )}
              <div className="message-main">
                <div className="message-author">
                  <strong>{m.author_name}</strong>
                  <time>{date(m.created_at)}</time>
                  {m.author === me.account.id && !m.deleted_at && (
                    <button
                      className="quiet"
                      onClick={() =>
                        open({
                          title: t("Delete message"),
                          type: "message_delete",
                          values: { id: m.id },
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
                <p>{m.deleted_at ? <em>{t("Deleted message")}</em> : m.body}</p>
              </div>
            </article>
          ))
        )}
      </div>
      {reportMode ? (
        <div className="chat-compose">
          <span>
            {chosen.length}
            {t(" selected")}
          </span>
          <button
            disabled={!chosen.length || chosen.length > 30}
            onClick={() => void report()}
          >
            {t("Review submission")}
          </button>
        </div>
      ) : (
        <form
          className="chat-compose"
          onSubmit={async (e) => {
            e.preventDefault();
            if (!draft.trim()) return;
            setBusy(true);
            try {
              await send("message_send", { room: room.id, body: draft });
              setDraft("");
              await load();
            } catch (e) {
              setError((e as Error).message);
            } finally {
              setBusy(false);
            }
          }}
        >
          <label className="sr-only" htmlFor="message-draft">
            {t("Message")}
          </label>
          <textarea
            id="message-draft"
            placeholder={t("Write a message…")}
            value={draft}
            onChange={(e) => setDraft(e.target.value)}
            rows={2}
            maxLength={4000}
            disabled={busy}
          />
          <button
            className="primary"
            type="submit"
            disabled={busy || !draft.trim()}
          >
            {busy ? t("Sending…") : t("Send")}
          </button>
        </form>
      )}
      {room.kind === "group" && (
        <button
          className="quiet"
          onClick={() =>
            open({
              title: t("Leave group"),
              type: "room_leave",
              values: { room: room.id },
              note: (
                <p>
                  {t(
                    "You will lose access to this conversation and its voice room.",
                  )}
                </p>
              ),
              submit: t("Leave now"),
            })
          }
        >
          {t("Leave this group")}
        </button>
      )}
    </section>
  );
}
