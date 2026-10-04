import { useEffect, useState } from "react";
import { useApp } from "./App";
import { date, money, type Data } from "./api";
import { t, message } from "./i18n";
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
  const blocked =
    server.maintenance ||
    !server.capabilities?.proxy_join ||
    projection?.observation_fresh === false;
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
        : server.maintenance
          ? t("text.under_maintenance")
          : !server.capabilities?.proxy_join
            ? t("text.connection_unavailable")
            : projection?.observation_fresh === false
              ? t("text.checking_world_status")
              : server.observed === "running"
                ? t("text.join_world")
                : t("text.wake_and_join")}
      <Icon name="arrow" />
    </button>
  );
}
export function PlayHub({ data }: { data: Data }) {
  const { me, act } = useApp();
  const servers: Data[] = data.servers ?? [];
  const play = data.play ?? {};
  const preferred =
    servers.find((s) => s.id === play.preferred_server_id) ??
    servers.find((s) => s.kind === "official") ??
    servers[0];
  const online = !!play.game_session;
  const linked = play.identity_ready === true;
  return (
    <div className="play-layout">
      <section className="play-hero">
        <div className="hero-copy">
          <span className="pill">
            {t("text.welcome_back_0", me.account.name)}
          </span>
          <h2>
            {preferred
              ? t("text.your_world_is_waiting")
              : t("text.make_room_for_an_adventure")}
          </h2>
          <p>
            {preferred
              ? t("text.pick_up_where_you_left_off_or_discover_somewhere_new")
              : t("text.explore_available_worlds_and_find_people_to_play_with")}
          </p>
          {preferred && (
            <div className="resume-world">
              <span className="world-mark">
                <Icon name="life" />
              </span>
              <div>
                <strong>{preferred.name}</strong>
                <small>
                  {preferred.kind === "official"
                    ? t("text.community_survival")
                    : t("text.community_world")}
                </small>
              </div>
              <Status value={preferred.observed} />
            </div>
          )}
          <div className="hero-actions">
            {!linked ? (
              <a className="button primary" href="#/account/linking">
                {t("text.link_your_minecraft_account")}
                <Icon name="arrow" />
              </a>
            ) : !online ? (
              <button
                className="primary"
                disabled={!me.game_address}
                onClick={() =>
                  void navigator.clipboard?.writeText(me.game_address ?? "")
                }
              >
                {t("text.copy_minecraft_address")}
                <Icon name="arrow" />
              </button>
            ) : preferred ? (
              <JoinWorld
                server={{ ...preferred, play_session: play.game_session }}
                primary
              />
            ) : null}
            <a className="button quiet" href="#/worlds">
              {t("text.explore_worlds")}
            </a>
          </div>
          <p className="join-guidance">
            {!linked
              ? t(
                  "text.connect_your_java_or_bedrock_identity_to_join_from_the_web",
                )
              : !online
                ? t(
                    "text.open_minecraft_connect_to_0_then_return_here_to_choose_a_world",
                    me.game_address,
                  )
                : t(
                    "text.minecraft_is_connected_your_transfer_completes_when_you_arrive",
                  )}
          </p>
        </div>
        <WorldArt kind={preferred?.kind ?? "official"} />
      </section>
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
      <div className="section-toolbar">
        <p>
          {t(
            "text.persistent_worlds_to_build_in_temporary_expeditions_to_explore",
          )}
        </p>
        <a className="button violet" href="#/expeditions">
          {t("text.explore_expeditions")}
          <Icon name="adventure" />
        </a>
      </div>
      <div className="world-grid">
        {worlds.map((world) => (
          <article className="world-card" key={world.id}>
            <a
              className="world-card-cover"
              href={"#/worlds/" + world.id}
              aria-label={world.name}
            >
              <WorldArt kind={world.kind} />
              <span className="pill">
                {world.kind === "official"
                  ? t("text.community_survival")
                  : t("text.community_world")}
              </span>
            </a>
            <div className="world-card-body">
              <div className="card-head">
                <h2>
                  <a href={"#/worlds/" + world.id}>{world.name}</a>
                </h2>
                <Status value={world.observed} />
              </div>
              <p>
                {world.kind === "official"
                  ? t(
                      "text.build_a_home_protect_your_land_and_share_a_world_with_t_219bdca23d",
                    )
                  : t("text.a_world_created_and_hosted_by_the_community")}
              </p>
              <div className="world-meta">
                <span>
                  {money(world.players)} {t("text.players_online_ac36de3e")}
                </span>
                <span>
                  {world.capabilities?.bedrock ? "Java · Bedrock" : "Java"}
                </span>
              </div>
              <div className="actions">
                <a className="button primary" href={"#/worlds/" + world.id}>
                  {t("text.open_world")}
                  <Icon name="arrow" />
                </a>
              </div>
            </div>
          </article>
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
      <section className="world-detail-hero">
        <WorldArt kind={world.kind} />
        <div className="hero-copy">
          <span className="pill">
            {world.kind === "official"
              ? t("text.community_survival")
              : t("text.community_world")}
          </span>
          <h2>{world.name}</h2>
          <p>
            {world.kind === "official"
              ? t(
                  "text.a_lasting_home_for_your_creations_your_team_and_your_ne_70d36513a4",
                )
              : t("text.discover_what_this_community_is_building")}
          </p>
          <div className="hero-actions">
            <JoinWorld server={{ ...world, play: data.play }} primary />
            {world.can_manage && (
              <a
                className="button quiet"
                href={"#/hosting/servers/" + world.id}
              >
                {t("text.open_hosting_tools")}
              </a>
            )}
          </div>
        </div>
      </section>
      <div className="world-overview-grid">
        <Card title={t("text.at_a_glance")}>
          <dl className="details-list">
            <div>
              <dt>{t("text.world_status")}</dt>
              <dd>
                <Status value={world.observed} />
              </dd>
            </div>
            <div>
              <dt>{t("text.players_online_ae9bb529")}</dt>
              <dd>{money(world.players)}</dd>
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
                  <small>
                    {t("text.protect_a_place_to_build_and_return_to_it")}
                  </small>
                </div>
                <Icon name="arrow" />
              </a>
              <a href={"#/worlds/" + world.id + "/economy"}>
                <Icon name="market" />
                <div>
                  <strong>{t("text.wallet_and_market")}</strong>
                  <small>{t("text.trade_materials_buildings_and_land")}</small>
                </div>
                <Icon name="arrow" />
              </a>
              <a href="#/expeditions">
                <Icon name="adventure" />
                <div>
                  <strong>{t("text.end_expeditions")}</strong>
                  <small>
                    {t("text.temporary_worlds_for_a_shared_adventure")}
                  </small>
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
        ["wallet", "text.wallet", "text.coins_62f014cb"],
        ["market", "text.market", "market"],
        ["storage", "text.storage", "stored-assets"],
        ["materials", "Materials", "materials"],
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
  const { open, act, isWorking } = useApp();
  const [now, setNow] = useState(Date.now());
  useEffect(() => {
    const timer = setInterval(() => setNow(Date.now()), 30000);
    return () => clearInterval(timer);
  }, []);
  const entries: Data[] = data.expeditions ?? [];
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
          <button
            className="primary violet"
            disabled={!configured || isWorking("expedition_prepare")}
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
      <Card title={t("text.current_expeditions")}>
        {current.length ? (
          current.map(row)
        ) : (
          <Empty>
            {t("text.no_expedition_is_underway_prepare_one_when_you_are_ready")}
          </Empty>
        )}
      </Card>
      <Card title={t("text.expedition_journal")}>
        {journal.length ? (
          journal.map(row)
        ) : (
          <Empty>{t("text.your_completed_expeditions_will_appear_here")}</Empty>
        )}
      </Card>
    </>
  );
}
