import { useEffect, useRef, useState } from "react";
import { api, type Data } from "./api";
import { useApp } from "./App";
import { t, message, messageError } from "./i18n";
import { unreadable } from "./identity";
import { Card, Empty, Icon, type Field } from "./ui";

const capabilityLabels = [
  ["can_build", "text.build"],
  ["can_sell", "text.sell"],
  ["can_spend", "text.spend_shared_coins"],
  ["can_manage_members", "text.manage_members"],
  ["can_administer", "text.manage_team"],
] as const;

function permissionFields(member: Data, leader: boolean): Field[] {
  const fields: Field[] = [
    {
      name: "build",
      label: message("text.build"),
      type: "checkbox",
      value: !!member.can_build,
    },
    {
      name: "sell",
      label: message("text.sell"),
      type: "checkbox",
      value: !!member.can_sell,
    },
    {
      name: "spend",
      label: message("text.spend_shared_coins"),
      type: "checkbox",
      value: !!member.can_spend,
    },
    {
      name: "members",
      label: message("text.manage_members"),
      type: "checkbox",
      value: !!member.can_manage_members,
    },
  ];
  if (leader)
    fields.push({
      name: "administer",
      label: message("text.manage_team"),
      type: "checkbox",
      value: !!member.can_administer,
    });
  return fields;
}

/** Team IDs are always explicit; contribution selection never supplies an owner. */
export function Teams({ data }: { data: Data }) {
  const { me, route, open, send, isWorking, go } = useApp();
  const teams: Data[] = data.teams ?? [];
  const team: Data | undefined = data.team;
  const create = () =>
    open({
      title: message("text.create_team"),
      type: "team_create",
      fields: [{ name: "name", label: message("text.team_name"), max: 64 }],
      submit: message("text.create_team"),
      action: async (values) => {
        const result = await send("team_create", values);
        if (result.team_id) go("/people/teams/" + result.team_id);
        return result;
      },
    });
  if (!route.id) {
    return (
      <div className="teams-workspace">
        {!teams.length ? (
          <Empty>{t("team.empty")}</Empty>
        ) : (
          <ul className="team-list">
            {teams.map((item) => (
              <li key={item.id}>
                <a
                  className="team-list-item"
                  href={"#/people/teams/" + item.id}
                  aria-label={item.name}
                >
                  <div className="grow">
                    <strong className="team-name">{item.name}</strong>
                    <small>{t("team.members_count", item.member_count)}</small>
                  </div>
                  <Icon name="arrow" />
                </a>
              </li>
            ))}
          </ul>
        )}
        <div className="team-collection-head">
          <button onClick={create} disabled={isWorking("team_create")}>
            <Icon name="plus" />
            {t("text.create_team")}
          </button>
        </div>
      </div>
    );
  }
  if (!team || team.id !== route.id) return null;
  const leader = team.leader === me.account.id;
  const permissions = team.permissions ?? {};
  return (
    <div className="teams-workspace">
      <div className="team-detail-head">
        <a href="#/people/teams">{t("team.back")}</a>
        <span>{t("team.members_count", team.member_count)}</span>
      </div>
      {route.section === "team" && (
        <>
          <div className="team-overview-grid">
            <Card title={team.name}>
              <a href={"#/timeline?room=" + team.room_id}>
                {t("text.team_chat")}
              </a>
            </Card>
            <Card title={t("team.your_permissions")}>
              {capabilityLabels.some(([key]) => permissions[key]) ? (
                <ul className="team-permissions">
                  {capabilityLabels
                    .filter(([key]) => permissions[key])
                    .map(([key, label]) => (
                      <li key={key}>{t(label)}</li>
                    ))}
                </ul>
              ) : (
                <p>{t("team.no_permissions")}</p>
              )}
            </Card>
          </div>
        </>
      )}
      {route.section === "team-members" && (
        <TeamMembers key={team.id} data={data} />
      )}
      {route.section === "team-settings" && (
        <Card title={t("text.settings")}>
          <p>{t("team.permissions_intro")}</p>
          {leader && <p>{t("team.leader_leave_note")}</p>}
          <div className="actions">
            <button
              className="quiet"
              disabled={leader || isWorking("team_leave", { team: team.id })}
              onClick={() =>
                open({
                  title: message("team.leave_title", team.name),
                  type: "team_leave",
                  values: { team: team.id },
                  teamScope: { id: team.id, permission: "member" },
                  note: () => (
                    <p>
                      {t(
                        "text.land_and_shared_assets_stay_with_the_team_transfer_lead_2f8f3742f8",
                      )}
                    </p>
                  ),
                  submit: message("text.leave_team"),
                  action: async () => {
                    const result = await send("team_leave", { team: team.id });
                    go("/people/teams");
                    return result;
                  },
                })
              }
            >
              {t("text.leave_team")}
            </button>
            {leader && (
              <button
                className="quiet danger"
                onClick={() =>
                  open({
                    title: message("team.disband_title", team.name),
                    type: "team_disband",
                    values: { team: team.id },
                    teamScope: { id: team.id, permission: "leader" },
                    note: () => (
                      <p>
                        {t(
                          "text.disband_a_team_after_disposing_of_its_land_stored_asset_3fd66c1a99",
                        )}
                      </p>
                    ),
                    submit: message("text.confirm_disbanding"),
                    action: async () => {
                      const result = await send("team_disband", {
                        team: team.id,
                      });
                      go("/people/teams");
                      return result;
                    },
                  })
                }
              >
                {t("text.disband")}
              </button>
            )}
          </div>
        </Card>
      )}
    </div>
  );
}

function TeamMembers({ data }: { data: Data }) {
  const { me, route, open, refresh } = useApp();
  const team = data.team;
  const leader = team.leader === me.account.id;
  const permissions = team.permissions ?? {};
  const [pages, setPages] = useState<Data[]>([]);
  const [cursor, setCursor] = useState<string | null>(team.members_next_after);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<unknown>(null);
  const [revoked, setRevoked] = useState(false);
  const controller = useRef<AbortController | null>(null);
  const origin = useRef(route.path);
  origin.current = route.path;
  // Changed membership/capabilities must invalidate expanded private member pages.
  const membershipVersion = JSON.stringify([
    team.permissions,
    team.member_count,
    team.members,
  ]);
  useEffect(() => {
    setPages([]);
    setCursor(team.members_next_after);
    setError(null);
    setRevoked(false);
    setLoading(false);
    return () => controller.current?.abort();
  }, [team.id, me.account.id, membershipVersion]);
  const members = [...team.members, ...pages].filter(
    (member, index, all) =>
      all.findIndex((item) => item.account_id === member.account_id) === index,
  );
  async function loadMore() {
    if (!cursor || loading) return;
    const source = route.path;
    const request = new AbortController();
    controller.current?.abort();
    controller.current = request;
    setLoading(true);
    setError(null);
    try {
      const result = await api(
        "/api/v1/teams/" + team.id + "?after=" + encodeURIComponent(cursor),
        { signal: request.signal },
      );
      if (request.signal.aborted || origin.current !== source) return;
      if (
        result.team.leader !== team.leader ||
        JSON.stringify(result.team.permissions) !== JSON.stringify(permissions)
      ) {
        // A pagination read also reauthorizes. Wait for the fresh page projection
        // before offering controls based on permissions that have changed.
        setPages([]);
        setRevoked(true);
        refresh();
        return;
      }
      setPages((previous) => [...previous, ...result.team.members]);
      setCursor(result.team.members_next_after);
    } catch (reason) {
      if (request.signal.aborted || origin.current !== source) return;
      if (unreadable(reason)) {
        setPages([]);
        setRevoked(true);
        refresh();
      }
      setError(reason);
    } finally {
      if (controller.current === request) setLoading(false);
    }
  }
  const role = (member: Data) =>
    member.account_id === team.leader
      ? "team.leader_role"
      : member.can_administer
        ? "team.administrator_role"
        : "team.member_role";
  return (
    <Card
      title={t("text.members")}
      action={
        !revoked && permissions.can_manage_members ? (
          <button
            onClick={() =>
              open({
                title: message("team.invite_title", team.name),
                type: "invite",
                values: { kind: "team", resource: team.id },
                teamScope: { id: team.id, permission: "members" },
                fields: [
                  {
                    name: "target",
                    label: message("text.invite_a_player"),
                    type: "player",
                  },
                ],
                submit: message("text.invite"),
              })
            }
          >
            {t("text.invite_member")}
          </button>
        ) : undefined
      }
    >
      {!!error && (
        <p role="alert" className="error">
          {messageError(error)}
        </p>
      )}
      {!revoked && (
        <>
          <ul className="team-member-list">
            {members.map((member) => (
              <li className="team-member-row" key={member.account_id}>
                <div className="grow">
                  <strong>{member.name}</strong>
                  <small>{t(role(member))}</small>
                </div>
                {member.account_id !== team.leader && (
                  <div className="actions">
                    {permissions.can_administer &&
                      (leader || !member.can_administer) && (
                        <button
                          className="quiet"
                          onClick={() =>
                            open({
                              title: message(
                                "team.permissions_title",
                                member.name,
                                team.name,
                              ),
                              type: "team_permissions",
                              values: {
                                team: team.id,
                                member: member.account_id,
                                ...(!leader ? { administer: false } : {}),
                              },
                              teamScope: { id: team.id, permission: "admin" },
                              note: () => (
                                <p>
                                  {t("team.permissions_intro")}
                                  {!leader && (
                                    <> {t("team.leader_assign_admin")}</>
                                  )}
                                </p>
                              ),
                              fields: permissionFields(member, leader),
                              submit: message("text.save"),
                            })
                          }
                        >
                          {t("text.role")}
                        </button>
                      )}
                    {leader && (
                      <button
                        className="quiet"
                        onClick={() =>
                          open({
                            title: message("team.transfer_title", team.name),
                            type: "team_transfer",
                            values: {
                              team: team.id,
                              target: member.account_id,
                            },
                            teamScope: { id: team.id, permission: "leader" },
                            note: () => (
                              <p>
                                {t(
                                  "team.transfer_note",
                                  member.name,
                                  team.name,
                                )}
                              </p>
                            ),
                            submit: message("text.transfer_leadership_now"),
                          })
                        }
                      >
                        {t("text.transfer")}
                      </button>
                    )}
                  </div>
                )}
              </li>
            ))}
          </ul>
          {cursor && (
            <button
              className="quiet"
              disabled={loading}
              onClick={() => void loadMore()}
            >
              {t(loading ? "team.loading_members" : "team.load_members")}
            </button>
          )}
        </>
      )}
    </Card>
  );
}
