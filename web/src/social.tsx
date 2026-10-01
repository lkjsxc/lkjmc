import { useEffect, useRef, useState } from "react";
import type { Room } from "livekit-client";
import { api, date, type Data } from "./api";
import { useApp } from "./App";
import { ActionForm, Card, Empty, Icon, PlayerPicker } from "./ui";

export function Social({ data }: { data: Data }) {
  const { me, open, act, send } = useApp();
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
    if (room) {
      const next = (data.rooms ?? []).find((r: Data) => r.id === room.id);
      if (!next) setRoom(null);
      else setRoom(next);
    }
  }, [data.rooms]);
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
  const invite = (kind: string, resource: string) =>
    open({
      title: "招待を送る",
      type: "invite",
      values: { kind, resource },
      fields: [{ name: "target", label: "招待するプレイヤー", type: "player" }],
      submit: "招待する",
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
          <span>通話中 · 録音なし</span>
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
            {muted ? "マイクをオン" : "マイクをミュート"}
          </button>
          <button onClick={() => void voice.disconnect()}>通話を終了</button>
        </div>
      )}
      <div className="grid two">
        <Card
          title="フレンド"
          action={
            <button
              onClick={() =>
                open({
                  title: "フレンド申請",
                  type: "friend_request",
                  fields: [
                    { name: "target", label: "プレイヤー", type: "player" },
                  ],
                  submit: "申請する",
                })
              }
            >
              <Icon name="plus" />
              追加
            </button>
          }
        >
          {!data.friends?.length ? (
            <Empty>名前で検索して、フレンドを追加しましょう。</Empty>
          ) : (
            data.friends.map((f: Data) => (
              <div className="friend-row" key={f.id}>
                <span className="avatar">{f.name.slice(0, 1)}</span>
                <div className="grow">
                  <strong>{f.name}</strong>
                  <small>
                    {f.state === "accepted"
                      ? f.server_id
                        ? "ゲームでプレイ中"
                        : "フレンド"
                      : f.requester === me.account.id
                        ? "申請への返事を待っています"
                        : "フレンド申請が届いています"}
                  </small>
                </div>
                <div className="actions">
                  {f.state === "pending" && f.requester !== me.account.id ? (
                    <>
                      <button
                        onClick={() =>
                          act("friend_respond", { target: f.id, accept: true })
                        }
                      >
                        承諾
                      </button>
                      <button
                        className="quiet"
                        onClick={() =>
                          act("friend_respond", { target: f.id, accept: false })
                        }
                      >
                        断る
                      </button>
                    </>
                  ) : f.state === "accepted" ? (
                    <button
                      onClick={() =>
                        void send("direct_room", { target: f.id })
                          .then((r) => {
                            setRoom({
                              id: r.room_id,
                              name: f.name,
                              kind: "dm",
                            });
                          })
                          .catch((e) => setError(e.message))
                      }
                    >
                      話す
                    </button>
                  ) : null}
                  <button
                    className="quiet"
                    aria-label={`${f.name}との関係を設定`}
                    onClick={() =>
                      open({
                        title: f.name,
                        fields: [
                          {
                            name: "action",
                            label: "操作",
                            type: "select",
                            options: [
                              {
                                value: "remove",
                                label: "フレンド・申請を解除",
                              },
                              { value: "block", label: "ブロックする" },
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
                    設定
                  </button>
                </div>
              </div>
            ))
          )}
        </Card>
        <Card title="一緒に遊ぶ仲間">
          <div className="group-section">
            <span className="eyebrow">TEAM · ずっと続く仲間</span>
            {data.team ? (
              <>
                <h3>{data.team.name}</h3>
                <p>共有の土地・財布・建物を持つチームです。</p>
                <div className="actions">
                  <button onClick={() => invite("team", data.team.id)}>
                    招待
                  </button>
                  <button
                    onClick={() =>
                      setRoom(
                        data.rooms.find(
                          (r: Data) => r.id === data.team.room_id,
                        ),
                      )
                    }
                  >
                    チームチャット
                  </button>
                </div>
                {data.team.members?.map((m: Data) => (
                  <div className="list-row" key={m.account_id}>
                    <div>
                      <strong>{m.name}</strong>
                      {m.account_id === data.team.leader && (
                        <small>リーダー</small>
                      )}
                    </div>
                    {me.account.id === data.team.leader &&
                      m.account_id !== data.team.leader && (
                        <div className="actions">
                          <button
                            className="quiet"
                            onClick={() =>
                              open({
                                title: `${m.name}の権限`,
                                type: "team_permissions",
                                values: {
                                  team: data.team.id,
                                  member: m.account_id,
                                },
                                fields: [
                                  ["build", "建築", m.can_build],
                                  ["sell", "売却", m.can_sell],
                                  ["spend", "共有残高の利用", m.can_spend],
                                  [
                                    "members",
                                    "メンバー管理",
                                    m.can_manage_members,
                                  ],
                                  [
                                    "administer",
                                    "チーム管理",
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
                            権限
                          </button>
                          <button
                            className="quiet"
                            onClick={() =>
                              open({
                                title: "リーダーを委譲する",
                                type: "team_transfer",
                                values: {
                                  team: data.team.id,
                                  target: m.account_id,
                                },
                                note: (
                                  <p>{m.name}さんにチームの管理を委ねます。</p>
                                ),
                                submit: "委譲する",
                              })
                            }
                          >
                            委譲
                          </button>
                        </div>
                      )}
                  </div>
                ))}
                <div className="actions">
                  <button
                    className="quiet"
                    onClick={() =>
                      open({
                        title: "チームを抜ける",
                        type: "team_leave",
                        note: (
                          <p>
                            チームの土地・共有資産はチームに残ります。リーダーの場合は先に委譲してください。
                          </p>
                        ),
                        submit: "チームを抜ける",
                      })
                    }
                  >
                    チームを抜ける
                  </button>
                  {me.account.id === data.team.leader && (
                    <button
                      className="quiet danger"
                      onClick={() =>
                        open({
                          title: "チームを解散する",
                          type: "team_disband",
                          values: { team: data.team.id },
                          note: (
                            <p>
                              土地・保管資産・共有残高を処分したチームを解散します。
                            </p>
                          ),
                          submit: "解散する",
                        })
                      }
                    >
                      解散
                    </button>
                  )}
                </div>
              </>
            ) : (
              <>
                <p>
                  チームは1つまで所属できます。チーム自身の実績で、土地の枠を増やせます。
                </p>
                <button
                  onClick={() =>
                    open({
                      title: "チームを作る",
                      type: "team_create",
                      fields: [{ name: "name", label: "チーム名", max: 64 }],
                      submit: "チームを作る",
                    })
                  }
                >
                  チームを作る
                </button>
              </>
            )}
          </div>
          <div className="group-section">
            <span className="eyebrow">PARTY · 今日だけの仲間</span>
            {data.party ? (
              <>
                <h3>{data.party.name}</h3>
                <div className="actions">
                  <button onClick={() => invite("party", data.party.id)}>
                    招待
                  </button>
                  <button
                    onClick={() =>
                      setRoom(
                        data.rooms.find(
                          (r: Data) => r.id === data.party.room_id,
                        ),
                      )
                    }
                  >
                    チャット
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
                      ? "準備完了を取り消す"
                      : "準備完了"}
                  </button>
                  <button className="quiet" onClick={() => act("party_leave")}>
                    抜ける
                  </button>
                </div>
                {data.party.members?.map((m: Data) => (
                  <p key={m.account_id}>
                    {m.ready ? "✓" : "○"} {m.name}
                    {m.account_id === data.party.leader ? " · リーダー" : ""}
                    {me.account.id === data.party.leader &&
                      m.account_id !== me.account.id && (
                        <button
                          className="quiet"
                          onClick={() =>
                            act("party_transfer", { target: m.account_id })
                          }
                        >
                          リーダーにする
                        </button>
                      )}
                  </p>
                ))}
              </>
            ) : (
              <>
                <p>冒険や待ち合わせのために、一時的に集まるグループです。</p>
                <button
                  onClick={() =>
                    open({
                      title: "パーティーを作る",
                      type: "party_create",
                      fields: [
                        { name: "name", label: "パーティー名", max: 80 },
                      ],
                      submit: "パーティーを作る",
                    })
                  }
                >
                  パーティーを作る
                </button>
              </>
            )}
          </div>
        </Card>
      </div>
      <div className="chat-layout">
        <Card
          title="会話"
          action={
            <button
              className="quiet"
              onClick={() =>
                open({
                  title: "グループチャットを作る",
                  type: "room_create",
                  fields: [{ name: "name", label: "グループ名", max: 80 }],
                  submit: "作成する",
                })
              }
            >
              <Icon name="plus" />
            </button>
          }
        >
          {!data.rooms?.length ? (
            <Empty>フレンドと話すか、グループを作ってみましょう。</Empty>
          ) : (
            data.rooms.map((r: Data) => (
              <button
                className={`room-option ${room?.id === r.id ? "selected" : ""}`}
                key={r.id}
                onClick={() => {
                  setRoom(r);
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
                  <small>{r.members?.length ?? 0} 人</small>
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
            <Empty>会話を選ぶと、ここにメッセージが表示されます。</Empty>
          </div>
        )}
      </div>
      <Card
        title="サーバーコミュニティ"
        action={
          <button
            onClick={() =>
              open({
                title: "コミュニティを作る",
                type: "community_create",
                fields: [{ name: "name", label: "コミュニティ名", max: 64 }],
                submit: "作成する",
              })
            }
          >
            作成する
          </button>
        }
      >
        <p>
          サーバーを共同で管理するグループです。作成枠はそれぞれのサーバー所有者のランクに従います。
        </p>
        {data.communities?.map((c: Data) => (
          <div className="list-row" key={c.id}>
            <strong>{c.name}</strong>
            <button onClick={() => invite("community", c.id)}>
              メンバーを招待
            </button>
          </div>
        ))}
      </Card>
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
        title: "提出する内容を確認する",
        type: "report",
        values: { target: null, message_ids: chosen },
        fields: [{ name: "reason", label: "通報の理由", type: "textarea" }],
        note: (
          <>
            <p>
              以下の{result.evidence.length}
              件だけを管理者に提出します。周辺の会話が必要な場合は、戻って選択に追加してください。
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
        submit: "この内容で通報する",
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
            {room.kind === "dm" ? "個別チャット" : "メンバーだけの会話"}
          </small>
        </div>
        <div className="actions">
          {room.kind === "group" && (
            <button className="quiet" onClick={onInvite}>
              招待
            </button>
          )}
          <button
            onClick={onVoice}
            disabled={!voiceAvailable}
            title={
              voiceAvailable ? "音声ルームに参加" : "音声サービスの接続準備中"
            }
          >
            <Icon name="voice" />
            通話
          </button>
          <button
            className="quiet"
            onClick={() => {
              setReportMode(!reportMode);
              setChosen([]);
            }}
          >
            {reportMode ? "選択を終了" : "通報"}
          </button>
        </div>
      </div>
      <label className="chat-search">
        <span className="sr-only">この会話を検索</span>
        <input
          type="search"
          placeholder="この会話を検索"
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
            以前のメッセージ
          </button>
        )}
        {!messages.length ? (
          <Empty>
            {query
              ? "一致するメッセージはありません。"
              : "最初のメッセージを送ってみましょう。"}
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
                  aria-label={`${m.author_name}のメッセージを通報に含める`}
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
                          title: "メッセージを削除する",
                          type: "message_delete",
                          values: { id: m.id },
                          note: (
                            <p>
                              この投稿を会話から削除します。提出済みの通報資料には残る場合があります。
                            </p>
                          ),
                          submit: "削除する",
                        })
                      }
                    >
                      削除
                    </button>
                  )}
                </div>
                <p>{m.deleted_at ? <em>削除されたメッセージ</em> : m.body}</p>
              </div>
            </article>
          ))
        )}
      </div>
      {reportMode ? (
        <div className="chat-compose">
          <span>{chosen.length}件選択</span>
          <button
            disabled={!chosen.length || chosen.length > 30}
            onClick={() => void report()}
          >
            提出内容を確認
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
            メッセージ
          </label>
          <textarea
            id="message-draft"
            placeholder="メッセージを入力…"
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
            {busy ? "送信中" : "送信"}
          </button>
        </form>
      )}
      {room.kind === "group" && (
        <button
          className="quiet"
          onClick={() =>
            open({
              title: "グループから退出する",
              type: "room_leave",
              values: { room: room.id },
              note: <p>退出後、この会話の閲覧と音声参加はできなくなります。</p>,
              submit: "退出する",
            })
          }
        >
          このグループから退出
        </button>
      )}
    </section>
  );
}
