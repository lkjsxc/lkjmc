import { PrivateCache, onResourceReset } from "./identity";
import { t, message, translateError, renderSystemMessage } from "./i18n";
import { useEffect, useState, type ReactNode } from "react";
import { api, date, money, type Data } from "./api";
import { useApp, LanguagePicker, PageBlock } from "./App";
import { NotificationItem } from "./jobs";
import { Achievements } from "./achievements";
import { AdminOperations } from "./adminOperations";
import { ActionForm, Card, Empty, Icon, Status, type Field } from "./ui";
export { Social } from "./social";
const rows = (data: Data, key: string): Data[] => data[key] ?? [];
type OwnerPermission = "can_build" | "can_sell" | "can_spend";
const ownerCan = (
  owner: Data | undefined,
  account: string,
  permission: OwnerPermission,
) =>
  !!owner && (owner.id === account || owner.permissions?.[permission] === true);
const placementPreviews = new PrivateCache<Data>(12);
onResourceReset((id) => placementPreviews.delete(id));
const nameField = (): Field => ({
  name: "name",
  label: message("text.name"),
  max: 64,
});
const playerField = (): Field => ({
  name: "target",
  label: message("text.player"),
  type: "player",
});
const visibilities = () => [
  { value: "private", label: message("text.you_and_administrators") },
  { value: "invite", label: message("text.invited_players") },
  { value: "public", label: message("text.public") },
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

export function Inbox({ data }: { data: Data }) {
  const { me, go, act, route } = useApp();
  const kinds: Record<string, string> = {
    room: t("text.group_chat"),
    team: t("text.team"),
    party: t("text.party"),
    community: t("text.community"),
    server: t("text.server"),
    teleport: t("text.teleport"),
  };
  const notices: Record<string, string> = {
    invitation: t("text.new_invitation"),
    invitation_response: t("text.invitation_response"),
    friend_request: t("text.friend_request"),
    friend_response: t("text.friend_request_response"),
    message: t("text.new_message"),
    transfer: t("text.coins_received"),
    market_sale: t("text.listing_sold"),
    achievement: t("text.achievement_unlocked"),
    job_finished: t("text.action_completed"),
    link_candidate: t("text.account_linking_confirmation"),
  };
  return (
    <>
      <div className="grid two">
        <PageBlock id="invitations">
          <Card title={t("text.invitations")}>
            <List
              values={rows(data, "invitations")}
              empty={t("text.no_invitations_to_respond_to")}
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
                        {t("text.accept")}
                      </button>
                      <button
                        className="quiet"
                        onClick={() =>
                          act("invite_respond", { id: i.id, accept: false })
                        }
                      >
                        {t("text.decline")}
                      </button>
                    </>
                  }
                >
                  <strong>{i.sender_name}</strong>
                  <p>
                    {i.kind === "teleport" ? (
                      t(
                        i.teleport_here
                          ? "teleport.here_note"
                          : "teleport.to_note",
                        i.sender_name,
                        date(i.expires_at),
                      )
                    ) : (
                      <>
                        {kinds[i.kind] ?? i.kind}
                        {t("text.invitation")}
                      </>
                    )}
                  </p>
                  <small>{date(i.created_at)}</small>
                </Row>
              )}
            />
            {route.component === "home" && (
              <a className="feed-more" href="#/play/invitations">
                {t("text.view_all")}{" "}
                {data.counts?.["invitations"] != null && (
                  <span>({data.counts["invitations"]})</span>
                )}
              </a>
            )}
          </Card>
        </PageBlock>
        <PageBlock id="notifications">
          <Card
            title={t("text.notifications")}
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
                  {t("text.mark_all_as_read")}
                </button>
              ) : undefined
            }
          >
            <List
              values={rows(data, "notifications")}
              empty={t("text.no_notifications_yet")}
              render={(n) => <NotificationItem key={n.id} notice={n} />}
            />
            {route.component === "home" && (
              <a className="feed-more" href="#/play/notifications">
                {t("text.view_all")}{" "}
                {data.counts?.["notifications"] != null && (
                  <span>({data.counts["notifications"]})</span>
                )}
              </a>
            )}
          </Card>
        </PageBlock>
      </div>
    </>
  );
}

export function Life({ data }: { data: Data }) {
  const { me, open, act } = useApp();
  const owners = rows(data, "owners");
  const buildOwners = owners.filter((owner) =>
    ownerCan(owner, me.account.id, "can_build"),
  );
  const ownerField: Field = {
    name: "owner",
    label: message("text.owner"),
    type: "select",
    options: buildOwners.map((o) => ({ value: o.id, label: o.name })),
    value:
      buildOwners.find((owner) => owner.id === me.account.id)?.id ??
      buildOwners[0]?.id,
  };
  return (
    <>
      <PageBlock id="coins">
        <div className="grid two">
          {owners.map((o) => (
            <section className="balance-card" key={o.id}>
              <p>
                {o.kind === "team"
                  ? t("text.team_assets")
                  : t("text.personal_assets")}{" "}
                · {o.name}
              </p>
              <strong>
                {money(o.wallet.balance - o.wallet.reserved)}{" "}
                <span>{t("text.coins")}</span>
              </strong>
              {o.wallet.reserved > 0 && (
                <small>
                  {t("text.reserved")}
                  {money(o.wallet.reserved)} {t("text.coins")}
                </small>
              )}
              <div className="land-meter">
                <span>{t("text.protected_land")}</span>
                <strong>
                  {o.used_chunks} / {o.land.chunks} {t("text.chunks")}
                </strong>
              </div>
              <progress
                value={o.used_chunks}
                max={Math.max(1, o.land.chunks)}
                aria-label={t("text.protected_land") + ": " + o.name}
              />
              {ownerCan(o, me.account.id, "can_spend") && (
                <button
                  className="quiet"
                  onClick={() =>
                    open({
                      title: message("economy.send_coins_title", o.name),
                      type: "wallet_transfer",
                      values: { owner: o.id },
                      ownerScope: { id: o.id, permission: "can_spend" },
                      fields: [
                        playerField(),
                        {
                          name: "amount",
                          label: message("text.amount"),
                          type: "number",
                          min: 1,
                          max: 1000000000000,
                        },
                      ],
                      submit: message("text.send_coins_now"),
                    })
                  }
                >
                  {t("text.send_coins_now")}
                </button>
              )}
            </section>
          ))}
        </div>
      </PageBlock>
      <PageBlock id="land">
        <Card
          title={t("text.protected_land")}
          action={
            <button
              className="primary small"
              disabled={!buildOwners.length}
              onClick={() =>
                open({
                  title: message("text.protect_land"),
                  type: "claim_create",
                  ownerScope: {
                    ids: buildOwners.map((owner) => owner.id),
                    permission: "can_build",
                  },
                  fields: [
                    ownerField,
                    nameField(),
                    ...["min_x", "min_z", "max_x", "max_z"].map(
                      (name, i): Field => ({
                        name,
                        label: [
                          message("text.west_chunk_x"),
                          message("text.north_chunk_z"),
                          message("text.east_chunk_x"),
                          message("text.south_chunk_z"),
                        ][i],
                        type: "number",
                        value: 0,
                        min: -1800000,
                        max: 1800000,
                      }),
                    ),
                  ],
                  note: () => (
                    <p>
                      {t(
                        "text.protect_land_in_the_survival_world_in_16_16_block_chunk_2b15bd96f9",
                      )}
                    </p>
                  ),
                  submit: message("text.request_protection"),
                })
              }
            >
              <Icon name="plus" />
              {t("text.protect_land_now")}
            </button>
          }
        >
          <List
            values={rows(data, "claims")}
            empty={t(
              "text.no_protected_land_yet_choose_protect_land_to_make_your_341371b95a",
            )}
            render={(c) => (
              <Row
                key={c.id}
                actions={
                  <>
                    <Status value={c.state} />
                    {ownerCan(
                      owners.find((owner) => owner.id === c.owner),
                      me.account.id,
                      "can_sell",
                    ) && (
                      <button
                        className="quiet danger"
                        disabled={c.state !== "active"}
                        onClick={() =>
                          open({
                            title: message("text.release_land_protection"),
                            type: "claim_release",
                            values: { id: c.id },
                            ownerScope: { id: c.owner, permission: "can_sell" },
                            note: () => (
                              <>
                                <p>
                                  {t(
                                    "economy.owner_note",
                                    owners.find((owner) => owner.id === c.owner)
                                      ?.name ?? "",
                                  )}
                                </p>
                                <p>
                                  {t(
                                    "text.release_protection_for_0_buildings_remain_and_other_pla_7a504da2b7",
                                    c.name,
                                  )}
                                </p>
                              </>
                            ),
                            submit: message("text.release_protection"),
                          })
                        }
                      >
                        {t("text.remove")}
                      </button>
                    )}
                  </>
                }
              >
                <strong>{c.name}</strong>
                <p>
                  {c.chunks} {t("text.chunks_x")}
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
            title={t("text.home")}
            action={
              <button
                className="quiet"
                onClick={() =>
                  open({
                    title: message("text.set_a_home_here"),
                    type: "home_set",
                    fields: [
                      {
                        name: "name",
                        label: message("text.home_name"),
                        max: 32,
                      },
                    ],
                    note: () => (
                      <p>
                        {t(
                          "text.save_your_current_position_in_the_official_smp_you_star_d8b43196f3",
                        )}
                      </p>
                    ),
                    submit: message("text.save_this_position"),
                  })
                }
              >
                {t("text.add_current_position")}
              </button>
            }
          >
            <List
              values={rows(data, "homes")}
              empty={t(
                "text.connect_to_the_game_to_save_places_you_want_to_return_to",
              )}
              render={(h) => (
                <Row
                  key={h.id}
                  actions={
                    <>
                      <button onClick={() => act("home_travel", { id: h.id })}>
                        {t("text.travel")}
                      </button>
                      <button
                        className="quiet"
                        onClick={() =>
                          open({
                            title: message("text.delete_home"),
                            type: "home_delete",
                            values: { id: h.id },
                            note: () => (
                              <p>{t("text.delete_home_0", h.name)}</p>
                            ),
                            submit: message("text.confirm_deletion"),
                          })
                        }
                      >
                        {t("text.delete")}
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
          <div className="actions">
            {[false, true].map((here) => (
              <button
                key={String(here)}
                onClick={() =>
                  open({
                    title: message(here ? "teleport.here" : "teleport.to"),
                    type: "teleport_request",
                    values: { here },
                    fields: [playerField()],
                    submit: message("text.send_request"),
                  })
                }
              >
                {t(here ? "teleport.here" : "teleport.to")}
              </button>
            ))}
            <a className="button quiet" href="#/play/invitations">
              {t("teleport.requests")}
            </a>
          </div>
        </PageBlock>
      </div>
      <PageBlock id="achievements">
        <Achievements groups={rows(data, "achievements")} />
      </PageBlock>
      <PageBlock id="coin-history">
        <Card title={t("text.coin_history")}>
          <List
            values={rows(data, "ledger")}
            empty={t("text.no_coin_transactions_yet")}
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
                      transfer: t("text.transfer"),
                      market: t("text.market"),
                      npc: t("text.material_sale"),
                      achievement: t("text.achievement_reward"),
                      adventure: t("text.adventure_preparation"),
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
  const owners = rows(data, "owners");
  const claims = rows(data, "claims");
  const sellOwners = owners.filter((owner) =>
    ownerCan(owner, me.account.id, "can_sell"),
  );
  const permitted = (ownerId: string, permission: OwnerPermission) =>
    ownerCan(
      owners.find((owner) => owner.id === ownerId),
      me.account.id,
      permission,
    );
  const ownerName = (ownerId: string) =>
    owners.find((owner) => owner.id === ownerId)?.name ?? "";
  const [preview, setPreview] = useState<Data | null>(
    () => placementPreviews.get(route.id ?? "") ?? null,
  );
  useEffect(() => {
    if (preview) placementPreviews.set(route.id ?? "", preview);
    else placementPreviews.delete(route.id ?? "");
  }, [preview, route.id]);
  const ownerField = (eligible: Data[]): Field => ({
    name: "owner",
    label: message("text.owner"),
    type: "select",
    options: eligible.map((owner) => ({ value: owner.id, label: owner.name })),
    value:
      eligible.find((owner) => owner.id === me.account.id)?.id ??
      eligible[0]?.id,
  });
  const claimField = (permission: OwnerPermission): Field => ({
    name: "claim_id",
    label: message("text.land_b6baff93"),
    type: "select",
    options: claims
      .filter(
        (claim) =>
          claim.state === "active" && permitted(claim.owner, permission),
      )
      .map((claim) => ({
        value: claim.id,
        label: ownerName(claim.owner) + " · " + claim.name,
      })),
  });
  const coordinateFields: Field[] = [
    { name: "x", label: message("text.origin_x"), type: "number", value: 0 },
    { name: "y", label: message("text.origin_y"), type: "number", value: 64 },
    { name: "z", label: message("text.origin_z"), type: "number", value: 0 },
    {
      name: "rotation",
      label: message("text.rotation"),
      type: "select",
      options: [0, 90, 180, 270].map((n) => ({
        value: String(n),
        label: `${n}°`,
      })),
    },
  ];
  function capture() {
    open({
      title: message("text.deposit_an_asset"),
      ownerScope: {
        ids: sellOwners.map((owner) => owner.id),
        permission: "can_sell",
      },
      fields: [
        ownerField(sellOwners),
        {
          name: "kind",
          label: message("text.type"),
          type: "select",
          options: [
            { value: "items", label: message("text.item_in_your_hand") },
            {
              value: "building",
              label: message("text.pack_a_selected_building"),
            },
            {
              value: "land",
              label: message("text.sell_land_with_its_buildings"),
            },
          ],
        },
        { name: "title", label: message("text.name"), max: 100 },
        { ...claimField("can_sell"), required: false },
        {
          name: "include_contents",
          label: message("text.include_container_contents"),
          type: "checkbox",
        },
      ],
      note: () => (
        <p>
          {t(
            "text.select_the_building_in_game_first_packing_removes_the_o_3f323faebf",
          )}
        </p>
      ),
      submit: message("text.start_deposit"),
      action: async (v) => {
        const { claim_id, ...other } = v;
        if (
          v.kind !== "items" &&
          claims.find((claim) => claim.id === claim_id)?.owner !== v.owner
        )
          throw message("economy.claim_owner_mismatch");
        await send("asset_capture", {
          ...other,
          selection: { claim_id: v.kind === "items" ? null : claim_id || null },
        });
      },
    });
  }
  function placement(asset: Data) {
    open({
      title: message("text.preview_building_placement"),
      ownerScope: {
        ids: [
          ...new Set([
            asset.owner,
            ...claims
              .filter(
                (claim) =>
                  claim.state === "active" &&
                  permitted(claim.owner, "can_build"),
              )
              .map((claim) => claim.owner),
          ]),
        ],
        permission: "can_build",
      },
      fields: [claimField("can_build"), ...coordinateFields],
      note: () => (
        <>
          <p>{t("economy.owner_note", ownerName(asset.owner))}</p>
          <p>
            {t(
              "text.choose_a_location_within_your_claim_check_for_collision_256a512a23",
            )}
          </p>
        </>
      ),
      submit: message("text.preview"),
      action: async (v) => {
        const result = await send("asset_place", {
          id: asset.id,
          placement: { ...v, rotation: Number(v.rotation), preview: true },
        });
        setPreview({
          asset_id: asset.id,
          asset_owner: asset.owner,
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
    if (job && ["failed", "cancelled", "delivery_unknown"].includes(job.state))
      setPreview((p) =>
        p
          ? {
              ...p,
              error:
                job.error ??
                message(
                  job.state === "delivery_unknown"
                    ? "economy.preview_delivery_unknown"
                    : "text.the_preview_did_not_complete",
                ),
            }
          : null,
      );
  }, [jobs, preview?.job_id]);
  const previewAllowed =
    !!preview &&
    permitted(preview.asset_owner, "can_build") &&
    claims.some(
      (claim) =>
        claim.id === preview.placement?.claim_id &&
        claim.state === "active" &&
        permitted(claim.owner, "can_build"),
    );
  useEffect(() => {
    if (preview && !previewAllowed) setPreview(null);
  }, [preview, previewAllowed]);
  const listings = rows(data, "listings").filter(
    (l) => kind === "all" || l.kind === kind,
  );
  const buyers = (listing: Data) =>
    owners.filter(
      (owner) =>
        owner.id !== listing.seller &&
        ownerCan(owner, me.account.id, "can_spend") &&
        (listing.kind !== "land" ||
          ownerCan(owner, me.account.id, "can_build")),
    );
  return (
    <>
      <PageBlock id="market">
        {" "}
        <div className="section-toolbar">
          <p>{t("text.trade_deposited_assets_a_5_fee_applies_to_sales")}</p>
          <button
            className="primary"
            onClick={capture}
            disabled={!sellOwners.length}
          >
            <Icon name="plus" />
            {t("text.prepare_a_listing")}
          </button>
        </div>
        <div className="tabs" role="group" aria-label={t("text.asset_type")}>
          {[
            ["all", t("text.all")],
            ["items", t("text.items")],
            ["building", t("text.packed_buildings")],
            ["land", t("text.land_with_buildings")],
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
                      ? t("text.items")
                      : l.kind === "building"
                        ? t("text.one_use_building")
                        : t("text.land_with_buildings")}
                  </span>
                </div>
                <h2>
                  {l.title_message
                    ? renderSystemMessage(l.title_message)
                    : l.title}
                </h2>
                <p>{l.seller_name}</p>
                <Manifest value={l.manifest} />
                <div className="price">
                  {money(l.price)} <small>{t("text.coins")}</small>
                </div>
                {permitted(l.seller, "can_sell") && (
                  <button
                    className="wide"
                    onClick={() =>
                      open({
                        title: message("text.withdraw_listing"),
                        type: "listing_cancel",
                        values: { id: l.id },
                        ownerScope: { id: l.seller, permission: "can_sell" },
                        note: () => (
                          <>
                            <p>
                              {t("economy.owner_note", ownerName(l.seller))}
                            </p>
                            <p>
                              {t(
                                "text.there_is_no_withdrawal_fee_your_asset_returns_to_storage",
                              )}
                            </p>
                          </>
                        ),
                        submit: message("text.withdraw"),
                      })
                    }
                  >
                    {t("text.withdraw_listing")}
                  </button>
                )}
                {!!buyers(l).length && (
                  <button
                    className="primary wide"
                    onClick={() =>
                      open({
                        title: message("text.buy_0", l.title),
                        type: "listing_buy",
                        values: { id: l.id },
                        ownerScope: {
                          ids: buyers(l).map((owner) => owner.id),
                          permission: "can_spend",
                          ...(l.kind === "land"
                            ? { additionalPermission: "can_build" as const }
                            : {}),
                        },
                        fields: [ownerField(buyers(l))],
                        note: () => (
                          <>
                            <p>
                              {money(l.price)}
                              {t(
                                "text.coins_will_be_paid_in_exchange_for_ownership",
                              )}
                            </p>
                            <Manifest value={l.manifest} />
                            <p>
                              {l.kind === "building"
                                ? t(
                                    "text.building_materials_are_included_place_the_building_in_y_a4acf9b693",
                                  )
                                : l.kind === "land"
                                  ? t(
                                      "text.purchased_land_also_uses_your_claim_allowance",
                                    )
                                  : t(
                                      "text.collect_the_item_in_the_official_smp_after_purchase",
                                    )}
                            </p>
                          </>
                        ),
                        submit: message("text.buy_for_0_coins", money(l.price)),
                      })
                    }
                  >
                    {t("text.buy")}
                  </button>
                )}
              </section>
            ))}
          </div>
        ) : (
          <Empty>
            {t(
              "text.no_listings_in_this_category_deposit_an_asset_to_create_a_listing",
            )}
          </Empty>
        )}
      </PageBlock>{" "}
      {previewAllowed && preview && !preview.result && (
        <Card title={t("text.placement_preview")}>
          <p
            role={preview.error ? "alert" : "status"}
            className={preview.error ? "error" : ""}
          >
            {preview.error
              ? translateError(preview.error)
              : t("text.checking_the_placement_area")}
          </p>
          <button
            onClick={() => showJob(preview.job_id, { kind: "asset.place" })}
          >
            {t("text.view_details")}
          </button>
          <button onClick={() => setPreview(null)}>{t("text.close")}</button>
        </Card>
      )}
      {previewAllowed && preview?.result && (
        <Card title={t("text.placement_preview")}>
          <p>
            {preview.result.clear
              ? t("text.the_placement_area_is_clear")
              : t("text.the_placement_area_is_blocked_clear_it_and_try_again")}
          </p>
          <Manifest value={preview.result} />
          <Actions>
            <button
              className="primary"
              disabled={!preview.result.clear}
              onClick={() =>
                open({
                  title: message("text.place_the_building_here"),
                  ownerScope: {
                    ids: [
                      preview.asset_owner,
                      claims.find(
                        (claim) => claim.id === preview.placement.claim_id,
                      )?.owner,
                    ],
                    permission: "can_build",
                  },
                  note: () => (
                    <>
                      <p>
                        {t(
                          "economy.owner_note",
                          ownerName(preview.asset_owner),
                        )}
                      </p>
                      <p>
                        {t(
                          "text.placing_the_building_consumes_the_packed_asset_pack_it_7287183bd0",
                        )}
                      </p>
                    </>
                  ),
                  submit: message("text.confirm_placement"),
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
              {t("text.place_building")}
            </button>
            <button onClick={() => setPreview(null)}>{t("text.close")}</button>
          </Actions>
        </Card>
      )}
      <PageBlock id="stored-assets">
        <Card title={t("text.stored_assets")}>
          <List
            values={rows(data, "assets").filter(
              (a) => !["placed", "delivered", "cancelled"].includes(a.state),
            )}
            empty={t(
              "text.no_stored_assets_purchases_and_deposited_assets_will_appear_here",
            )}
            render={(a) => (
              <Row
                key={a.id}
                actions={
                  <>
                    <Status value={a.state} />
                    {a.state === "escrowed" && (
                      <>
                        {permitted(a.owner, "can_sell") && (
                          <button
                            onClick={() =>
                              open({
                                title: message("text.set_a_price_and_list"),
                                type: "listing_create",
                                values: { asset: a.id },
                                ownerScope: {
                                  id: a.owner,
                                  permission: "can_sell",
                                },
                                note: () => (
                                  <p>
                                    {t(
                                      "economy.owner_note",
                                      ownerName(a.owner),
                                    )}
                                  </p>
                                ),
                                fields: [
                                  {
                                    name: "price",
                                    label: message("text.price_coins"),
                                    type: "number",
                                    min: 1,
                                    max: 1000000000000,
                                  },
                                ],
                                submit: message("text.create_listing"),
                              })
                            }
                          >
                            {t("text.list_for_sale")}
                          </button>
                        )}
                        {a.kind === "building" &&
                        permitted(a.owner, "can_build") ? (
                          <button
                            onClick={() => placement(a)}
                            disabled={
                              !claims.some(
                                (claim) =>
                                  claim.state === "active" &&
                                  permitted(claim.owner, "can_build"),
                              )
                            }
                          >
                            {t("text.place")}
                          </button>
                        ) : a.kind === "items" &&
                          permitted(a.owner, "can_spend") ? (
                          <button
                            onClick={() => act("asset_receive", { id: a.id })}
                          >
                            {t("text.collect_in_game")}
                          </button>
                        ) : a.kind === "land" &&
                          permitted(a.owner, "can_sell") ? (
                          <button
                            onClick={() =>
                              open({
                                title: message("text.release_deposited_land"),
                                type: "asset_withdraw",
                                values: { id: a.id },
                                ownerScope: {
                                  id: a.owner,
                                  permission: "can_sell",
                                },
                                note: () => (
                                  <>
                                    <p>
                                      {t(
                                        "economy.owner_note",
                                        ownerName(a.owner),
                                      )}
                                    </p>
                                    <p>
                                      {t(
                                        "text.return_the_land_and_buildings_to_normal_use_review_and_40b0bbf97d",
                                      )}
                                    </p>
                                  </>
                                ),
                                submit: message("text.release_deposit"),
                              })
                            }
                          >
                            {t("text.release_deposit")}
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
                              title: message(
                                "text.transfer_a_pet_with_this_building",
                              ),
                              type: "asset_consent",
                              values: {
                                id: a.id,
                                manifest_sha256: a.manifest_sha256,
                              },
                              note: () => (
                                <>
                                  <p>
                                    {t(
                                      "text.i_agree_to_transfer_ownership_of_my_pets_inside_this_bu_b571434076",
                                    )}
                                  </p>
                                  <Manifest value={a.manifest} />
                                </>
                              ),
                              submit: message("text.i_agree"),
                            })
                          }
                        >
                          {t("text.confirm_as_pet_owner")}
                        </button>
                      )}
                    {a.state === "capturing" &&
                      permitted(a.owner, "can_sell") &&
                      a.manifest_sha256 && (
                        <button
                          className="quiet"
                          onClick={() => act("asset_withdraw", { id: a.id })}
                        >
                          {t("text.cancel_packing_request")}
                        </button>
                      )}
                  </>
                }
              >
                <strong>
                  {a.title_message
                    ? renderSystemMessage(a.title_message)
                    : a.title}
                </strong>
                {ownerName(a.owner) && (
                  <small>{t("economy.owner_note", ownerName(a.owner))}</small>
                )}
                <Manifest value={a.manifest} />
              </Row>
            )}
          />
        </Card>
      </PageBlock>
      <PageBlock id="materials">
        <Card title={t("text.sell_materials")}>
          <div className="section-toolbar">
            <div>
              <strong>
                {t("text.remaining_today")}
                {money(data.npc_remaining)} {t("text.coins")}
              </strong>
              <small>
                {t(
                  "text.fixed_prices_2_000_coins_replenished_daily_at_00_00_utc",
                )}
              </small>
            </div>
            <button
              onClick={() =>
                open({
                  title: message("text.sell_materials_now"),
                  type: "npc_sell",
                  fields: [
                    {
                      name: "material",
                      label: message("text.material"),
                      type: "select",
                      options: rows(data, "prices").map((p) => ({
                        value: p.material,
                        label: message(
                          "text.0_1_coins_each",
                          p.material,
                          p.price,
                        ),
                      })),
                    },
                    {
                      name: "amount",
                      label: message("text.quantity"),
                      type: "number",
                      min: 1,
                      max: 2304,
                      value: 1,
                    },
                  ],
                  note: () => (
                    <p>
                      {t(
                        "text.connect_to_the_official_smp_to_sell_materials_from_your_ee12756cac",
                      )}
                    </p>
                  ),
                  submit: message("text.sell_materials_now"),
                })
              }
            >
              {t("text.sell_materials_now")}
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
          {t("text.dimensions")}{" "}
          {Array.isArray(m.dimensions)
            ? m.dimensions.join(" × ")
            : String(m.dimensions)}
        </p>
      )}
      {(m.block_count ?? m.blocks) !== undefined && (
        <p>
          {money(m.block_count ?? m.blocks)} {t("text.blocks")}
        </p>
      )}
      {m.material && m.amount !== undefined && (
        <p>
          {m.material} × {m.amount}
        </p>
      )}
      {m.origin && (
        <p>
          {t("text.origin")}
          {m.origin.join(", ")} {t("text.rotation_922994dc")}
          {m.rotation}°
        </p>
      )}
      {m.footprint && (
        <p>
          {t("text.placement_area_x")}
          {m.footprint.min_x}〜{m.footprint.max_x} / Y {m.footprint.min_y}〜
          {m.footprint.max_y} / Z {m.footprint.min_z}〜{m.footprint.max_z}
        </p>
      )}
      {m.containers?.length > 0 && (
        <details>
          <summary>
            {t("text.container_contents")}
            {m.containers.length} {t("text.stacks")}
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
              {e.trades ? t("text.villager_trades_included") : ""}
            </li>
          ))}
        </ul>
      )}
      {m.contents_included !== undefined && (
        <small>
          {t("text.container_contents_44b4593e")}
          {m.contents_included ? t("text.included") : t("text.not_included")}
        </small>
      )}
    </div>
  );
}

export function Settings({ data }: { data: Data }) {
  const { me, send, act, open } = useApp();
  const [code, setCode] = useState("");
  const policies = [
    { value: "friends", label: message("text.friends_only") },
    { value: "everyone", label: message("text.everyone") },
    { value: "none", label: message("text.nobody") },
  ];
  return (
    <>
      <div className="grid two">
        <PageBlock id="profile">
          <Card title={t("text.profile")}>
            <LanguagePicker
              save={(language) => send("language", { language })}
            />
            <p>{t("text.your_language_is_shared_with_linked_game_accounts")}</p>
            <label className="field">
              {t("text.account_id")}
              <input readOnly value={me.account.id} />
            </label>
            <ActionForm
              fields={[
                {
                  name: "display_name",
                  label: message("text.display_name"),
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
          <Card title={t("text.privacy")}>
            <ActionForm
              fields={[
                {
                  name: "dm_policy",
                  label: message("text.who_can_send_you_dms"),
                  type: "select",
                  value: me.account.dm_policy,
                  options: policies,
                },
                {
                  name: "activity_policy",
                  label: message("text.who_can_see_your_activity"),
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
          <Card title={t("text.link_game_accounts")}>
            <p>
              {t(
                "text.combine_web_java_and_bedrock_identities_into_one_accoun_3de05e9f6d",
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
                  title: message("text.create_a_link_code"),
                  note: () => (
                    <p>
                      {t(
                        "text.enter_this_code_on_your_other_account_it_expires_in_ten_minutes",
                      )}
                    </p>
                  ),
                  submit: message("text.create_code"),
                  action: async () => {
                    const result = await send("link_begin");
                    setCode(result.code);
                  },
                })
              }
            >
              {t("text.create_link_code")}
            </button>
            {code && (
              <p className="link-code">
                <code>{code}</code>
                <small>{t("text.enter_on_your_other_account")}</small>
              </p>
            )}
            <ActionForm
              fields={[
                {
                  name: "code",
                  label: message("text.link_code_from_your_other_account"),
                },
              ]}
              submit={t("text.link_to_this_account")}
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
                          "text.choose_the_game_data_to_keep_using_the_other_data_is_ar_f23b5dd22c",
                        )}
                      </p>
                      {l.profiles?.map((p: Data) => (
                        <button
                          key={p.id}
                          onClick={() =>
                            open({
                              title: message("text.confirm_game_data"),
                              type: "link_confirm",
                              values: { id: l.id, selected_profile: p.id },
                              note: () => (
                                <p>
                                  {t(
                                    "text.use_the_game_data_for_0_the_other_data_is_archived_fini_7307217e2f",
                                    p.name,
                                  )}
                                  {!p.native_uuid &&
                                    t(
                                      "text.this_profile_has_no_game_inventory_or_achievements_so_i_a08b53d1b7",
                                    )}
                                </p>
                              ),
                              submit: message("text.link_using_this_data"),
                            })
                          }
                        >
                          {p.name} · {money(p.wallet.balance)}
                          {t("text.coins")}
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
        <Card title={t("text.blocked_players")}>
          <List
            values={rows(data, "blocks")}
            empty={t("text.no_blocked_players")}
            render={(b) => (
              <Row
                key={b.id}
                actions={
                  <button
                    onClick={() =>
                      act("block", { target: b.id, blocked: false })
                    }
                  >
                    {t("text.remove")}
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
        <Card title={t("text.your_reports")}>
          <p>
            {t(
              "text.administrators_receive_only_the_messages_you_select_and_submit",
            )}
          </p>
          <List
            values={rows(data, "reports")}
            empty={t(
              "text.no_reports_submitted_select_messages_in_chat_to_submit_a_report",
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
            title: message("text.sign_out_of_this_account"),
            submit: message("text.sign_out"),
            action: async () => {
              await api("/auth/logout", { method: "POST", body: "{}" });
              location.assign("/");
            },
          })
        }
      >
        {t("text.sign_out_on_this_device")}
      </button>
    </>
  );
}

export function Admin({ data }: { data: Data }) {
  const { me, open, act } = useApp();
  if (!me.account.administrator)
    return <Empty>{t("text.administrator_access_is_required")}</Empty>;
  return (
    <>
      <PageBlock id="reports">
        <Card title={t("text.reports")}>
          <List
            values={rows(data, "reports")}
            empty={t("text.no_reports_waiting_for_review")}
            render={(r) => (
              <Row
                key={r.id}
                actions={
                  <button
                    onClick={() =>
                      open({
                        title: message("text.review_submitted_evidence"),
                        note: () => (
                          <p>
                            {t(
                              "text.your_access_and_its_time_are_recorded_in_the_audit_log_5fe9e71163",
                            )}
                          </p>
                        ),
                        submit: message("text.review_evidence"),
                        action: async () => {
                          const report = await api(`/api/v1/reports/${r.id}`);
                          queueMicrotask(() =>
                            open({
                              title: message("text.report_evidence"),
                              type: "report_resolve",
                              values: { id: r.id },
                              fields: [
                                {
                                  name: "status",
                                  label: message("text.resolution"),
                                  type: "select",
                                  options: [
                                    {
                                      value: "investigating",
                                      label: message("text.investigating"),
                                    },
                                    {
                                      value: "resolved",
                                      label: message("text.resolved"),
                                    },
                                    {
                                      value: "dismissed",
                                      label: message("text.no_action_needed"),
                                    },
                                  ],
                                },
                                {
                                  name: "resolution",
                                  label: message("text.resolution_notes"),
                                  type: "textarea",
                                },
                              ],
                              note: () => (
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
                    {t("text.open_evidence")}
                  </button>
                }
              >
                <strong>
                  {t("text.report")}
                  {date(r.created_at)}
                </strong>
                <small>{r.status}</small>
              </Row>
            )}
          />
        </Card>
      </PageBlock>
      <PageBlock id="ranks">
        <Card title={t("text.hosting_access_tiers")}>
          <p>
            {t(
              "text.administrators_approve_hosting_limits_separately_from_p_c1d96b5dc1",
            )}
          </p>
          <Actions>
            <button
              onClick={() =>
                open({
                  title: message("text.assign_a_tier"),
                  type: "rank_set",
                  fields: [
                    playerField(),
                    {
                      name: "rank",
                      label: message("text.tier_number"),
                      type: "number",
                      min: 0,
                      max: 32767,
                    },
                  ],
                })
              }
            >
              {t("text.assign_to_player")}
            </button>
            <button
              onClick={() =>
                open({
                  title: message("text.configure_tier_limits"),
                  type: "rank_configure",
                  fields: [
                    {
                      name: "id",
                      label: message("text.tier_number"),
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
                          message("text.server_count"),
                          message("text.concurrent_servers"),
                          message("text.active_memory_mib"),
                          message("text.cpu_1_core_1000"),
                          message("text.storage_mib"),
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
              {t("text.save_tier")}
            </button>
            <button
              onClick={() =>
                open({
                  title: message("text.set_access_restriction"),
                  type: "ban",
                  fields: [
                    playerField(),
                    {
                      name: "hours",
                      label: message("text.hours_0_to_remove"),
                      type: "number",
                      min: 0,
                      max: 876000,
                      value: 24,
                    },
                    {
                      name: "reason",
                      label: message("text.reason"),
                      type: "textarea",
                    },
                  ],
                  submit: message("text.apply_restriction"),
                })
              }
            >
              {t("text.access_restrictions")}
            </button>
          </Actions>
          <List
            values={rows(data, "ranks")}
            empty={t("text.no_tiers_configured")}
            render={(r) => (
              <Row key={r.id}>
                <strong>
                  {r.id} ·{" "}
                  {r.name_message
                    ? renderSystemMessage(r.name_message)
                    : r.name}
                </strong>
                <p>
                  {t("text.created")}
                  {r.server_count} {t("text.concurrent")}
                  {r.concurrent_servers} · {money(r.memory_mib)} MiB ·{" "}
                  {r.cpu_millis / 1000} {t("text.cores_storage")}{" "}
                  {money(r.storage_mib)} MiB
                </p>
              </Row>
            )}
          />
        </Card>
      </PageBlock>
      <PageBlock id="backups">
        <Card
          title={t("text.official_backups")}
          action={
            <button onClick={() => act("official_backup")}>
              {t("text.back_up_all_official_data")}
            </button>
          }
        >
          <p>
            {t(
              "text.save_worlds_inventories_claims_ledgers_and_stored_assets_together",
            )}
          </p>
          <p>
            {data.backup_policy?.enabled
              ? t(
                  "text.daily_at_0_00_utc_keeps_seven_daily_and_four_weekly_suc_fc7735a0cc",
                  String(data.backup_policy.hour_utc).padStart(2, "0"),
                )
              : t("text.automatic_backups_are_currently_disabled")}
            {t(
              "text.manual_and_pinned_backups_are_excluded_from_automatic_pruning",
            )}
          </p>
          <p>
            {t("text.last_completed_backup")}
            {data.backup_policy?.last_completed_at
              ? date(data.backup_policy.last_completed_at)
              : t("text.no_record")}
          </p>
          <List
            values={rows(data, "backups")}
            empty={t("text.no_backups_recorded")}
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
                          {b.pinned
                            ? t("text.unpin_backup")
                            : t("text.pin_backup")}
                        </button>
                      )}
                  </>
                }
              >
                <strong>
                  {b.kind === "official"
                    ? t("text.all_official_data")
                    : t("text.personal_server")}
                </strong>
                <small>{date(b.created_at)}</small>
                {b.kind === "official" && (
                  <p>
                    {b.scheduled_for
                      ? t("text.daily_automatic_backup")
                      : t("text.manual_backup")}
                    {b.pinned ? t("text.pinned") : ""}
                    {b.completed_at
                      ? t("text.completed_0", date(b.completed_at))
                      : ""}
                  </p>
                )}
                {b.error && <p className="error">{translateError(b.error)}</p>}
              </Row>
            )}
          />
        </Card>
      </PageBlock>
      <PageBlock id="operations">
        <AdminOperations data={data} />
      </PageBlock>
      <PageBlock id="audit">
        <Card title={t("text.audit_log")}>
          <List
            values={rows(data, "audit")}
            empty={t("text.no_administrative_actions_recorded")}
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
