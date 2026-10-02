import { useEffect, useState, type ReactNode } from "react";
import { api, date, money, type Data } from "./api";
import { useApp } from "./App";
import { ActionForm, Card, Empty, Icon, Status, type Field } from "./ui";
export { Social } from "./social";
const rows = (data: Data, key: string): Data[] => data[key] ?? [];
const nameField: Field = { name: "name", label: "名前", max: 64 };
const playerField: Field = {
  name: "target",
  label: "プレイヤー",
  type: "player",
};
const visibilities = [
  { value: "private", label: "自分と管理者のみ" },
  { value: "invite", label: "招待した人" },
  { value: "public", label: "全員に公開" },
];
function Actions({ children }: { children: ReactNode }) {
  return <div className="actions">{children}</div>;
}
function Row({
  children,
  actions,
}: {
  children: ReactNode;
  actions?: ReactNode;
}) {
  return (
    <div className="list-row">
      <div>{children}</div>
      {actions && <Actions>{actions}</Actions>}
    </div>
  );
}
function List({
  values,
  empty,
  render,
}: {
  values: Data[];
  empty: string;
  render: (item: Data) => ReactNode;
}) {
  return values.length ? (
    <div className="list">{values.map(render)}</div>
  ) : (
    <Empty>{empty}</Empty>
  );
}

export function Home({ data }: { data: Data }) {
  const { me, go, act } = useApp();
  const kinds: Record<string, string> = {
    room: "グループチャット",
    team: "チーム",
    party: "パーティー",
    community: "コミュニティ",
    server: "サーバー",
    teleport: "テレポート",
  };
  const notices: Record<string, string> = {
    invitation: "招待が届きました",
    invitation_response: "招待への回答",
    friend_request: "フレンド申請",
    friend_response: "フレンド申請への回答",
    message: "新しいメッセージ",
    transfer: "コインを受け取りました",
    market_sale: "出品が購入されました",
    achievement: "実績を達成しました",
    job_finished: "処理結果が届きました",
    link_candidate: "アカウント連携の確認",
  };
  return (
    <>
      <section className="welcome">
        <div>
          <p className="eyebrow">ログイン中のアカウント</p>
          <h2>{me.account.name}</h2>
          <p>招待、通知、実行した操作の結果を確認できます。</p>
          <button className="primary" onClick={() => go("play")}>
            サーバー一覧を開く <Icon name="arrow" />
          </button>
        </div>
        <div className="welcome-side">
          <span>利用ランク</span>
          <strong>{me.account.rank.name}</strong>
          <button className="quiet" onClick={() => go("settings")}>
            アカウント設定
          </button>
        </div>
      </section>
      <div className="grid two">
        <Card title="届いている招待">
          <List
            values={rows(data, "invitations")}
            empty="未回答の招待はありません。"
            render={(i) => (
              <Row
                key={i.id}
                actions={
                  <>
                    <button
                      className="primary small"
                      onClick={() =>
                        act("invite_respond", { id: i.id, accept: true })
                      }
                    >
                      承諾
                    </button>
                    <button
                      className="quiet"
                      onClick={() =>
                        act("invite_respond", { id: i.id, accept: false })
                      }
                    >
                      断る
                    </button>
                  </>
                }
              >
                <strong>{i.sender_name}</strong>
                <p>{kinds[i.kind] ?? i.kind}への招待</p>
                <small>{date(i.created_at)}</small>
              </Row>
            )}
          />
        </Card>
        <Card
          title="お知らせ"
          action={
            rows(data, "notifications").length ? (
              <button
                className="quiet"
                onClick={() =>
                  act("notifications_read", {
                    through: Math.max(
                      ...data.notifications.map((n: Data) => n.id),
                    ),
                  })
                }
              >
                すべて既読
              </button>
            ) : undefined
          }
        >
          <List
            values={rows(data, "notifications")}
            empty="お知らせはありません。"
            render={(n) => (
              <Row
                key={n.id}
                actions={
                  !n.read_at ? (
                    <span className="unread-dot" aria-label="未読" />
                  ) : undefined
                }
              >
                <strong>{notices[n.kind] ?? "更新がありました"}</strong>
                <small>{date(n.created_at)}</small>
                {n.body?.amount && <p>{money(n.body.amount)} コイン</p>}
                {n.body?.state && <Status value={n.body.state} />}
              </Row>
            )}
          />
        </Card>
      </div>
      <Card title="最近の操作">
        <List
          values={rows(data, "jobs")}
          empty="操作履歴はありません。サーバーの起動などを実行すると、処理結果を表示します。"
          render={(j) => (
            <Row key={j.id} actions={<Status value={j.state} />}>
              <strong>{j.progress?.message ?? j.kind}</strong>
              <small>{date(j.created_at)}</small>
              {j.error && <p className="error">{j.error}</p>}
            </Row>
          )}
        />
      </Card>
    </>
  );
}

export function Play({ data }: { data: Data }) {
  const { act } = useApp();
  const servers = rows(data, "servers");
  return (
    <>
      <p className="intro">
        ゲーム内のロビーから参加することもできます。休止中のサーバーは、参加時に起動します。
      </p>
      {servers.length ? (
        <div className="grid three">
          {servers.map((s) => (
            <section
              className={`server-card ${s.kind === "official" ? "official" : ""}`}
              key={s.id}
            >
              <div className="server-card-top">
                <span className="eyebrow">
                  {s.kind === "official"
                    ? "公式SMP"
                    : s.kind === "lobby"
                      ? "ロビー"
                      : "ユーザーサーバー"}
                </span>
                <Status value={s.observed} />
              </div>
              <h2>{s.name}</h2>
              <p>
                {s.software} · {s.version}
              </p>
              <div className="server-meta">
                <span>{s.players} 人が接続中</span>
                <span>
                  {s.capabilities?.bedrock ? "Java / Bedrock" : "Java"}
                </span>
              </div>
              {s.error && <p className="error">{s.error}</p>}
              {s.capabilities?.client_mods && (
                <p className="notice">指定されたクライアントMODが必要です。</p>
              )}
              <button
                className="primary wide"
                disabled={!s.capabilities?.proxy_join || s.maintenance}
                onClick={() => act("server_join", { id: s.id })}
              >
                {s.maintenance
                  ? "メンテナンス中"
                  : !s.capabilities?.proxy_join
                    ? "接続設定を確認中"
                    : s.observed === "running"
                      ? "参加する"
                      : "起動して参加する"}
              </button>
            </section>
          ))}
        </div>
      ) : (
        <Empty>参加できるサーバーはありません。</Empty>
      )}
      <div className="note-box">
        <Icon name="life" />
        <div>
          <strong>公式SMPの初回参加と再接続</strong>
          <p>
            初回参加時は、ほかのプレイヤーの開始地点や保護地から10,000ブロック以上離れた安全な場所に移動します。ログイン時はロビーに入り、SMPを選択すると前回の有効な位置に戻ります。
          </p>
        </div>
      </div>
    </>
  );
}

export function Life({ data }: { data: Data }) {
  const { me, open, act } = useApp();
  const owners = rows(data, "owners");
  const ownerField: Field = {
    name: "owner",
    label: "所有者",
    type: "select",
    options: owners.map((o) => ({ value: o.id, label: o.name })),
  };
  return (
    <>
      <div className="grid two">
        {owners.map((o) => (
          <section className="balance-card" key={o.id}>
            <p>
              {o.kind === "team" ? "チームの共有資産" : "個人資産"} · {o.name}
            </p>
            <strong>
              {money(o.wallet.balance - o.wallet.reserved)} <span>コイン</span>
            </strong>
            {o.wallet.reserved > 0 && (
              <small>準備中の予約: {money(o.wallet.reserved)} コイン</small>
            )}
            <div className="land-meter">
              <span>保護した土地</span>
              <strong>
                {o.used_chunks} / {o.land.chunks} チャンク
              </strong>
            </div>
            <progress value={o.used_chunks} max={o.land.chunks} />
            <button
              className="quiet"
              onClick={() =>
                open({
                  title: "コインを送る",
                  type: "wallet_transfer",
                  values: { owner: o.id },
                  fields: [
                    playerField,
                    {
                      name: "amount",
                      label: "送るコイン",
                      type: "number",
                      min: 1,
                      max: 1000000000000,
                    },
                  ],
                  submit: "送金する",
                })
              }
            >
              送金する
            </button>
          </section>
        ))}
      </div>
      <Card
        title="保護した土地"
        action={
          <button
            className="primary small"
            onClick={() =>
              open({
                title: "土地を保護する",
                type: "claim_create",
                fields: [
                  ownerField,
                  nameField,
                  ...["min_x", "min_z", "max_x", "max_z"].map(
                    (name, i): Field => ({
                      name,
                      label: [
                        "西端のチャンクX",
                        "北端のチャンクZ",
                        "東端のチャンクX",
                        "南端のチャンクZ",
                      ][i],
                      type: "number",
                      value: 0,
                      min: -1800000,
                      max: 1800000,
                    }),
                  ),
                ],
                note: (
                  <p>
                    生活ワールドで、1チャンク＝16×16ブロック単位の土地を保護します。個人の初期枠は4チャンクです。実際の保護が反映されるまでは「準備中」と表示されます。
                  </p>
                ),
                submit: "保護を予約する",
              })
            }
          >
            <Icon name="plus" />
            土地を保護
          </button>
        }
      >
        <List
          values={rows(data, "claims")}
          empty="保護している土地はありません。「土地を保護」から登録できます。"
          render={(c) => (
            <Row
              key={c.id}
              actions={
                <>
                  <Status value={c.state} />
                  <button
                    className="quiet danger"
                    disabled={c.state !== "active"}
                    onClick={() =>
                      open({
                        title: "土地の保護を解除する",
                        type: "claim_release",
                        values: { id: c.id },
                        note: (
                          <p>
                            「{c.name}
                            」の保護を解除します。中の建物は残り、ほかの人も変更できるようになります。
                          </p>
                        ),
                        submit: "保護を解除する",
                      })
                    }
                  >
                    解除
                  </button>
                </>
              }
            >
              <strong>{c.name}</strong>
              <p>
                {c.chunks} チャンク · X {c.min_x}〜{c.max_x} / Z {c.min_z}〜
                {c.max_z}
              </p>
            </Row>
          )}
        />
      </Card>
      <div className="grid two">
        <Card
          title="ホーム"
          action={
            <button
              className="quiet"
              onClick={() =>
                open({
                  title: "現在地をホームにする",
                  type: "home_set",
                  fields: [{ name: "name", label: "ホーム名", max: 32 }],
                  note: (
                    <p>
                      公式SMPで立っている場所を保存します。初期枠は3つです。
                    </p>
                  ),
                  submit: "現在地を保存",
                })
              }
            >
              現在地を追加
            </button>
          }
        >
          <List
            values={rows(data, "homes")}
            empty="ゲームに接続して、よく戻る場所をホームに登録できます。"
            render={(h) => (
              <Row
                key={h.id}
                actions={
                  <>
                    <button onClick={() => act("home_travel", { id: h.id })}>
                      移動
                    </button>
                    <button
                      className="quiet"
                      onClick={() =>
                        open({
                          title: "ホームを削除する",
                          type: "home_delete",
                          values: { id: h.id },
                          note: <p>「{h.name}」を削除します。</p>,
                          submit: "削除する",
                        })
                      }
                    >
                      削除
                    </button>
                  </>
                }
              >
                <strong>{h.name}</strong>
              </Row>
            )}
          />
        </Card>
        <Card title="待ち合わせ">
          <p>相手が承諾したときにだけ、その人のところへ移動します。</p>
          <button
            onClick={() =>
              open({
                title: "テレポートをお願いする",
                type: "teleport_request",
                fields: [playerField],
                note: (
                  <p>
                    お互いが公式SMPに接続している必要があります。PvP直後の30秒間は移動できません。
                  </p>
                ),
                submit: "申請を送る",
              })
            }
          >
            プレイヤーを選ぶ
          </button>
        </Card>
      </div>
      <Card title="実績">
        <div className="grid three">
          {rows(data, "achievements").map((a) => (
            <div
              className={`achievement ${a.earned_at ? "earned" : ""}`}
              key={a.key}
            >
              <span className="eyebrow">{a.team ? "TEAM" : "PERSONAL"}</span>
              <h3>{a.title}</h3>
              <p>{a.description}</p>
              <progress value={a.progress} max={a.target} />
              <small>
                {money(a.progress)} / {money(a.target)}{" "}
                {a.earned_at ? "· 達成済み" : ""}
              </small>
              <div className="rewards">
                {a.land_chunks > 0 && <span>土地 +{a.land_chunks}</span>}
                {a.coins > 0 && <span>{money(a.coins)} コイン</span>}
              </div>
            </div>
          ))}
        </div>
      </Card>
      <Card title="コインの記録">
        <List
          values={rows(data, "ledger")}
          empty="コインの取引履歴はありません。"
          render={(l) => (
            <Row
              key={`${l.id}-${l.owner}`}
              actions={
                <strong className={l.amount > 0 ? "positive" : ""}>
                  {l.amount > 0 ? "+" : ""}
                  {money(l.amount)}
                </strong>
              }
            >
              <strong>
                {(
                  {
                    transfer: "送金",
                    market: "マーケット",
                    npc: "素材の売却",
                    achievement: "実績報酬",
                    adventure: "冒険の準備",
                  } as Data
                )[l.kind] ?? l.kind}
              </strong>
              <small>{date(l.created_at)}</small>
            </Row>
          )}
        />
      </Card>
    </>
  );
}

export function Market({ data }: { data: Data }) {
  const { me, open, act, send } = useApp();
  const [kind, setKind] = useState("all");
  const [owners, setOwners] = useState<Data[]>([]);
  const [claims, setClaims] = useState<Data[]>([]);
  const [preview, setPreview] = useState<Data | null>(null);
  useEffect(() => {
    api("/api/v1/view/life").then((v) => {
      setOwners(v.owners);
      setClaims(v.claims);
    });
  }, []);
  const ownerField: Field = {
    name: "owner",
    label: "所有者",
    type: "select",
    options: owners.map((o) => ({ value: o.id, label: o.name })),
  };
  const claimField: Field = {
    name: "claim_id",
    label: "土地",
    type: "select",
    options: claims
      .filter((c) => c.state === "active")
      .map((c) => ({ value: c.id, label: c.name })),
  };
  const coordinateFields: Field[] = [
    { name: "x", label: "原点 X", type: "number", value: 0 },
    { name: "y", label: "原点 Y", type: "number", value: 64 },
    { name: "z", label: "原点 Z", type: "number", value: 0 },
    {
      name: "rotation",
      label: "向き",
      type: "select",
      options: [0, 90, 180, 270].map((n) => ({
        value: String(n),
        label: `${n}°`,
      })),
    },
  ];
  function capture() {
    open({
      title: "商品を預託する",
      fields: [
        ownerField,
        {
          name: "kind",
          label: "種類",
          type: "select",
          options: [
            { value: "items", label: "手に持ったアイテム" },
            { value: "building", label: "選択した建物を梱包" },
            { value: "land", label: "土地と建物をそのまま売る" },
          ],
        },
        { name: "title", label: "名前", max: 100 },
        { ...claimField, required: false },
        {
          name: "include_contents",
          label: "容器の中身を含める",
          type: "checkbox",
        },
      ],
      note: (
        <p>
          建物はゲーム内で範囲を選んでから操作してください。梱包では元の建物を撤去します。動物・村人・装飾も含まれ、購入者が一度だけ設置できます。中身を含めない容器は先に空にしてください。
        </p>
      ),
      submit: "預託を開始",
      action: async (v) => {
        const { claim_id, ...other } = v;
        await send("asset_capture", {
          ...other,
          selection: { claim_id: claim_id || null },
        });
      },
    });
  }
  function placement(asset: Data) {
    open({
      title: "建物の設置プレビュー",
      fields: [claimField, ...coordinateFields],
      note: (
        <p>
          保護地内で設置範囲を選びます。地形や建物との衝突を確認してから、確定できます。
        </p>
      ),
      submit: "プレビューする",
      action: async (v) => {
        const result = await send("asset_place", {
          id: asset.id,
          placement: { ...v, rotation: Number(v.rotation), preview: true },
        });
        setPreview({
          asset_id: asset.id,
          job_id: result.job_id,
          placement: { ...v, rotation: Number(v.rotation) },
        });
      },
    });
  }
  useEffect(() => {
    if (!preview || preview.result) return;
    const timer = setInterval(() => {
      api(`/api/v1/jobs/${preview.job_id}`).then((j) => {
        if (j.state === "succeeded")
          setPreview((p) => (p ? { ...p, result: j.result } : null));
        if (j.state === "failed") setPreview(null);
      });
    }, 2500);
    return () => clearInterval(timer);
  }, [preview]);
  const mine = owners.map((o) => o.id);
  const listings = rows(data, "listings").filter(
    (l) => kind === "all" || l.kind === kind,
  );
  return (
    <>
      <div className="section-toolbar">
        <p>預託済みの現物を取引します。手数料は売却額の5%。</p>
        <button className="primary" onClick={capture}>
          <Icon name="plus" />
          出品の準備
        </button>
      </div>
      <div className="tabs" role="group" aria-label="商品の種類">
        {[
          ["all", "すべて"],
          ["items", "アイテム"],
          ["building", "梱包した建物"],
          ["land", "土地と建物"],
        ].map(([v, n]) => (
          <button
            key={v}
            className={v === kind ? "selected" : ""}
            onClick={() => setKind(v)}
          >
            {n}
          </button>
        ))}
      </div>
      {listings.length ? (
        <div className="grid three">
          {listings.map((l) => (
            <section className="market-card" key={l.id}>
              <div className="market-kind">
                <Icon name={l.kind === "items" ? "market" : "life"} />
                <span>
                  {l.kind === "items"
                    ? "アイテム"
                    : l.kind === "building"
                      ? "一度だけ設置できる建物"
                      : "土地と建物"}
                </span>
              </div>
              <h2>{l.title}</h2>
              <p>{l.seller_name}</p>
              <Manifest value={l.manifest} />
              <div className="price">
                {money(l.price)} <small>コイン</small>
              </div>
              {mine.includes(l.seller) ? (
                <button
                  className="wide"
                  onClick={() =>
                    open({
                      title: "出品を取り下げる",
                      type: "listing_cancel",
                      values: { id: l.id },
                      note: (
                        <p>
                          取り下げに手数料はかかりません。預けた現物は保管一覧に戻ります。
                        </p>
                      ),
                      submit: "取り下げる",
                    })
                  }
                >
                  出品を取り下げる
                </button>
              ) : (
                <button
                  className="primary wide"
                  onClick={() =>
                    open({
                      title: `「${l.title}」を購入する`,
                      type: "listing_buy",
                      values: { id: l.id },
                      fields: [ownerField],
                      note: (
                        <>
                          <p>
                            {money(l.price)}
                            コインを支払い、所有権を受け取ります。
                          </p>
                          <Manifest value={l.manifest} />
                          <p>
                            {l.kind === "building"
                              ? "設置する材料は含まれています。購入後に自分の保護地で設置してください。"
                              : l.kind === "land"
                                ? "購入する土地も保護枠を使用します。"
                                : "購入後、公式SMP内で受け取れます。"}
                          </p>
                        </>
                      ),
                      submit: `${money(l.price)}コインで購入`,
                    })
                  }
                >
                  購入する
                </button>
              )}
            </section>
          ))}
        </div>
      ) : (
        <Empty>
          この種類の出品はありません。商品の預託完了後に出品できます。
        </Empty>
      )}
      {preview?.result && (
        <Card title="設置プレビューの結果">
          <p>
            {preview.result.clear
              ? "設置範囲に問題はありません。"
              : "設置範囲に障害があります。整地してからやり直してください。"}
          </p>
          <Manifest value={preview.result} />
          <Actions>
            <button
              className="primary"
              disabled={!preview.result.clear}
              onClick={() =>
                open({
                  title: "この場所に建物を設置する",
                  note: (
                    <p>
                      設置すると梱包資産は使用済みになります。再販売する場合は建物を再び梱包します。
                    </p>
                  ),
                  submit: "設置を確定する",
                  action: async () => {
                    await send("asset_place", {
                      id: preview.asset_id,
                      placement: {
                        ...preview.placement,
                        preview: false,
                        preview_hash: preview.result.preview_hash,
                      },
                    });
                    setPreview(null);
                  },
                })
              }
            >
              設置を確定
            </button>
            <button onClick={() => setPreview(null)}>閉じる</button>
          </Actions>
        </Card>
      )}
      <Card title="保管中の資産">
        <List
          values={rows(data, "assets").filter(
            (a) => !["placed", "delivered", "cancelled"].includes(a.state),
          )}
          empty="保管中の資産はありません。購入した商品や預託した資産を表示します。"
          render={(a) => (
            <Row
              key={a.id}
              actions={
                <>
                  <Status value={a.state} />
                  {a.state === "escrowed" && mine.includes(a.owner) && (
                    <>
                      <button
                        onClick={() =>
                          open({
                            title: "価格を付けて出品する",
                            type: "listing_create",
                            values: { asset: a.id },
                            fields: [
                              {
                                name: "price",
                                label: "販売価格（コイン）",
                                type: "number",
                                min: 1,
                                max: 1000000000000,
                              },
                            ],
                            submit: "出品する",
                          })
                        }
                      >
                        出品
                      </button>
                      {a.kind === "building" ? (
                        <button onClick={() => placement(a)}>設置する</button>
                      ) : a.kind === "items" ? (
                        <button
                          onClick={() => act("asset_receive", { id: a.id })}
                        >
                          ゲームで受け取る
                        </button>
                      ) : a.kind === "land" ? (
                        <button
                          onClick={() =>
                            open({
                              title: "土地の預託を解除する",
                              type: "asset_withdraw",
                              values: { id: a.id },
                              note: (
                                <p>
                                  土地と建物を通常利用へ戻します。再び出品する際は内容を確認して預け直してください。
                                </p>
                              ),
                              submit: "預託を解除",
                            })
                          }
                        >
                          預託を解除
                        </button>
                      ) : null}
                    </>
                  )}
                  {a.state === "capturing" &&
                    a.manifest?.required_consents?.includes(me.account.id) && (
                      <button
                        onClick={() =>
                          open({
                            title: "ペットを建物と一緒に譲る",
                            type: "asset_consent",
                            values: {
                              id: a.id,
                              manifest_sha256: a.manifest_sha256,
                            },
                            note: (
                              <>
                                <p>
                                  この建物に含まれる自分のペットの所有権を、建物の新しい所有者に移すことに同意します。
                                </p>
                                <Manifest value={a.manifest} />
                              </>
                            ),
                            submit: "この内容に同意する",
                          })
                        }
                      >
                        飼い主として確認
                      </button>
                    )}
                  {a.state === "capturing" &&
                    mine.includes(a.owner) &&
                    a.manifest_sha256 && (
                      <button
                        className="quiet"
                        onClick={() => act("asset_withdraw", { id: a.id })}
                      >
                        同意待ちの梱包を取り消す
                      </button>
                    )}
                </>
              }
            >
              <strong>{a.title}</strong>
              <Manifest value={a.manifest} />
            </Row>
          )}
        />
      </Card>
      <Card title="素材の買い取り">
        <div className="section-toolbar">
          <div>
            <strong>今日の残り {money(data.npc_remaining)} コイン</strong>
            <small>固定価格 · 毎日 UTC 0:00 に2,000コイン分を補充</small>
          </div>
          <button
            onClick={() =>
              open({
                title: "素材を売る",
                type: "npc_sell",
                fields: [
                  {
                    name: "material",
                    label: "素材",
                    type: "select",
                    options: rows(data, "prices").map((p) => ({
                      value: p.material,
                      label: `${p.material} · 1個 ${p.price}コイン`,
                    })),
                  },
                  {
                    name: "amount",
                    label: "個数",
                    type: "number",
                    min: 1,
                    max: 2304,
                    value: 1,
                  },
                ],
                note: (
                  <p>
                    公式SMPに接続し、手持ちの素材を売ります。素材を回収できた後に入金します。
                  </p>
                ),
                submit: "素材を売る",
              })
            }
          >
            素材を売る
          </button>
        </div>
        <div className="price-list">
          {rows(data, "prices").map((p) => (
            <span key={p.material}>
              {p.material}
              <b>{p.price}</b>
            </span>
          ))}
        </div>
      </Card>
    </>
  );
}
function Manifest({ value }: { value: Data }) {
  const m = value?.summary ?? value ?? {};
  return (
    <div className="manifest">
      {m.dimensions && (
        <p>
          大きさ{" "}
          {Array.isArray(m.dimensions)
            ? m.dimensions.join(" × ")
            : String(m.dimensions)}
        </p>
      )}
      {(m.block_count ?? m.blocks) !== undefined && (
        <p>{money(m.block_count ?? m.blocks)} ブロック</p>
      )}
      {m.material && m.amount !== undefined && (
        <p>
          {m.material} × {m.amount}
        </p>
      )}
      {m.origin && (
        <p>
          原点: {m.origin.join(", ")} · 回転 {m.rotation}°
        </p>
      )}
      {m.footprint && (
        <p>
          設置範囲: X {m.footprint.min_x}〜{m.footprint.max_x} / Y{" "}
          {m.footprint.min_y}〜{m.footprint.max_y} / Z {m.footprint.min_z}〜
          {m.footprint.max_z}
        </p>
      )}
      {m.containers?.length > 0 && (
        <details>
          <summary>収納の中身（{m.containers.length} スタック）</summary>
          <ul>
            {m.containers.map((i: Data, n: number) => (
              <li key={n}>
                {i.name ?? i.material} × {i.amount}
                {i.enchantments && Object.keys(i.enchantments).length > 0
                  ? ` · ${Object.entries(i.enchantments)
                      .map(([k, v]) => `${k} ${v}`)
                      .join(", ")}`
                  : ""}
              </li>
            ))}
          </ul>
        </details>
      )}
      {Array.isArray(m.items) && (
        <ul>
          {m.items.map((i: Data, n: number) => (
            <li key={n}>
              {i.name ?? i.material} × {i.amount}
            </li>
          ))}
        </ul>
      )}
      {m.entities && (
        <ul>
          {m.entities.map((e: Data, n: number) => (
            <li key={n}>
              {e.name ?? e.type}
              {e.trades ? " · 村人の取引を引き継ぎ" : ""}
            </li>
          ))}
        </ul>
      )}
      {m.contents_included !== undefined && (
        <small>容器の中身: {m.contents_included ? "含む" : "含まない"}</small>
      )}
    </div>
  );
}

export function Adventure({ data }: { data: Data }) {
  const { open, act } = useApp();
  return (
    <>
      <section className="adventure-hero">
        <span className="eyebrow">利用条件</span>
        <h2>プライベートエンドの作成</h2>
        <p>
          個人またはパーティー専用のエンドを作成します。インベントリは公式SMPと共通で、入手したアイテムを持ち帰れます。
        </p>
        <div className="adventure-cost">
          <div>
            <strong>1,000</strong>
            <small>コイン</small>
          </div>
          <span>＋</span>
          <div>
            <strong>12</strong>
            <small>エンダーアイ</small>
          </div>
          <div>
            <strong>3時間</strong>
            <small>利用開始から</small>
          </div>
        </div>
        <button
          className="primary"
          onClick={() =>
            open({
              title: "一時エンドを準備する",
              type: "adventure_create",
              note: (
                <>
                  <p>
                    1,000コインとエンダーアイ12個を用意してください。パーティーの場合は全員が公式SMPに接続し、準備完了にする必要があります。参加者は準備開始時に確定します。
                  </p>
                  <p>
                    ワールドが実際に開いてから3時間で閉じます。落としたアイテムは期限までに回収してください。開く前のキャンセルや開始失敗では、コインと素材を返却します。確保済みのアイテムは、ゲーム内の預かり資産から受け取れます。
                  </p>
                </>
              ),
              submit: "コインと素材を予約する",
            })
          }
        >
          冒険を準備する <Icon name="arrow" />
        </button>
      </section>
      <p className="intro">
        常設のネザーとエンドには、準備費用なしで行けます。
      </p>
      <Card title="自分たちの冒険">
        <List
          values={rows(data, "adventures")}
          empty="進行中の冒険はありません。1人、またはパーティーで準備できます。"
          render={(a) => (
            <Row
              key={a.id}
              actions={
                <>
                  <Status value={a.state} />
                  {a.state === "active" && (
                    <button
                      className="primary small"
                      onClick={() => act("adventure_join", { id: a.id })}
                    >
                      冒険へ入る
                    </button>
                  )}
                  {a.can_cancel && (
                    <button
                      onClick={() =>
                        open({
                          title: "準備を取り消す",
                          type: "adventure_cancel",
                          values: { id: a.id },
                          note: (
                            <p>
                              ワールドを閉じ、コインの予約を解除します。確保済みのエンダーアイは預かり資産から受け取れます。返却完了まで操作結果を確認してください。
                            </p>
                          ),
                          submit: "取り消して返却する",
                        })
                      }
                    >
                      取り消す
                    </button>
                  )}
                  {a.can_receive && (
                    <button
                      onClick={() =>
                        act("asset_receive", { id: a.material_asset })
                      }
                    >
                      返却アイテムを受け取る
                    </button>
                  )}
                </>
              }
            >
              <strong>プライベート End</strong>
              <small>準備開始 {date(a.created_at)}</small>
              {a.expires_at && <p>終了日時 {date(a.expires_at)}</p>}
            </Row>
          )}
        />
      </Card>
    </>
  );
}

export function Servers({ data }: { data: Data }) {
  const { me, open, act, send, refresh } = useApp();
  const [uploading, setUploading] = useState("");
  const [error, setError] = useState("");
  async function upload(server: string, file: File) {
    setUploading(server);
    setError("");
    try {
      const form = new FormData();
      form.append("file", file);
      const result = await api(`/api/v1/servers/${server}/artifacts`, {
        method: "POST",
        body: form,
      });
      refresh();
      open({
        title: "保存したファイルを反映する",
        type: "server_install",
        values: { id: server, artifact: result.id },
        fields: [
          {
            name: "path",
            label: "サーバー内の保存先",
            value: result.name.endsWith(".jar") ? "server.jar" : result.name,
          },
        ],
        note: (
          <p>
            アップロードしたファイルは保存済みです。反映するにはサーバーを停止してください。world
            ZIP は指定フォルダーに展開します。
          </p>
        ),
        submit: "ファイルを反映する",
      });
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setUploading("");
    }
  }
  const rank = me.account.rank;
  return (
    <>
      <div className="section-toolbar">
        <div>
          <p>
            作成枠 {rank.server_count} 台 · 同時起動 {rank.concurrent_servers}{" "}
            台
          </p>
          <small>
            稼働メモリ {money(rank.memory_mib)} MiB · CPU{" "}
            {rank.cpu_millis / 1000} コア · 保存容量 {money(rank.storage_mib)}{" "}
            MiB
          </small>
        </div>
        <button
          className="primary"
          disabled={rank.server_count === 0}
          onClick={() =>
            open({
              title: "サーバーを作成する",
              type: "server_create",
              values: { community: null },
              fields: [
                nameField,
                {
                  name: "software",
                  label: "サーバーの種類",
                  type: "select",
                  options: [
                    "paper",
                    "fabric",
                    "forge",
                    "neoforge",
                    "custom",
                  ].map((s) => ({ value: s, label: s })),
                },
                {
                  name: "version",
                  label: "Minecraft バージョン",
                  hint: "使用するJARに合わせて指定してください。",
                },
                {
                  name: "memory_mib",
                  label: "メモリ（MiB）",
                  type: "number",
                  min: 512,
                  max: rank.memory_mib,
                  value: Math.min(2048, rank.memory_mib),
                },
                {
                  name: "cpu_millis",
                  label: "CPU（1コア＝1000）",
                  type: "number",
                  min: 1000,
                  step: 1000,
                  hint: "1000ずつ増やすと1コア追加されます。",
                  max: rank.cpu_millis,
                  value: Math.min(1000, rank.cpu_millis),
                },
                {
                  name: "storage_mib",
                  label: "保存容量（MiB）",
                  type: "number",
                  min: 1024,
                  max: rank.storage_mib,
                  value: Math.min(10240, rank.storage_mib),
                },
                {
                  name: "visibility",
                  label: "公開範囲",
                  type: "select",
                  options: visibilities,
                },
              ],
              note: (
                <p>
                  隔離した環境を作成します。任意のJARやMODは、作成後にアップロードできます。接続対応は実際の構成を確認して表示します。
                </p>
              ),
              submit: "サーバーを作成",
            })
          }
        >
          <Icon name="plus" />
          サーバーを作成
        </button>
      </div>
      {rank.server_count === 0 && (
        <p className="notice">
          現在のランクではサーバー作成が許可されていません。利用したい構成を管理者に伝え、ランクの承認を受けてください。
        </p>
      )}
      {error && (
        <p role="alert" className="error">
          {error}
        </p>
      )}
      <List
        values={rows(data, "servers")}
        empty="管理対象のサーバーはありません。作成後は起動・停止、ファイル、バックアップを管理できます。"
        render={(s) => (
          <Card
            key={s.id}
            title={s.name}
            action={<Status value={s.observed} />}
          >
            <div className="section-toolbar">
              <p>
                {s.software} {s.version} · {s.memory_mib} MiB ·{" "}
                {s.cpu_millis / 1000} コア
              </p>
              <Actions>
                <button
                  className="primary small"
                  disabled={s.desired === "running"}
                  onClick={() => act("server_start", { id: s.id })}
                >
                  起動
                </button>
                <button
                  disabled={s.observed === "stopped" || s.kind === "lobby"}
                  onClick={() =>
                    open({
                      title: "サーバーを停止する",
                      type: "server_stop",
                      values: { id: s.id },
                      note: (
                        <p>
                          「{s.name}
                          」の参加者を退出させ、ワールドを保存して停止します。
                        </p>
                      ),
                      submit: "保存して停止",
                    })
                  }
                >
                  停止
                </button>
                <button
                  className="quiet"
                  onClick={() =>
                    open({
                      title: "サーバー設定",
                      type: "server_configure",
                      values: { id: s.id },
                      fields: [
                        { ...nameField, value: s.name },
                        {
                          name: "visibility",
                          label: "公開範囲",
                          type: "select",
                          value: s.visibility,
                          options: visibilities,
                        },
                      ],
                    })
                  }
                >
                  設定
                </button>
              </Actions>
            </div>
            {s.error && <p className="error">{s.error}</p>}
            <details>
              <summary>コンソールとログ</summary>
              <button onClick={() => act("server_logs", { id: s.id })}>
                最新ログを取得
              </button>
              <ActionForm
                fields={[
                  { name: "line", label: "コンソールコマンド", max: 1024 },
                ]}
                submit="送信する"
                onSubmit={(v) => send("server_console", { id: s.id, ...v })}
              />
            </details>
            <details>
              <summary>ファイル</summary>
              <label className="upload-zone">
                {uploading === s.id
                  ? "アップロードしています…"
                  : "JAR・MOD・プラグイン・ワールドをアップロード"}
                <input
                  type="file"
                  disabled={Boolean(uploading)}
                  onChange={(e) => {
                    const file = e.target.files?.[0];
                    if (file) void upload(s.id, file);
                    e.target.value = "";
                  }}
                />
              </label>
              {s.artifacts?.map((a: Data) => (
                <Row
                  key={a.id}
                  actions={
                    <button
                      onClick={() =>
                        open({
                          title: "ファイルを反映する",
                          type: "server_install",
                          values: { id: s.id, artifact: a.id },
                          fields: [
                            { name: "path", label: "保存先", value: a.name },
                          ],
                          submit: "反映する",
                        })
                      }
                    >
                      反映
                    </button>
                  }
                >
                  <strong>{a.name}</strong>
                  <small>
                    {money(Math.ceil(a.bytes / 1024))} KiB ·{" "}
                    {date(a.created_at)}
                  </small>
                </Row>
              ))}
            </details>
            <details>
              <summary>バックアップ</summary>
              <p>
                復元すると現在のワールドをバックアップ時点へ戻します。復元前に停止してください。
              </p>
              <button onClick={() => act("server_backup", { id: s.id })}>
                バックアップを作成
              </button>
              {s.backups?.map((b: Data) => (
                <Row
                  key={b.id}
                  actions={
                    <>
                      <Status value={b.state} />
                      <button
                        disabled={
                          b.state !== "ready" || s.observed !== "stopped"
                        }
                        onClick={() =>
                          open({
                            title: "バックアップから復元する",
                            type: "server_restore",
                            values: { id: s.id, backup: b.id },
                            note: (
                              <p>
                                「{s.name}」を{date(b.created_at)}
                                の内容に戻します。現在の内容が必要なら、先にバックアップを作成してください。
                              </p>
                            ),
                            submit: "この時点に復元する",
                          })
                        }
                      >
                        復元
                      </button>
                    </>
                  }
                >
                  <strong>{date(b.created_at)}</strong>
                </Row>
              ))}
            </details>
            <details>
              <summary>メンバーと管理権限</summary>
              <button
                onClick={() =>
                  open({
                    title: "メンバーの権限を設定する",
                    type: "server_member",
                    values: { id: s.id },
                    fields: [
                      { ...playerField, name: "member" },
                      {
                        name: "role",
                        label: "権限",
                        type: "select",
                        options: [
                          { value: "guest", label: "参加者" },
                          { value: "operator", label: "起動・停止・ログ" },
                          { value: "administrator", label: "共同管理者" },
                        ],
                      },
                    ],
                  })
                }
              >
                メンバーを追加
              </button>
              {s.members?.map((m: Data) => (
                <Row
                  key={m.account_id}
                  actions={
                    <button
                      className="quiet"
                      onClick={() =>
                        act("server_member", {
                          id: s.id,
                          member: m.account_id,
                          role: null,
                        })
                      }
                    >
                      外す
                    </button>
                  }
                >
                  <strong>{m.name}</strong>
                  <small>{m.role}</small>
                </Row>
              ))}
            </details>
          </Card>
        )}
      />
    </>
  );
}

export function Settings({ data }: { data: Data }) {
  const { me, send, act, open } = useApp();
  const [code, setCode] = useState("");
  const policies = [
    { value: "friends", label: "フレンドのみ" },
    { value: "everyone", label: "全員" },
    { value: "none", label: "公開しない / 受け取らない" },
  ];
  return (
    <>
      <div className="grid two">
        <Card title="プロフィールと公開範囲">
          <ActionForm
            fields={[
              {
                name: "display_name",
                label: "表示名",
                value: me.account.name,
                max: 64,
              },
              {
                name: "dm_policy",
                label: "DMを受け取る相手",
                type: "select",
                value: me.account.dm_policy,
                options: policies,
              },
              {
                name: "activity_policy",
                label: "プレイ状況を見せる相手",
                type: "select",
                value: me.account.activity_policy,
                options: policies,
              },
            ]}
            onSubmit={(v) => send("privacy", v)}
          />
        </Card>
        <Card title="ゲームアカウントを連携">
          <p>
            Web とゲーム、Java と Bedrock
            を一つのアカウントにまとめます。すでに両方で遊んでいる場合は、使うプレイデータを1つ選びます。
          </p>
          <ul>
            {me.account.identities?.map((i: Data, n: number) => (
              <li key={n}>
                {i.issuer === "java"
                  ? "Java"
                  : i.issuer === "bedrock"
                    ? "Bedrock"
                    : "Web"}{" "}
                · {i.display_name}
              </li>
            ))}
          </ul>
          <button
            onClick={() =>
              open({
                title: "連携コードを発行する",
                note: (
                  <p>
                    もう一方のアカウントで、このコードを入力してください。コードは10分で失効します。
                  </p>
                ),
                submit: "コードを発行",
                action: async () => {
                  const result = await send("link_begin");
                  setCode(result.code);
                },
              })
            }
          >
            連携コードを発行
          </button>
          {code && (
            <p className="link-code">
              <code>{code}</code>
              <small>もう一方のアカウントで入力</small>
            </p>
          )}
          <ActionForm
            fields={[{ name: "code", label: "受け取った連携コード" }]}
            submit="このアカウントと連携"
            onSubmit={(v) => send("link_present", v)}
          />
          {rows(data, "links").map((l) => (
            <div className="link-request" key={l.id}>
              <Status value={l.state} />
              {l.initiator === me.account.id &&
                l.candidate &&
                l.state === "pending" && (
                  <>
                    <p>
                      使い続けるプレイデータを選んでください。もう一方はアーカイブされ、通貨やアイテムは合算しません。
                    </p>
                    {l.profiles?.map((p: Data) => (
                      <button
                        key={p.id}
                        onClick={() =>
                          open({
                            title: "使うプレイデータを確定する",
                            type: "link_confirm",
                            values: { id: l.id, selected_profile: p.id },
                            note: (
                              <p>
                                「{p.name}
                                」のプレイデータを使用します。もう一方のデータはアーカイブします。出品や冒険を終えてから確定してください。連携のため両アカウントのゲーム接続を切断します。
                                {!p.native_uuid &&
                                  "このデータにはゲームの持ち物や実績がないため、新しく開始します。"}
                              </p>
                            ),
                            submit: "このデータを使って連携する",
                          })
                        }
                      >
                        {p.name} · {money(p.wallet.balance)}コイン
                      </button>
                    ))}
                  </>
                )}
            </div>
          ))}
        </Card>
      </div>
      <Card title="ブロックしているプレイヤー">
        <List
          values={rows(data, "blocks")}
          empty="ブロックしているプレイヤーはいません。"
          render={(b) => (
            <Row
              key={b.id}
              actions={
                <button
                  onClick={() => act("block", { target: b.id, blocked: false })}
                >
                  解除
                </button>
              }
            >
              <strong>{b.name}</strong>
            </Row>
          )}
        />
      </Card>
      <Card title="提出した通報">
        <p>管理者には、あなたが選んで提出したメッセージだけを渡します。</p>
        <List
          values={rows(data, "reports")}
          empty="提出した通報はありません。チャットから内容を選んで通報できます。"
          render={(r) => (
            <Row key={r.id} actions={<span>{r.status}</span>}>
              <strong>{r.reason}</strong>
              <small>{date(r.created_at)}</small>
              {r.resolution && <p>{r.resolution}</p>}
            </Row>
          )}
        />
      </Card>
      <button
        className="quiet danger"
        onClick={() =>
          open({
            title: "ログアウトする",
            submit: "ログアウト",
            action: async () => {
              await api("/auth/logout", { method: "POST", body: "{}" });
              location.assign("/");
            },
          })
        }
      >
        この端末からログアウト
      </button>
    </>
  );
}

export function Admin({ data }: { data: Data }) {
  const { me, open, act } = useApp();
  if (!me.account.administrator) return <Empty>運営権限が必要です。</Empty>;
  return (
    <>
      <Card title="通報への対応">
        <List
          values={rows(data, "reports")}
          empty="対応待ちの通報はありません。"
          render={(r) => (
            <Row
              key={r.id}
              actions={
                <button
                  onClick={() =>
                    open({
                      title: "通報の提出内容を確認する",
                      note: (
                        <p>
                          閲覧者・日時を監査記録に残します。提出された範囲だけを表示します。
                        </p>
                      ),
                      submit: "内容を確認する",
                      action: async () => {
                        const report = await api(`/api/v1/reports/${r.id}`);
                        queueMicrotask(() =>
                          open({
                            title: "通報の内容",
                            type: "report_resolve",
                            values: { id: r.id },
                            fields: [
                              {
                                name: "status",
                                label: "対応",
                                type: "select",
                                options: [
                                  { value: "investigating", label: "調査中" },
                                  { value: "resolved", label: "解決" },
                                  { value: "dismissed", label: "対応不要" },
                                ],
                              },
                              {
                                name: "resolution",
                                label: "対応内容",
                                type: "textarea",
                              },
                            ],
                            note: (
                              <>
                                <p>{report.reason}</p>
                                {report.evidence.map((m: Data) => (
                                  <blockquote key={m.id}>
                                    <strong>{m.author_name}</strong>
                                    <p>{m.body}</p>
                                    <small>{date(m.created_at)}</small>
                                  </blockquote>
                                ))}
                              </>
                            ),
                          }),
                        );
                      },
                    })
                  }
                >
                  提出内容を開く
                </button>
              }
            >
              <strong>通報 · {date(r.created_at)}</strong>
              <small>{r.status}</small>
            </Row>
          )}
        />
      </Card>
      <Card title="ホスティングの信頼ランク">
        <p>
          プレイ時間や実績とは別に、運営が利用枠を承認します。降格しても保存データは削除しません。
        </p>
        <Actions>
          <button
            onClick={() =>
              open({
                title: "ランクを割り当てる",
                type: "rank_set",
                fields: [
                  playerField,
                  {
                    name: "rank",
                    label: "ランク番号",
                    type: "number",
                    min: 0,
                    max: 32767,
                  },
                ],
              })
            }
          >
            プレイヤーに割り当て
          </button>
          <button
            onClick={() =>
              open({
                title: "ランクの上限を設定する",
                type: "rank_configure",
                fields: [
                  {
                    name: "id",
                    label: "ランク番号",
                    type: "number",
                    min: 0,
                    max: 32767,
                  },
                  nameField,
                  ...[
                    "server_count",
                    "concurrent_servers",
                    "memory_mib",
                    "cpu_millis",
                    "storage_mib",
                  ].map(
                    (name, i): Field => ({
                      name,
                      label: [
                        "サーバー作成数",
                        "同時起動数",
                        "稼働メモリ（MiB）",
                        "CPU（1コア＝1000）",
                        "保存容量（MiB）",
                      ][i],
                      type: "number",
                      min: 0,
                      value: 0,
                    }),
                  ),
                ],
              })
            }
          >
            ランクを設定
          </button>
          <button
            onClick={() =>
              open({
                title: "利用制限を設定する",
                type: "ban",
                fields: [
                  playerField,
                  {
                    name: "hours",
                    label: "制限する時間（0で解除）",
                    type: "number",
                    min: 0,
                    max: 876000,
                    value: 24,
                  },
                  { name: "reason", label: "理由", type: "textarea" },
                ],
                submit: "制限を設定",
              })
            }
          >
            利用制限
          </button>
        </Actions>
        <List
          values={rows(data, "ranks")}
          empty="ランクがありません。"
          render={(r) => (
            <Row key={r.id}>
              <strong>
                {r.id} · {r.name}
              </strong>
              <p>
                作成 {r.server_count} / 同時 {r.concurrent_servers} ·{" "}
                {money(r.memory_mib)} MiB · {r.cpu_millis / 1000} コア · 保存{" "}
                {money(r.storage_mib)} MiB
              </p>
            </Row>
          )}
        />
      </Card>
      <Card
        title="公式バックアップ"
        action={
          <button onClick={() => act("official_backup")}>公式全体を保存</button>
        }
      >
        <p>ワールド・インベントリ・土地・台帳・保管資産を一緒に保存します。</p>
        <p>
          {data.backup_policy?.enabled
            ? `毎日 ${String((data.backup_policy.hour_utc + 9) % 24).padStart(2, "0")}:00（日本時間）に自動保存。成功した日次7世代・週次4世代を残します。`
            : "自動保存は現在無効です。"}
          手動保存と固定した保存は自動整理の対象になりません。
        </p>
        <p>
          最後の保存完了：
          {data.backup_policy?.last_completed_at
            ? date(data.backup_policy.last_completed_at)
            : "記録なし"}
        </p>
        <List
          values={rows(data, "backups")}
          empty="バックアップの記録はありません。"
          render={(b) => (
            <Row
              key={b.id}
              actions={
                <>
                  <Status value={b.state} />
                  {b.kind === "official" &&
                    b.state === "ready" &&
                    b.scheduled_for && (
                      <button
                        onClick={() =>
                          act("backup_pin", { id: b.id, pinned: !b.pinned })
                        }
                      >
                        {b.pinned ? "固定を解除" : "この保存を固定"}
                      </button>
                    )}
                </>
              }
            >
              <strong>
                {b.kind === "official" ? "公式全体" : "個人サーバー"}
              </strong>
              <small>{date(b.created_at)}</small>
              {b.kind === "official" && (
                <p>
                  {b.scheduled_for ? "日次の自動保存" : "手動保存"}
                  {b.pinned ? " · 固定中" : ""}
                  {b.completed_at ? ` · 完了 ${date(b.completed_at)}` : ""}
                </p>
              )}
              {b.error && <p className="error">{b.error}</p>}
            </Row>
          )}
        />
      </Card>
      <Card title="確認が必要な処理">
        <List
          values={rows(data, "jobs")}
          empty="確認が必要な処理はありません。"
          render={(j) => (
            <Row key={j.id} actions={<Status value={j.state} />}>
              <strong>{j.kind}</strong>
              <p>{j.error ?? j.progress?.message}</p>
              <small>{date(j.updated_at)}</small>
            </Row>
          )}
        />
      </Card>
      <Card title="監査記録">
        <List
          values={rows(data, "audit")}
          empty="管理操作の記録はありません。"
          render={(a) => (
            <Row key={a.id}>
              <strong>{a.action}</strong>
              <small>
                {date(a.created_at)} · {a.actor ?? a.service}
              </small>
            </Row>
          )}
        />
      </Card>
    </>
  );
}
