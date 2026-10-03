import { t, translateError } from "./i18n";
import { useEffect, useState, type ReactNode } from "react";
import { jobTitle, api, date, money, type Data } from "./api";
import { useApp, LanguagePicker, PageBlock } from "./App";
import { JobList } from "./pages";
import { NotificationItem } from "./jobs";
import { ActionForm, Card, Empty, Icon, Status, type Field } from "./ui";
export { Social } from "./social";
const rows = (data: Data, key: string): Data[] => data[key] ?? [];
const placementPreviews = new Map<string, Data>();
const nameField = (): Field => ({ name: "name", label: t("Name"), max: 64 });
const playerField = (): Field => ({
  name: "target",
  label: t("Player"),
  type: "player",
});
const visibilities = () => [
  { value: "private", label: t("You and administrators") },
  { value: "invite", label: t("Invited players") },
  { value: "public", label: t("Public") },
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
  const { me, go, act, route } = useApp();
  const kinds: Record<string, string> = {
    room: t("Group chat"),
    team: t("Team"),
    party: t("Party"),
    community: t("Community"),
    server: t("Server"),
    teleport: t("Teleport"),
  };
  const notices: Record<string, string> = {
    invitation: t("New invitation"),
    invitation_response: t("Invitation response"),
    friend_request: t("Friend request"),
    friend_response: t("Friend request response"),
    message: t("New message"),
    transfer: t("Coins received"),
    market_sale: t("Listing sold"),
    achievement: t("Achievement unlocked"),
    job_finished: t("Action completed"),
    link_candidate: t("Account linking confirmation"),
  };
  return (
    <>
      <PageBlock id="overview">
        <div className="home-summary">
          <p>
            {t("Signed in as")} {me.account.name}
          </p>
          <a className="button primary" href="#/servers">
            {t("Browse servers")}
          </a>
          <a className="card overview-link" href="#/home/invitations">
            <strong>{t("Invitations")}</strong>
            <span>
              {money(data.counts?.invitations ?? data.invitations?.length ?? 0)}
            </span>
          </a>
          <a className="card overview-link" href="#/home/notifications">
            <strong>{t("Unread notifications")}</strong>
            <span>
              {money(
                data.counts?.notifications ??
                  data.notifications?.filter((n: Data) => !n.read_at).length ??
                  0,
              )}
            </span>
          </a>
          <a className="card overview-link" href="#/timeline">
            <strong>{t("Timeline")}</strong>
            <span>{t("Messages and updates")}</span>
          </a>
        </div>
      </PageBlock>
      <div className="grid two">
        <PageBlock id="invitations">
          <Card title={t("Invitations")}>
            <List
              values={rows(data, "invitations")}
              empty={t("No invitations to respond to.")}
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
                        {t("Accept")}
                      </button>
                      <button
                        className="quiet"
                        onClick={() =>
                          act("invite_respond", { id: i.id, accept: false })
                        }
                      >
                        {t("Decline")}
                      </button>
                    </>
                  }
                >
                  <strong>{i.sender_name}</strong>
                  <p>
                    {kinds[i.kind] ?? i.kind}
                    {t(" invitation")}
                  </p>
                  <small>{date(i.created_at)}</small>
                </Row>
              )}
            />
            {route.component === "home" && (
              <a className="feed-more" href="#/home/invitations">
                {t("View all")}{" "}
                {data.counts?.["invitations"] != null && (
                  <span>({data.counts["invitations"]})</span>
                )}
              </a>
            )}
          </Card>
        </PageBlock>
        <PageBlock id="notifications">
          <Card
            title={t("Notifications")}
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
                  {t("Mark all as read")}
                </button>
              ) : undefined
            }
          >
            <List
              values={rows(data, "notifications")}
              empty={t("No notifications yet.")}
              render={(n) => <NotificationItem key={n.id} notice={n} />}
            />
            {route.component === "home" && (
              <a className="feed-more" href="#/home/notifications">
                {t("View all")}{" "}
                {data.counts?.["notifications"] != null && (
                  <span>({data.counts["notifications"]})</span>
                )}
              </a>
            )}
          </Card>
        </PageBlock>
      </div>
      <PageBlock id="activity">
        <Card title={t("Recent actions")}>
          <JobList jobs={data.jobs} />
          {route.component === "home" && (
            <a className="feed-more" href="#/home/activity">
              {t("View all")}{" "}
              {data.counts?.["jobs"] != null && (
                <span>({data.counts["jobs"]})</span>
              )}
            </a>
          )}
        </Card>
      </PageBlock>
    </>
  );
}

export function Play({
  data,
  overview = false,
}: {
  data: Data;
  overview?: boolean;
}) {
  const { act, isWorking } = useApp();
  const servers = rows(data, "servers");
  return (
    <>
      <p className="intro">
        {t(
          "You can also join from the in-game lobby. Sleeping servers start when you join.",
        )}
      </p>
      {servers.length ? (
        <div className="server-list">
          {servers.map((s) => (
            <article className="server-row" key={s.id}>
              <div className="grow">
                <h2>
                  <a href={"#/servers/" + s.id}>{s.name}</a>
                </h2>
                <p>
                  {s.software} {s.version} · {s.players}
                  {t(" players online")} ·{" "}
                  {s.capabilities?.bedrock ? "Java / Bedrock" : "Java"}
                </p>
                {s.capabilities?.client_mods && (
                  <small>{t("The specified client mods are required.")}</small>
                )}
              </div>
              <Status value={s.observed} />
              <div className="actions">
                <button
                  className="primary"
                  disabled={
                    !s.capabilities?.proxy_join ||
                    s.maintenance ||
                    isWorking("server_join", { id: s.id })
                  }
                  onClick={() => act("server_join", { id: s.id })}
                >
                  {s.maintenance
                    ? t("Under maintenance")
                    : !s.capabilities?.proxy_join
                      ? t("Checking connection settings")
                      : s.observed === "running"
                        ? t("Join")
                        : t("Start and join")}
                </button>
              </div>
            </article>
          ))}
        </div>
      ) : (
        <Empty>{t("No servers available to join.")}</Empty>
      )}
    </>
  );
}

export function Smp({ data }: { data: Data }) {
  return (
    <>
      <Play
        overview
        data={{
          servers: rows(data, "servers").filter((s) => s.kind === "official"),
        }}
      />
      <div className="note-box">
        <Icon name="life" />
        <div>
          <strong>{t("Joining and returning to SMP")}</strong>
          <p>
            {t(
              "Your first spawn is a safe place at least 10,000 blocks from other players’ starting points and claims. You enter the lobby on login; choose SMP to return to your last valid position.",
            )}
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
    label: t("Owner"),
    type: "select",
    options: owners.map((o) => ({ value: o.id, label: o.name })),
  };
  return (
    <>
      <PageBlock id="coins">
        <div className="grid two">
          {owners.map((o) => (
            <section className="balance-card" key={o.id}>
              <p>
                {o.kind === "team" ? t("Team assets") : t("Personal assets")} ·{" "}
                {o.name}
              </p>
              <strong>
                {money(o.wallet.balance - o.wallet.reserved)}{" "}
                <span>{t(" coins")}</span>
              </strong>
              {o.wallet.reserved > 0 && (
                <small>
                  {t("Reserved: ")}
                  {money(o.wallet.reserved)} {t(" coins")}
                </small>
              )}
              <div className="land-meter">
                <span>{t("Protected land")}</span>
                <strong>
                  {o.used_chunks} / {o.land.chunks} {t(" chunks")}
                </strong>
              </div>
              <progress value={o.used_chunks} max={o.land.chunks} />
              <button
                className="quiet"
                onClick={() =>
                  open({
                    title: t("Send coins"),
                    type: "wallet_transfer",
                    values: { owner: o.id },
                    fields: [
                      playerField(),
                      {
                        name: "amount",
                        label: t("Amount"),
                        type: "number",
                        min: 1,
                        max: 1000000000000,
                      },
                    ],
                    submit: t("Send coins now"),
                  })
                }
              >
                {t("Send coins now")}
              </button>
            </section>
          ))}
        </div>
      </PageBlock>
      <PageBlock id="land">
        <Card
          title={t("Protected land")}
          action={
            <button
              className="primary small"
              onClick={() =>
                open({
                  title: t("Protect land"),
                  type: "claim_create",
                  fields: [
                    ownerField,
                    nameField(),
                    ...["min_x", "min_z", "max_x", "max_z"].map(
                      (name, i): Field => ({
                        name,
                        label: [
                          t("West chunk X"),
                          t("North chunk Z"),
                          t("East chunk X"),
                          t("South chunk Z"),
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
                      {t(
                        "Protect land in the survival world in 16 × 16 block chunks. You start with four chunks. A new claim stays pending until protection is applied in-game.",
                      )}
                    </p>
                  ),
                  submit: t("Request protection"),
                })
              }
            >
              <Icon name="plus" />
              {t("Protect land now")}
            </button>
          }
        >
          <List
            values={rows(data, "claims")}
            empty={t(
              "No protected land yet. Choose “Protect land” to make your first claim.",
            )}
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
                          title: t("Release land protection"),
                          type: "claim_release",
                          values: { id: c.id },
                          note: (
                            <p>
                              {t(
                                "Release protection for “{0}”? Buildings remain and other players will be able to edit them.",
                                c.name,
                              )}
                            </p>
                          ),
                          submit: t("Release protection"),
                        })
                      }
                    >
                      {t("Remove")}
                    </button>
                  </>
                }
              >
                <strong>{c.name}</strong>
                <p>
                  {c.chunks} {t(" chunks · X ")}
                  {c.min_x}〜{c.max_x} / Z {c.min_z}〜{c.max_z}
                </p>
              </Row>
            )}
          />
        </Card>
      </PageBlock>
      <div className="grid two">
        <PageBlock id="homes">
          <Card
            title={t("Home")}
            action={
              <button
                className="quiet"
                onClick={() =>
                  open({
                    title: t("Set a home here"),
                    type: "home_set",
                    fields: [{ name: "name", label: t("Home name"), max: 32 }],
                    note: (
                      <p>
                        {t(
                          "Save your current position in the official SMP. You start with three home slots.",
                        )}
                      </p>
                    ),
                    submit: t("Save this position"),
                  })
                }
              >
                {t("Add current position")}
              </button>
            }
          >
            <List
              values={rows(data, "homes")}
              empty={t(
                "Connect to the game to save places you want to return to.",
              )}
              render={(h) => (
                <Row
                  key={h.id}
                  actions={
                    <>
                      <button onClick={() => act("home_travel", { id: h.id })}>
                        {t("Travel")}
                      </button>
                      <button
                        className="quiet"
                        onClick={() =>
                          open({
                            title: t("Delete home"),
                            type: "home_delete",
                            values: { id: h.id },
                            note: <p>{t("Delete home “{0}”?", h.name)}</p>,
                            submit: t("Confirm deletion"),
                          })
                        }
                      >
                        {t("Delete")}
                      </button>
                    </>
                  }
                >
                  <strong>{h.name}</strong>
                </Row>
              )}
            />
          </Card>
        </PageBlock>
        <PageBlock id="meetup">
          <Card title={t("Meet up")}>
            <p>
              {t(
                "Travel to another player only after they accept your request.",
              )}
            </p>
            <button
              onClick={() =>
                open({
                  title: t("Request a teleport"),
                  type: "teleport_request",
                  fields: [playerField()],
                  note: (
                    <p>
                      {t(
                        "Both players must be in the official SMP. Teleports are unavailable for 30 seconds after PvP.",
                      )}
                    </p>
                  ),
                  submit: t("Send request"),
                })
              }
            >
              {t("Choose a player")}
            </button>
          </Card>
        </PageBlock>
      </div>
      <PageBlock id="achievements">
        <Card title={t("Achievements")}>
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
                  {a.earned_at ? t("· Earned") : ""}
                </small>
                <div className="rewards">
                  {a.land_chunks > 0 && (
                    <span>
                      {t("Land +")}
                      {a.land_chunks}
                    </span>
                  )}
                  {a.coins > 0 && (
                    <span>
                      {money(a.coins)} {t(" coins")}
                    </span>
                  )}
                </div>
              </div>
            ))}
          </div>
        </Card>
      </PageBlock>
      <PageBlock id="coin-history">
        <Card title={t("Coin history")}>
          <List
            values={rows(data, "ledger")}
            empty={t("No coin transactions yet.")}
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
                      transfer: t("Transfer"),
                      market: t("Market"),
                      npc: t("Material sale"),
                      achievement: t("Achievement reward"),
                      adventure: t("Adventure preparation"),
                    } as Data
                  )[l.kind] ?? l.kind}
                </strong>
                <small>{date(l.created_at)}</small>
              </Row>
            )}
          />
        </Card>
      </PageBlock>
    </>
  );
}

export function Market({ data }: { data: Data }) {
  const { me, open, act, send, route, jobs, showJob } = useApp();
  const [kind, setKind] = useState("all");
  const [owners, setOwners] = useState<Data[]>([]);
  const [claims, setClaims] = useState<Data[]>([]);
  const [preview, setPreview] = useState<Data | null>(
    () => placementPreviews.get(route.id ?? "") ?? null,
  );
  useEffect(() => {
    if (preview) placementPreviews.set(route.id ?? "", preview);
    else placementPreviews.delete(route.id ?? "");
  }, [preview, route.id]);
  useEffect(() => {
    api("/api/v1/servers/" + route.id + "?section=land")
      .then((v) => {
        setOwners(v.owners);
        setClaims(v.claims);
      })
      .catch((e) => console.error(e));
  }, [route.id]);
  const ownerField: Field = {
    name: "owner",
    label: t("Owner"),
    type: "select",
    options: owners.map((o) => ({ value: o.id, label: o.name })),
  };
  const claimField: Field = {
    name: "claim_id",
    label: t("Land"),
    type: "select",
    options: claims
      .filter((c) => c.state === "active")
      .map((c) => ({ value: c.id, label: c.name })),
  };
  const coordinateFields: Field[] = [
    { name: "x", label: t("Origin X"), type: "number", value: 0 },
    { name: "y", label: t("Origin Y"), type: "number", value: 64 },
    { name: "z", label: t("Origin Z"), type: "number", value: 0 },
    {
      name: "rotation",
      label: t("Rotation"),
      type: "select",
      options: [0, 90, 180, 270].map((n) => ({
        value: String(n),
        label: `${n}°`,
      })),
    },
  ];
  function capture() {
    open({
      title: t("Deposit an asset"),
      fields: [
        ownerField,
        {
          name: "kind",
          label: t("Type"),
          type: "select",
          options: [
            { value: "items", label: t("Item in your hand") },
            { value: "building", label: t("Pack a selected building") },
            { value: "land", label: t("Sell land with its buildings") },
          ],
        },
        { name: "title", label: t("Name"), max: 100 },
        { ...claimField, required: false },
        {
          name: "include_contents",
          label: t("Include container contents"),
          type: "checkbox",
        },
      ],
      note: (
        <p>
          {t(
            "Select the building in-game first. Packing removes the original structure, including animals, villagers, and decorations. The buyer can place it once. Empty containers first if their contents are not included.",
          )}
        </p>
      ),
      submit: t("Start deposit"),
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
      title: t("Preview building placement"),
      fields: [claimField, ...coordinateFields],
      note: (
        <p>
          {t(
            "Choose a location within your claim. Check for collisions with terrain and buildings before confirming.",
          )}
        </p>
      ),
      submit: t("Preview"),
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
    if (!preview || preview.result || preview.error) return;
    const job = jobs.find((j) => j.id === preview.job_id);
    if (job?.state === "succeeded")
      setPreview((p) => (p ? { ...p, result: job.result } : null));
    if (job && ["failed", "cancelled"].includes(job.state))
      setPreview((p) =>
        p
          ? { ...p, error: job.error ?? t("The preview did not complete.") }
          : null,
      );
  }, [jobs, preview?.job_id]);
  const mine = owners.map((o) => o.id);
  const listings = rows(data, "listings").filter(
    (l) => kind === "all" || l.kind === kind,
  );
  return (
    <>
      <PageBlock id="market">
        {" "}
        <div className="section-toolbar">
          <p>{t("Trade deposited assets. A 5% fee applies to sales.")}</p>
          <button className="primary" onClick={capture}>
            <Icon name="plus" />
            {t("Prepare a listing")}
          </button>
        </div>
        <div className="tabs" role="group" aria-label={t("Asset type")}>
          {[
            ["all", t("All")],
            ["items", t("Items")],
            ["building", t("Packed buildings")],
            ["land", t("Land with buildings")],
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
                      ? t("Items")
                      : l.kind === "building"
                        ? t("One-use building")
                        : t("Land with buildings")}
                  </span>
                </div>
                <h2>{l.title}</h2>
                <p>{l.seller_name}</p>
                <Manifest value={l.manifest} />
                <div className="price">
                  {money(l.price)} <small>{t(" coins")}</small>
                </div>
                {mine.includes(l.seller) ? (
                  <button
                    className="wide"
                    onClick={() =>
                      open({
                        title: t("Withdraw listing"),
                        type: "listing_cancel",
                        values: { id: l.id },
                        note: (
                          <p>
                            {t(
                              "There is no withdrawal fee. Your asset returns to storage.",
                            )}
                          </p>
                        ),
                        submit: t("Withdraw"),
                      })
                    }
                  >
                    {t("Withdraw listing")}
                  </button>
                ) : (
                  <button
                    className="primary wide"
                    onClick={() =>
                      open({
                        title: t("Buy “{0}”", l.title),
                        type: "listing_buy",
                        values: { id: l.id },
                        fields: [ownerField],
                        note: (
                          <>
                            <p>
                              {money(l.price)}
                              {t(
                                " coins will be paid in exchange for ownership.",
                              )}
                            </p>
                            <Manifest value={l.manifest} />
                            <p>
                              {l.kind === "building"
                                ? t(
                                    "Building materials are included. Place the building in your own claim after purchase.",
                                  )
                                : l.kind === "land"
                                  ? t(
                                      "Purchased land also uses your claim allowance.",
                                    )
                                  : t(
                                      "Collect the item in the official SMP after purchase.",
                                    )}
                            </p>
                          </>
                        ),
                        submit: t("Buy for {0} coins", money(l.price)),
                      })
                    }
                  >
                    {t("Buy")}
                  </button>
                )}
              </section>
            ))}
          </div>
        ) : (
          <Empty>
            {t(
              "No listings in this category. Deposit an asset to create a listing.",
            )}
          </Empty>
        )}
      </PageBlock>{" "}
      {preview && !preview.result && (
        <Card title={t("Placement preview")}>
          <p
            role={preview.error ? "alert" : "status"}
            className={preview.error ? "error" : ""}
          >
            {preview.error
              ? translateError(preview.error)
              : t("Checking the placement area…")}
          </p>
          <button
            onClick={() => showJob(preview.job_id, { kind: "asset.place" })}
          >
            {t("View details")}
          </button>
          <button onClick={() => setPreview(null)}>{t("Close")}</button>
        </Card>
      )}
      {preview?.result && (
        <Card title={t("Placement preview")}>
          <p>
            {preview.result.clear
              ? t("The placement area is clear.")
              : t("The placement area is blocked. Clear it and try again.")}
          </p>
          <Manifest value={preview.result} />
          <Actions>
            <button
              className="primary"
              disabled={!preview.result.clear}
              onClick={() =>
                open({
                  title: t("Place the building here"),
                  note: (
                    <p>
                      {t(
                        "Placing the building consumes the packed asset. Pack it again if you want to resell it.",
                      )}
                    </p>
                  ),
                  submit: t("Confirm placement"),
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
              {t("Place building")}
            </button>
            <button onClick={() => setPreview(null)}>{t("Close")}</button>
          </Actions>
        </Card>
      )}
      <PageBlock id="stored-assets">
        <Card title={t("Stored assets")}>
          <List
            values={rows(data, "assets").filter(
              (a) => !["placed", "delivered", "cancelled"].includes(a.state),
            )}
            empty={t(
              "No stored assets. Purchases and deposited assets will appear here.",
            )}
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
                              title: t("Set a price and list"),
                              type: "listing_create",
                              values: { asset: a.id },
                              fields: [
                                {
                                  name: "price",
                                  label: t("Price (coins)"),
                                  type: "number",
                                  min: 1,
                                  max: 1000000000000,
                                },
                              ],
                              submit: t("Create listing"),
                            })
                          }
                        >
                          {t("List for sale")}
                        </button>
                        {a.kind === "building" ? (
                          <button onClick={() => placement(a)}>
                            {t("Place")}
                          </button>
                        ) : a.kind === "items" ? (
                          <button
                            onClick={() => act("asset_receive", { id: a.id })}
                          >
                            {t("Collect in-game")}
                          </button>
                        ) : a.kind === "land" ? (
                          <button
                            onClick={() =>
                              open({
                                title: t("Release deposited land"),
                                type: "asset_withdraw",
                                values: { id: a.id },
                                note: (
                                  <p>
                                    {t(
                                      "Return the land and buildings to normal use. Review and deposit them again before relisting.",
                                    )}
                                  </p>
                                ),
                                submit: t("Release deposit"),
                              })
                            }
                          >
                            {t("Release deposit")}
                          </button>
                        ) : null}
                      </>
                    )}
                    {a.state === "capturing" &&
                      a.manifest?.required_consents?.includes(
                        me.account.id,
                      ) && (
                        <button
                          onClick={() =>
                            open({
                              title: t("Transfer a pet with this building"),
                              type: "asset_consent",
                              values: {
                                id: a.id,
                                manifest_sha256: a.manifest_sha256,
                              },
                              note: (
                                <>
                                  <p>
                                    {t(
                                      "I agree to transfer ownership of my pets inside this building to its new owner.",
                                    )}
                                  </p>
                                  <Manifest value={a.manifest} />
                                </>
                              ),
                              submit: t("I agree"),
                            })
                          }
                        >
                          {t("Confirm as pet owner")}
                        </button>
                      )}
                    {a.state === "capturing" &&
                      mine.includes(a.owner) &&
                      a.manifest_sha256 && (
                        <button
                          className="quiet"
                          onClick={() => act("asset_withdraw", { id: a.id })}
                        >
                          {t("Cancel packing request")}
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
      </PageBlock>
      <PageBlock id="materials">
        <Card title={t("Sell materials")}>
          <div className="section-toolbar">
            <div>
              <strong>
                {t("Remaining today")}
                {money(data.npc_remaining)} {t(" coins")}
              </strong>
              <small>
                {t("Fixed prices · 2,000 coins replenished daily at 00:00 UTC")}
              </small>
            </div>
            <button
              onClick={() =>
                open({
                  title: t("Sell materials now"),
                  type: "npc_sell",
                  fields: [
                    {
                      name: "material",
                      label: t("Material"),
                      type: "select",
                      options: rows(data, "prices").map((p) => ({
                        value: p.material,
                        label: t("{0} · {1} coins each", p.material, p.price),
                      })),
                    },
                    {
                      name: "amount",
                      label: t("Quantity"),
                      type: "number",
                      min: 1,
                      max: 2304,
                      value: 1,
                    },
                  ],
                  note: (
                    <p>
                      {t(
                        "Connect to the official SMP to sell materials from your inventory. Coins are credited after the materials are collected.",
                      )}
                    </p>
                  ),
                  submit: t("Sell materials now"),
                })
              }
            >
              {t("Sell materials now")}
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
      </PageBlock>
    </>
  );
}
function Manifest({ value }: { value: Data }) {
  const m = value?.summary ?? value ?? {};
  return (
    <div className="manifest">
      {m.dimensions && (
        <p>
          {t("Dimensions")}{" "}
          {Array.isArray(m.dimensions)
            ? m.dimensions.join(" × ")
            : String(m.dimensions)}
        </p>
      )}
      {(m.block_count ?? m.blocks) !== undefined && (
        <p>
          {money(m.block_count ?? m.blocks)} {t(" blocks")}
        </p>
      )}
      {m.material && m.amount !== undefined && (
        <p>
          {m.material} × {m.amount}
        </p>
      )}
      {m.origin && (
        <p>
          {t("Origin: ")}
          {m.origin.join(", ")} {t("· Rotation ")}
          {m.rotation}°
        </p>
      )}
      {m.footprint && (
        <p>
          {t("Placement area: X ")}
          {m.footprint.min_x}〜{m.footprint.max_x} / Y {m.footprint.min_y}〜
          {m.footprint.max_y} / Z {m.footprint.min_z}〜{m.footprint.max_z}
        </p>
      )}
      {m.containers?.length > 0 && (
        <details>
          <summary>
            {t("Container contents (")}
            {m.containers.length} {t(" stacks)")}
          </summary>
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
              {e.trades ? t(" · Villager trades included") : ""}
            </li>
          ))}
        </ul>
      )}
      {m.contents_included !== undefined && (
        <small>
          {t("Container contents: ")}
          {m.contents_included ? t("Included") : t("Not included")}
        </small>
      )}
    </div>
  );
}

export function Adventure({ data }: { data: Data }) {
  const { open, act } = useApp();
  return (
    <>
      <section className="adventure-hero">
        <span className="eyebrow">{t("Requirements")}</span>
        <h2>{t("Create a private End")}</h2>
        <p>
          {t(
            "Create an End world for yourself or your party. Your inventory is shared with the official SMP, so you can bring items back.",
          )}
        </p>
        <div className="adventure-cost">
          <div>
            <strong>1,000</strong>
            <small>{t(" coins")}</small>
          </div>
          <span>＋</span>
          <div>
            <strong>12</strong>
            <small>{t("Eyes of Ender")}</small>
          </div>
          <div>
            <strong>{t("3 hours")}</strong>
            <small>{t("From activation")}</small>
          </div>
        </div>
        <button
          className="primary"
          onClick={() =>
            open({
              title: t("Prepare a private End"),
              type: "adventure_create",
              note: (
                <>
                  <p>
                    {t(
                      "You need 1,000 coins and 12 Eyes of Ender. Every party member must be in the official SMP and marked ready. The participant list is fixed when preparation starts.",
                    )}
                  </p>
                  <p>
                    {t(
                      "The world closes three hours after it opens. Collect dropped items before then. Cancellation before opening or a failed start refunds coins and materials. Collect reserved items from your stored assets in-game.",
                    )}
                  </p>
                </>
              ),
              submit: t("Reserve coins and materials"),
            })
          }
        >
          {t("Prepare adventure")}
          <Icon name="arrow" />
        </button>
      </section>
      <p className="intro">
        {t("The permanent Nether and End do not have a preparation fee.")}
      </p>
      <Card title={t("Your adventures")}>
        <List
          values={rows(data, "adventures")}
          empty={t(
            "No adventures in progress. Prepare one for yourself or your party.",
          )}
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
                      {t("Enter adventure")}
                    </button>
                  )}
                  {a.can_cancel && (
                    <button
                      onClick={() =>
                        open({
                          title: t("Cancel preparation"),
                          type: "adventure_cancel",
                          values: { id: a.id },
                          note: (
                            <p>
                              {t(
                                "The world closes and reserved coins are released. Collect reserved Eyes of Ender from stored assets. Check the result until the refund is complete.",
                              )}
                            </p>
                          ),
                          submit: t("Cancel and refund"),
                        })
                      }
                    >
                      {t("Cancel")}
                    </button>
                  )}
                  {a.can_receive && (
                    <button
                      onClick={() =>
                        act("asset_receive", { id: a.material_asset })
                      }
                    >
                      {t("Collect refunded items")}
                    </button>
                  )}
                </>
              }
            >
              <strong>{t("Private End")}</strong>
              <small>
                {t("Preparation started")}
                {date(a.created_at)}
              </small>
              {a.expires_at && (
                <p>
                  {t("Closes at")}
                  {date(a.expires_at)}
                </p>
              )}
            </Row>
          )}
        />
      </Card>
    </>
  );
}

export function Settings({ data }: { data: Data }) {
  const { me, send, act, open } = useApp();
  const [code, setCode] = useState("");
  const policies = [
    { value: "friends", label: t("Friends only") },
    { value: "everyone", label: t("Everyone") },
    { value: "none", label: t("Nobody") },
  ];
  return (
    <>
      <div className="grid two">
        <PageBlock id="profile">
          <Card title={t("Profile")}>
            <LanguagePicker
              save={(language) => send("language", { language })}
            />
            <p>{t("Your language is shared with linked game accounts.")}</p>
            <label className="field">
              {t("Account ID")}
              <input readOnly value={me.account.id} />
            </label>
            <ActionForm
              fields={[
                {
                  name: "display_name",
                  label: t("Display name"),
                  value: me.account.name,
                  max: 64,
                },
              ]}
              onSubmit={(v) =>
                send("privacy", {
                  ...v,
                  dm_policy: me.account.dm_policy,
                  activity_policy: me.account.activity_policy,
                })
              }
            />
          </Card>
        </PageBlock>
        <PageBlock id="privacy">
          <Card title={t("Privacy")}>
            <ActionForm
              fields={[
                {
                  name: "dm_policy",
                  label: t("Who can send you DMs"),
                  type: "select",
                  value: me.account.dm_policy,
                  options: policies,
                },
                {
                  name: "activity_policy",
                  label: t("Who can see your activity"),
                  type: "select",
                  value: me.account.activity_policy,
                  options: policies,
                },
              ]}
              onSubmit={(v) =>
                send("privacy", { ...v, display_name: me.account.name })
              }
            />
          </Card>
        </PageBlock>
        <PageBlock id="linking">
          <Card title={t("Link game accounts")}>
            <p>
              {t(
                "Combine Web, Java, and Bedrock identities into one account. If you have played with both accounts, choose one set of game data to keep using.",
              )}
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
                  title: t("Create a link code"),
                  note: (
                    <p>
                      {t(
                        "Enter this code on your other account. It expires in ten minutes.",
                      )}
                    </p>
                  ),
                  submit: t("Create code"),
                  action: async () => {
                    const result = await send("link_begin");
                    setCode(result.code);
                  },
                })
              }
            >
              {t("Create link code")}
            </button>
            {code && (
              <p className="link-code">
                <code>{code}</code>
                <small>{t("Enter on your other account")}</small>
              </p>
            )}
            <ActionForm
              fields={[
                { name: "code", label: t("Link code from your other account") },
              ]}
              submit={t("Link to this account")}
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
                        {t(
                          "Choose the game data to keep using. The other data is archived; coins and items are not combined.",
                        )}
                      </p>
                      {l.profiles?.map((p: Data) => (
                        <button
                          key={p.id}
                          onClick={() =>
                            open({
                              title: t("Confirm game data"),
                              type: "link_confirm",
                              values: { id: l.id, selected_profile: p.id },
                              note: (
                                <p>
                                  {t(
                                    "Use the game data for “{0}”? The other data is archived. Finish listings and adventures first. Both game connections will be disconnected during linking.",
                                    p.name,
                                  )}
                                  {!p.native_uuid &&
                                    t(
                                      "This profile has no game inventory or achievements, so it starts fresh.",
                                    )}
                                </p>
                              ),
                              submit: t("Link using this data"),
                            })
                          }
                        >
                          {p.name} · {money(p.wallet.balance)}
                          {t(" coins")}
                        </button>
                      ))}
                    </>
                  )}
              </div>
            ))}
          </Card>
        </PageBlock>
      </div>
      <PageBlock id="blocks">
        <Card title={t("Blocked players")}>
          <List
            values={rows(data, "blocks")}
            empty={t("No blocked players.")}
            render={(b) => (
              <Row
                key={b.id}
                actions={
                  <button
                    onClick={() =>
                      act("block", { target: b.id, blocked: false })
                    }
                  >
                    {t("Remove")}
                  </button>
                }
              >
                <strong>{b.name}</strong>
              </Row>
            )}
          />
        </Card>
      </PageBlock>
      <PageBlock id="reports">
        <Card title={t("Your reports")}>
          <p>
            {t(
              "Administrators receive only the messages you select and submit.",
            )}
          </p>
          <List
            values={rows(data, "reports")}
            empty={t(
              "No reports submitted. Select messages in chat to submit a report.",
            )}
            render={(r) => (
              <Row key={r.id} actions={<span>{r.status}</span>}>
                <strong>{r.reason}</strong>
                <small>{date(r.created_at)}</small>
                {r.resolution && <p>{r.resolution}</p>}
              </Row>
            )}
          />
        </Card>
      </PageBlock>
      <button
        className="quiet danger"
        onClick={() =>
          open({
            title: t("Sign out of this account"),
            submit: t("Sign out"),
            action: async () => {
              await api("/auth/logout", { method: "POST", body: "{}" });
              location.assign("/");
            },
          })
        }
      >
        {t("Sign out on this device")}
      </button>
    </>
  );
}

export function Admin({ data }: { data: Data }) {
  const { me, open, act, showJob } = useApp();
  if (!me.account.administrator)
    return <Empty>{t("Administrator access is required.")}</Empty>;
  return (
    <>
      <PageBlock id="reports">
        <Card title={t("Reports")}>
          <List
            values={rows(data, "reports")}
            empty={t("No reports waiting for review.")}
            render={(r) => (
              <Row
                key={r.id}
                actions={
                  <button
                    onClick={() =>
                      open({
                        title: t("Review submitted evidence"),
                        note: (
                          <p>
                            {t(
                              "Your access and its time are recorded in the audit log. Only submitted evidence is shown.",
                            )}
                          </p>
                        ),
                        submit: t("Review evidence"),
                        action: async () => {
                          const report = await api(`/api/v1/reports/${r.id}`);
                          queueMicrotask(() =>
                            open({
                              title: t("Report evidence"),
                              type: "report_resolve",
                              values: { id: r.id },
                              fields: [
                                {
                                  name: "status",
                                  label: t("Resolution"),
                                  type: "select",
                                  options: [
                                    {
                                      value: "investigating",
                                      label: t("Investigating"),
                                    },
                                    { value: "resolved", label: t("Resolved") },
                                    {
                                      value: "dismissed",
                                      label: t("No action needed"),
                                    },
                                  ],
                                },
                                {
                                  name: "resolution",
                                  label: t("Resolution notes"),
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
                    {t("Open evidence")}
                  </button>
                }
              >
                <strong>
                  {t("Report · ")}
                  {date(r.created_at)}
                </strong>
                <small>{r.status}</small>
              </Row>
            )}
          />
        </Card>
      </PageBlock>
      <PageBlock id="ranks">
        <Card title={t("Hosting access tiers")}>
          <p>
            {t(
              "Administrators approve hosting limits separately from play time and achievements. A downgrade does not delete saved data.",
            )}
          </p>
          <Actions>
            <button
              onClick={() =>
                open({
                  title: t("Assign a tier"),
                  type: "rank_set",
                  fields: [
                    playerField(),
                    {
                      name: "rank",
                      label: t("Tier number"),
                      type: "number",
                      min: 0,
                      max: 32767,
                    },
                  ],
                })
              }
            >
              {t("Assign to player")}
            </button>
            <button
              onClick={() =>
                open({
                  title: t("Configure tier limits"),
                  type: "rank_configure",
                  fields: [
                    {
                      name: "id",
                      label: t("Tier number"),
                      type: "number",
                      min: 0,
                      max: 32767,
                    },
                    nameField(),
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
                          t("Server count"),
                          t("Concurrent servers"),
                          t("Active memory (MiB)"),
                          t("CPU (1 core = 1000)"),
                          t("Storage (MiB)"),
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
              {t("Save tier")}
            </button>
            <button
              onClick={() =>
                open({
                  title: t("Set access restriction"),
                  type: "ban",
                  fields: [
                    playerField(),
                    {
                      name: "hours",
                      label: t("Hours (0 to remove)"),
                      type: "number",
                      min: 0,
                      max: 876000,
                      value: 24,
                    },
                    { name: "reason", label: t("Reason"), type: "textarea" },
                  ],
                  submit: t("Apply restriction"),
                })
              }
            >
              {t("Access restrictions")}
            </button>
          </Actions>
          <List
            values={rows(data, "ranks")}
            empty={t("No tiers configured.")}
            render={(r) => (
              <Row key={r.id}>
                <strong>
                  {r.id} · {r.name}
                </strong>
                <p>
                  {t("Created ")}
                  {r.server_count} {t("/ Concurrent ")}
                  {r.concurrent_servers} · {money(r.memory_mib)} MiB ·{" "}
                  {r.cpu_millis / 1000} {t(" cores · Storage ")}{" "}
                  {money(r.storage_mib)} MiB
                </p>
              </Row>
            )}
          />
        </Card>
      </PageBlock>
      <PageBlock id="backups">
        <Card
          title={t("Official backups")}
          action={
            <button onClick={() => act("official_backup")}>
              {t("Back up all official data")}
            </button>
          }
        >
          <p>
            {t(
              "Save worlds, inventories, claims, ledgers, and stored assets together.",
            )}
          </p>
          <p>
            {data.backup_policy?.enabled
              ? t(
                  "Daily at {0}:00 UTC. Keeps seven daily and four weekly successful backups.",
                  String(data.backup_policy.hour_utc).padStart(2, "0"),
                )
              : t("Automatic backups are currently disabled.")}
            {t(
              "Manual and pinned backups are excluded from automatic pruning.",
            )}
          </p>
          <p>
            {t("Last completed backup: ")}
            {data.backup_policy?.last_completed_at
              ? date(data.backup_policy.last_completed_at)
              : t("No record")}
          </p>
          <List
            values={rows(data, "backups")}
            empty={t("No backups recorded.")}
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
                          {b.pinned ? t("Unpin backup") : t("Pin backup")}
                        </button>
                      )}
                  </>
                }
              >
                <strong>
                  {b.kind === "official"
                    ? t("All official data")
                    : t("Personal server")}
                </strong>
                <small>{date(b.created_at)}</small>
                {b.kind === "official" && (
                  <p>
                    {b.scheduled_for
                      ? t("Daily automatic backup")
                      : t("Manual backup")}
                    {b.pinned ? t(" · Pinned") : ""}
                    {b.completed_at
                      ? t(" · Completed {0}", date(b.completed_at))
                      : ""}
                  </p>
                )}
                {b.error && <p className="error">{b.error}</p>}
              </Row>
            )}
          />
        </Card>
      </PageBlock>
      <PageBlock id="jobs">
        <Card title={t("Actions needing attention")}>
          <List
            values={rows(data, "jobs").filter(
              (j) =>
                !["server.logs", "server.files", "server.file.read"].includes(
                  j.kind,
                ),
            )}
            empty={t("No actions need attention.")}
            render={(j) => (
              <Row
                key={j.id}
                actions={
                  <>
                    <Status value={j.state} />
                    <button onClick={() => showJob(j.id, j)}>
                      {t("View details")}
                    </button>
                  </>
                }
              >
                <strong>
                  {jobTitle(j)}
                  {j.server_name ? " · " + j.server_name : ""}
                </strong>
                <p>{j.error ?? j.progress?.message}</p>
                <small>{date(j.updated_at)}</small>
              </Row>
            )}
          />
        </Card>
      </PageBlock>
      <PageBlock id="audit">
        <Card title={t("Audit log")}>
          <List
            values={rows(data, "audit")}
            empty={t("No administrative actions recorded.")}
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
      </PageBlock>
    </>
  );
}
