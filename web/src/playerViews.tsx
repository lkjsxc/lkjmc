import { useEffect, useState } from "react";
import { useApp } from "./App";
import { date, money, type Data } from "./api";
import { t, message } from "./i18n";
import { hostingActionReasons } from "./hostingStatus";
import { Card, Empty, Icon, Status } from "./ui";

function WorldArt({ kind = "custom" }: { kind?: string }) {
  return (
    <div className={`world-art world-art-${kind}`} aria-hidden="true">
      <span className="world-orbit" />
      <span className="block block-one" />
      <span className="block block-two" />
      <span className="block block-three" />
    </div>
  );
}
function JoinWorld({
  server,
  primary = false,
}: {
  server: Data;
  primary?: boolean;
}) {
  const { me, act, isWorking } = useApp();
  const projection = server.status;
  const identityReady =
    server.play?.identity_ready ??
    me.account.identities?.some((identity: Data) =>
      ["java", "bedrock"].includes(identity.issuer),
    );
  const session = server.play_session ?? server.play?.game_session;
  const blocked = projection?.actions?.join?.allowed !== true;
  const pending = isWorking("server_join", { id: server.id });
  if (!identityReady)
    return (
      <a className="button primary" href="#/account/linking">
        {t("text.link_a_game_account")}
      </a>
    );
  if (!session)
    return (
      <a className="button primary" href="#/play">
        {t("text.open_minecraft_to_join")}
      </a>
    );
  return (
    <button
      className={primary ? "primary" : ""}
      disabled={!!blocked || pending}
      onClick={() => act("server_join", { id: server.id })}
    >
      {pending
        ? t("text.joining")
        : blocked
          ? t(
              hostingActionReasons[projection?.actions?.join?.reason] ??
                "text.connection_unavailable",
            )
          : projection?.game_state === "running"
            ? t("text.join_world")
            : t("text.wake_and_join")}

      <Icon name="arrow" />
    </button>
  );
}
export function PlayHub({ data }: { data: Data }) {
  const { act } = useApp();
  return (
    <div className="play-layout">
      <Worlds data={data} />
      <div className="play-secondary">
        <Card
          title={t("text.ready_to_play")}
          action={
            <a href="#/people">
              {t("text.people_7db20897")}
              <Icon name="arrow" />
            </a>
          }
        >
          {data.friends?.length ? (
            <div className="presence-list">
              {data.friends.map((friend: Data) => (
                <a className="presence-person" href="#/people" key={friend.id}>
                  <span className="avatar">{friend.name.slice(0, 1)}</span>
                  <div>
                    <strong>{friend.name}</strong>
                    <small>{t("text.online_in_game")}</small>
                  </div>
                  <span className="presence-dot" />
                </a>
              ))}
            </div>
          ) : (
            <Empty>
              {t(
                "text.invite_a_friend_and_make_your_next_session_a_shared_one",
              )}
            </Empty>
          )}
        </Card>
        <Card
          title={t("text.invitations")}
          action={<a href="#/play/invitations">{t("text.view_all")}</a>}
        >
          {data.invitations?.length ? (
            <div className="list">
              {data.invitations.map((invite: Data) => (
                <div className="list-row" key={invite.id}>
                  <div>
                    <strong>{invite.sender_name}</strong>
                    <small>{t("text.invited_you_to_play_together")}</small>
                  </div>
                  <div className="actions">
                    <button
                      className="primary"
                      onClick={() =>
                        act("invite_respond", { id: invite.id, accept: true })
                      }
                    >
                      {t("text.accept")}
                    </button>
                    <button
                      className="quiet"
                      onClick={() =>
                        act("invite_respond", { id: invite.id, accept: false })
                      }
                    >
                      {t("text.decline")}
                    </button>
                  </div>
                </div>
              ))}
            </div>
          ) : (
            <Empty>
              {t(
                "text.no_invitations_waiting_your_next_adventure_is_up_to_you",
              )}
            </Empty>
          )}
        </Card>
      </div>
    </div>
  );
}
export function Worlds({ data }: { data: Data }) {
  const worlds: Data[] = data.servers ?? [];
  return (
    <>
      <div className="world-grid">
        {worlds.map((world) => (
          <a
            className="world-card"
            key={world.id}
            href={"#/worlds/" + world.id}
            aria-label={world.name}
          >
            <div className="world-card-body">
              <div className="card-head">
                <h2>{world.name}</h2>
                <Status value={world.status?.game_state ?? "unknown"} />
              </div>
              <div className="world-meta">
                <span>
                  {world.status?.observation_fresh ? money(world.players) : "—"}{" "}
                  {t("text.players_online_ac36de3e")}
                </span>
                <span>
                  {world.capabilities?.bedrock ? "Java · Bedrock" : "Java"}
                </span>
                <Icon name="arrow" />
              </div>
            </div>
          </a>
        ))}
      </div>
      {!worlds.length && <Empty>{t("text.no_worlds_are_available_yet")}</Empty>}
    </>
  );
}
export function WorldOverview({ data }: { data: Data }) {
  const { me } = useApp();
  const world = data.server;
  if (!world) return <Empty>{t("text.this_world_is_unavailable")}</Empty>;
  return (
    <>
      <div className="world-actions actions">
        <JoinWorld server={{ ...world, play: data.play }} primary />
        {world.can_manage && (
          <a className="button quiet" href={"#/hosting/servers/" + world.id}>
            {t("text.hosting")}
          </a>
        )}
      </div>
      <div className="world-overview-grid">
        <Card title={t("text.at_a_glance")}>
          <dl className="details-list">
            <div>
              <dt>{t("text.world_status")}</dt>
              <dd>
                <Status value={world.status?.game_state ?? "unknown"} />
              </dd>
            </div>
            <div>
              <dt>{t("text.players_online_ae9bb529")}</dt>
              <dd>
                {world.status?.observation_fresh ? money(world.players) : "—"}
              </dd>
            </div>
            <div>
              <dt>{t("text.edition")}</dt>
              <dd>{world.capabilities?.bedrock ? "Java / Bedrock" : "Java"}</dd>
            </div>
          </dl>
          {world.capabilities?.client_mods && (
            <p className="notice">
              {t("text.the_specified_client_mods_are_required")}
            </p>
          )}
        </Card>
        {world.kind === "official" && (
          <Card title={t("text.make_this_world_yours")}>
            <div className="task-links">
              <a href={"#/worlds/" + world.id + "/world"}>
                <Icon name="life" />
                <div>
                  <strong>{t("text.your_land_and_homes")}</strong>
                </div>
                <Icon name="arrow" />
              </a>
              <a href={"#/worlds/" + world.id + "/economy"}>
                <Icon name="market" />
                <div>
                  <strong>{t("text.wallet_and_market")}</strong>
                </div>
                <Icon name="arrow" />
              </a>
              <a href={"#/worlds/" + world.id + "/expeditions"}>
                <Icon name="adventure" />
                <div>
                  <strong>{t("text.end_expeditions")}</strong>
                </div>
                <Icon name="arrow" />
              </a>
            </div>
          </Card>
        )}
      </div>
      <details className="technical-details">
        <summary>{t("text.connection_and_software")}</summary>
        <dl className="details-list">
          <div>
            <dt>{t("text.address")}</dt>
            <dd>
              <code>{me.game_address}</code>
            </dd>
          </div>
          <div>
            <dt>{t("text.server_software")}</dt>
            <dd>
              {world.software} {world.version}
            </dd>
          </div>
          <div>
            <dt>{t("text.last_checked")}</dt>
            <dd>
              {date(world.last_observed_at) || t("text.not_observed_yet")}
            </dd>
          </div>
        </dl>
      </details>
    </>
  );
}
export function WorldToolNavigation() {
  const { route } = useApp();
  const economy = route.path.includes("/economy");
  const tabs = economy
    ? [
        ["wallet", "text.wallet", "coins"],
        ["market", "text.market", "market"],
        ["storage", "text.storage", "stored-assets"],
        ["materials", "text.sell_materials", "materials"],
        ["history", "text.coin_history", "coin-history"],
      ]
    : [
        ["land", "text.protected_land", "land"],
        ["homes", "text.homes", "homes"],
        ["meetup", "text.meet_up", "meetup"],
        ["achievements", "text.achievements", "achievements"],
      ];
  return (
    <nav className="task-tabs" aria-label={t("text.world_tools")}>
      {tabs.map(([tab, label, section]) => (
        <a
          key={tab}
          href={
            "#/worlds/" +
            route.id +
            (economy ? "/economy" : "/world") +
            "?tab=" +
            tab
          }
          aria-current={route.section === section ? "page" : undefined}
        >
          {t(label)}
        </a>
      ))}
    </nav>
  );
}
export function PeopleNavigation() {
  const { route } = useApp();
  const group = route.path.split("/")[2] ?? "friends";
  return (
    <nav className="people-tabs" aria-label={t("text.people_groups")}>
      {[
        ["friends", "text.friends", "social"],
        ["teams", "text.teams", "teams"],
        ["parties", "text.parties", "parties"],
      ].map(([id, label, icon]) => (
        <a
          href={"#/people/" + id}
          key={id}
          aria-current={group === id ? "page" : undefined}
        >
          <Icon name={icon} />
          {t(label)}
        </a>
      ))}
    </nav>
  );
}
export function Expeditions({ data }: { data: Data }) {
  const { open, act, isWorking, route } = useApp();
  const base = "#/worlds/" + route.id + "/expeditions";
  const [now, setNow] = useState(Date.now());
  useEffect(() => {
    const timer = setInterval(() => setNow(Date.now()), 30000);
    return () => clearInterval(timer);
  }, []);
  const entries: Data[] = data.expedition
    ? [data.expedition]
    : (data.expeditions ?? []);
  const overview = route.section === "expeditions";
  const preparation = data.preparation;
  const participants: Data[] = preparation?.participants ?? [];
  const participantReason = (participant: Data) =>
    participant.occupied
      ? "text.finish_the_current_expedition_first"
      : !participant.online
        ? "text.must_be_online_in_smp"
        : participant.in_combat
          ? "text.wait_until_combat_ends"
          : !participant.ready
            ? "text.consent_needed"
            : "text.ready_for_expedition";
  const current = entries.filter(
    (entry) =>
      !["closed", "refunded", "cancelled", "failed"].includes(entry.state),
  );
  const journal = entries.filter((entry) =>
    ["closed", "refunded", "cancelled", "failed"].includes(entry.state),
  );
  const cost = data.cost;
  const duration = Number(data.duration_seconds);
  const configured =
    cost &&
    Number.isFinite(Number(cost.coins)) &&
    Number.isFinite(Number(cost.ender_eyes)) &&
    duration > 0;
  const canPrepare = configured && preparation?.can_prepare === true;
  const blockedParticipant = participants.find(
    (participant) =>
      participantReason(participant) !== "text.ready_for_expedition",
  );
  const preparationReason = !preparation
    ? "text.checking_availability"
    : !preparation.is_leader
      ? "text.ask_your_party_leader_to_prepare_this_expedition"
      : blockedParticipant
        ? participantReason(blockedParticipant)
        : Number(preparation.available_coins) < Number(cost?.coins)
          ? "text.more_coins_needed"
          : "text.unavailable";
  const row = (entry: Data) => (
    <article className="expedition-entry" key={entry.id}>
      <div className="card-head">
        <div>
          <span className="pill violet">{t("text.end_expedition")}</span>
          <h3>
            {entry.participants
              ?.map((participant: Data) => participant.name)
              .join(", ") || t("text.your_expedition")}
          </h3>
        </div>
        <Status value={entry.state} />
      </div>
      <dl className="details-list">
        {entry.expires_at && (
          <div>
            <dt>{t("text.world_closes")}</dt>
            <dd>
              {date(entry.expires_at)}
              {entry.state === "active" && (
                <small>
                  {t(
                    "text.0_minutes_remaining",
                    Math.max(
                      0,
                      Math.ceil((Date.parse(entry.expires_at) - now) / 60000),
                    ),
                  )}
                </small>
              )}
            </dd>
          </div>
        )}
        <div>
          <dt>{t("text.prepared")}</dt>
          <dd>{date(entry.created_at)}</dd>
        </div>
      </dl>
      <div className="actions">
        {route.section !== "detail" && (
          <a className="button quiet" href={base + "/" + entry.id}>
            {t("text.view_details")}
          </a>
        )}
        {entry.can_enter && (
          <button
            className="primary"
            disabled={isWorking("expedition_enter", { id: entry.id })}
            onClick={() => act("expedition_enter", { id: entry.id })}
          >
            {t("text.enter_expedition")}
          </button>
        )}
        {entry.can_return && (
          <button
            disabled={isWorking("expedition_return", { id: entry.id })}
            onClick={() => act("expedition_return", { id: entry.id })}
          >
            {t("text.return_to_survival")}
          </button>
        )}
        {entry.can_cancel && (
          <button
            className="quiet"
            onClick={() =>
              open({
                title: message("text.cancel_expedition_preparation"),
                type: "expedition_cancel",
                values: { id: entry.id },
                note: () => (
                  <p>
                    {t(
                      "text.reserved_coins_are_released_reserved_items_return_to_st_497815c001",
                    )}
                  </p>
                ),
                submit: message("text.cancel_and_refund"),
              })
            }
          >
            {t("text.cancel_preparation")}
          </button>
        )}
        {entry.can_receive && (
          <button
            onClick={() => act("asset_receive", { id: entry.material_asset })}
          >
            {t("text.collect_returned_items")}
          </button>
        )}
      </div>
    </article>
  );
  return (
    <>
      {overview && (
        <section className="expedition-hero">
          <div className="hero-copy">
            <span className="pill violet">
              {t("text.temporary_world_end_dimension")}
            </span>
            <h2>{t("text.go_somewhere_that_will_not_last")}</h2>
            <p>
              {t(
                "text.prepare_an_end_world_for_yourself_or_your_party_bring_y_c0232e809a",
              )}
            </p>
            {configured ? (
              <div className="expedition-requirements">
                <div>
                  <strong>{money(cost.coins)}</strong>
                  <span>{t("text.coins_62f014cb")}</span>
                </div>
                <div>
                  <strong>{money(cost.ender_eyes)}</strong>
                  <span>{t("text.eyes_of_ender")}</span>
                </div>
                <div>
                  <strong>{money(duration / 3600)}</strong>
                  <span>{t("text.hours_after_opening")}</span>
                </div>
              </div>
            ) : (
              <p className="notice">
                {t(
                  "text.expedition_requirements_are_unavailable_try_again_once_a49dd28615",
                )}
              </p>
            )}
            {preparation && (
              <div className="expedition-preparation">
                <h3>{t("text.party_readiness")}</h3>
                <p>
                  {t(
                    "text.available_0_coins",
                    money(preparation.available_coins),
                  )}
                </p>
                <ul className="preparation-roster">
                  {participants.map((participant) => (
                    <li key={participant.account_id ?? participant.name}>
                      <strong>{participant.name}</strong>
                      <span
                        className={
                          participantReason(participant) ===
                          "text.ready_for_expedition"
                            ? "ready"
                            : "muted"
                        }
                      >
                        {t(participantReason(participant))}
                      </span>
                    </li>
                  ))}
                </ul>
              </div>
            )}
            {configured && !canPrepare && (
              <p className="notice preparation-reason" role="status">
                {t(preparationReason)}{" "}
                {t("text.resolve_the_requirements_below_then_refresh")}{" "}
                <a href="#/people/parties/ready">{t("text.party_readiness")}</a>
              </p>
            )}
            <button
              className="primary violet"
              disabled={!canPrepare || isWorking("expedition_prepare")}
              onClick={() =>
                open({
                  title: message("text.prepare_an_end_expedition"),
                  type: "expedition_prepare",
                  note: () => (
                    <div>
                      <p>
                        {t(
                          "text.reserve_0_coins_and_1_eyes_of_ender_every_party_member_02a73e96e1",
                          money(cost.coins),
                          money(cost.ender_eyes),
                        )}
                      </p>
                      <p>
                        {t(
                          "text.the_participant_list_locks_when_preparation_starts_the_2ebd0e5b24",
                          money(duration / 3600),
                        )}
                      </p>
                      <p>
                        {t(
                          "text.cancelling_before_opening_or_a_failed_start_refunds_the_567f5e6471",
                        )}
                      </p>
                    </div>
                  ),
                  submit: message("text.reserve_and_prepare"),
                })
              }
            >
              {t("text.prepare_expedition")}
              <Icon name="arrow" />
            </button>
          </div>
          <WorldArt kind="expedition" />
        </section>
      )}
      {overview ? (
        <>
          <Card title={t("text.current_expeditions")}>
            {current.length ? (
              current.map(row)
            ) : (
              <Empty>
                {t(
                  "text.no_expedition_is_underway_prepare_one_when_you_are_ready",
                )}
              </Empty>
            )}
          </Card>
          <Card
            title={t("text.expedition_journal")}
            action={
              <a className="button quiet" href={base + "/journal"}>
                {t("text.view_details")}
              </a>
            }
          >
            {journal.length ? (
              journal.slice(0, 5).map(row)
            ) : (
              <Empty>
                {t("text.your_completed_expeditions_will_appear_here")}
              </Empty>
            )}
          </Card>
        </>
      ) : (
        <>
          <a className="button quiet" href={base}>
            {t("text.expeditions")}
          </a>
          <Card
            title={t(
              route.section === "detail"
                ? "text.end_expedition"
                : "text.expedition_journal",
            )}
          >
            {entries.length ? (
              entries.map(row)
            ) : (
              <Empty>
                {t("text.your_completed_expeditions_will_appear_here")}
              </Empty>
            )}
          </Card>
        </>
      )}
    </>
  );
}
