import { t, message, messageError } from "./i18n";
import { useState } from "react";
import type { Data } from "./api";
import { useApp, PageBlock } from "./App";
import { Card, Empty, Icon, type Field } from "./ui";

export function Social({ data }: { data: Data }) {
  const { me, open, act, send, route, go } = useApp();
  const [error, setError] = useState<unknown>(null);
  const friends = (data.friends ?? []).filter((f: Data) =>
    route.section === "incoming"
      ? f.state === "pending" && f.requester !== me.account.id
      : route.section === "outgoing"
        ? f.state === "pending" && f.requester === me.account.id
        : f.state === "accepted",
  );
  const invite = (kind: string, resource: string) =>
    open({
      title: message("text.send_an_invitation"),
      type: "invite",
      values: { kind, resource },
      fields: [
        {
          name: "target",
          label: message("text.invite_a_player"),
          type: "player",
        },
      ],
      submit: message("text.invite"),
    });
  return (
    <>
      {error && (
        <p role="alert" className="error">
          {messageError(error)}
        </p>
      )}
      <div className="grid two">
        <PageBlock id={["friends", "incoming", "outgoing"]}>
          <Card
            title={t("text.friends")}
            action={
              <button
                onClick={() =>
                  open({
                    title: message("text.friend_request"),
                    type: "friend_request",
                    fields: [
                      {
                        name: "target",
                        label: message("text.player"),
                        type: "player",
                      },
                    ],
                    submit: message("text.send_friend_request"),
                  })
                }
              >
                <Icon name="plus" />
                {t("text.add")}
              </button>
            }
          >
            {!friends.length ? (
              <Empty>
                {t("text.no_friends_yet_choose_add_to_find_a_player")}
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
                          ? t("text.online_in_game")
                          : t("text.friends")
                        : f.requester === me.account.id
                          ? t("text.awaiting_response")
                          : t("text.incoming_friend_request")}
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
                          {t("text.accept")}
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
                          {t("text.decline")}
                        </button>
                      </>
                    ) : f.state === "accepted" ? (
                      <button
                        onClick={() =>
                          void send("direct_room", { target: f.id })
                            .then((r) => {
                              go("/timeline?room=" + r.room_id);
                            })
                            .catch((e) => setError(e))
                        }
                      >
                        {t("text.message")}
                      </button>
                    ) : null}
                    <button
                      className="quiet"
                      aria-label={t("text.manage_friendship_with_0", f.name)}
                      onClick={() =>
                        open({
                          title: f.name,
                          fields: [
                            {
                              name: "action",
                              label: message("text.actions"),
                              type: "select",
                              options: [
                                {
                                  value: "remove",
                                  label: message(
                                    "text.remove_friend_or_request",
                                  ),
                                },
                                {
                                  value: "block",
                                  label: message("text.block_player"),
                                },
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
                      {t("text.settings")}
                    </button>
                  </div>
                </div>
              ))
            )}
          </Card>
        </PageBlock>
        <PageBlock id={["team", "team-members", "team-settings"]}>
          <Card title={data.team?.name ?? t("text.teams")}>
            <div className="group-section">
              {data.team ? (
                <>
                  <PageBlock id="team">
                    <p>
                      {t("text.share_land_coins_and_buildings_with_your_team")}
                    </p>
                    <a href={"#/timeline?room=" + data.team.room_id}>
                      {t("text.team_chat")}
                    </a>
                  </PageBlock>
                  <PageBlock id="team-members">
                    {(me.account.id === data.team.leader ||
                      data.team.members?.find(
                        (m: Data) => m.account_id === me.account.id,
                      )?.can_manage_members) && (
                      <button onClick={() => invite("team", data.team.id)}>
                        {t("text.invite_member")}
                      </button>
                    )}
                    {data.team.members?.map((m: Data) => (
                      <div className="list-row" key={m.account_id}>
                        <div>
                          <strong>{m.name}</strong>
                          {m.account_id === data.team.leader && (
                            <small>{t("text.leader")}</small>
                          )}
                        </div>
                        {me.account.id === data.team.leader &&
                          m.account_id !== data.team.leader && (
                            <div className="actions">
                              <button
                                className="quiet"
                                onClick={() =>
                                  open({
                                    title: message(
                                      "text.permissions_for_0",
                                      m.name,
                                    ),
                                    type: "team_permissions",
                                    values: {
                                      team: data.team.id,
                                      member: m.account_id,
                                    },
                                    fields: [
                                      [
                                        "build",
                                        message("text.build"),
                                        m.can_build,
                                      ],
                                      [
                                        "sell",
                                        message("text.sell"),
                                        m.can_sell,
                                      ],
                                      [
                                        "spend",
                                        message("text.spend_shared_coins"),
                                        m.can_spend,
                                      ],
                                      [
                                        "members",
                                        message("text.manage_members"),
                                        m.can_manage_members,
                                      ],
                                      [
                                        "administer",
                                        message("text.manage_team"),
                                        m.can_administer,
                                      ],
                                    ].map(([name, label, value]) => ({
                                      name: String(name),
                                      label: label as Field["label"],
                                      type: "checkbox",
                                      value: Boolean(value),
                                    })),
                                  })
                                }
                              >
                                {t("text.role")}
                              </button>
                              <button
                                className="quiet"
                                onClick={() =>
                                  open({
                                    title: message("text.transfer_leadership"),
                                    type: "team_transfer",
                                    values: {
                                      team: data.team.id,
                                      target: m.account_id,
                                    },
                                    note: () => (
                                      <p>
                                        {m.name}
                                        {t(
                                          "text.will_become_your_team_s_leader",
                                        )}
                                      </p>
                                    ),
                                    submit: message(
                                      "text.transfer_leadership_now",
                                    ),
                                  })
                                }
                              >
                                {t("text.transfer")}
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
                            title: message("text.leave_team"),
                            type: "team_leave",
                            note: () => (
                              <p>
                                {t(
                                  "text.land_and_shared_assets_stay_with_the_team_transfer_lead_2f8f3742f8",
                                )}
                              </p>
                            ),
                            submit: message("text.leave_team"),
                          })
                        }
                      >
                        {t("text.leave_team")}
                      </button>
                      {me.account.id === data.team.leader && (
                        <button
                          className="quiet danger"
                          onClick={() =>
                            open({
                              title: message("text.disband_team"),
                              type: "team_disband",
                              values: { team: data.team.id },
                              note: () => (
                                <p>
                                  {t(
                                    "text.disband_a_team_after_disposing_of_its_land_stored_asset_3fd66c1a99",
                                  )}
                                </p>
                              ),
                              submit: message("text.confirm_disbanding"),
                            })
                          }
                        >
                          {t("text.disband")}
                        </button>
                      )}
                    </div>
                  </PageBlock>
                </>
              ) : (
                <>
                  <p>
                    {t(
                      "text.you_can_belong_to_one_team_the_team_s_own_achievements_2d42a440be",
                    )}
                  </p>
                  <button
                    onClick={() =>
                      open({
                        title: message("text.create_team"),
                        type: "team_create",
                        fields: [
                          {
                            name: "name",
                            label: message("text.team_name"),
                            max: 64,
                          },
                        ],
                        submit: message("text.create_team"),
                      })
                    }
                  >
                    {t("text.create_team")}
                  </button>
                </>
              )}
            </div>
          </Card>
        </PageBlock>
        <PageBlock id={["party", "party-members", "party-ready"]}>
          <Card title={t("text.parties")}>
            <div className="group-section">
              <span className="eyebrow">{t("text.party")}</span>
              {data.party ? (
                <>
                  <h3>{data.party.name}</h3>
                  <div className="actions">
                    <button onClick={() => invite("party", data.party.id)}>
                      {t("text.invite_member")}
                    </button>
                    <button
                      onClick={() => go("/timeline?room=" + data.party.room_id)}
                    >
                      {t("text.chat_room")}
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
                        ? t("text.cancel_ready_status")
                        : t("text.ready")}
                    </button>
                    <button
                      className="quiet"
                      onClick={() => act("party_leave")}
                    >
                      {t("text.leave")}
                    </button>
                  </div>
                  <PageBlock id={["party-members", "party-ready"]}>
                    {" "}
                    {data.party.members?.map((m: Data) => (
                      <p key={m.account_id}>
                        {m.ready ? "✓" : "○"} {m.name}
                        {m.account_id === data.party.leader
                          ? t("text.leader_bc9cfa8a")
                          : ""}
                        {me.account.id === data.party.leader &&
                          m.account_id !== me.account.id && (
                            <button
                              className="quiet"
                              onClick={() =>
                                act("party_transfer", { target: m.account_id })
                              }
                            >
                              {t("text.make_leader")}
                            </button>
                          )}
                      </p>
                    ))}
                  </PageBlock>
                </>
              ) : (
                <>
                  <p>
                    {t("text.a_temporary_group_for_adventures_and_meeting_up")}
                  </p>
                  <button
                    onClick={() =>
                      open({
                        title: message("text.create_party"),
                        type: "party_create",
                        fields: [
                          {
                            name: "name",
                            label: message("text.party_name"),
                            max: 80,
                          },
                        ],
                        submit: message("text.create_party"),
                      })
                    }
                  >
                    {t("text.create_party")}
                  </button>
                </>
              )}
            </div>
          </Card>
        </PageBlock>
      </div>
      <PageBlock id="communities">
        <Card
          title={t("text.server_communities")}
          action={
            <button
              onClick={() =>
                open({
                  title: message("text.create_community"),
                  type: "community_create",
                  fields: [
                    {
                      name: "name",
                      label: message("text.community_name"),
                      max: 64,
                    },
                  ],
                  submit: message("text.create"),
                })
              }
            >
              {t("text.create")}
            </button>
          }
        >
          <p>
            {t(
              "text.manage_servers_together_each_server_owner_s_tier_determ_bbce52e70a",
            )}
          </p>
          {data.communities?.map((c: Data) => (
            <div className="list-row" key={c.id}>
              <strong>{c.name}</strong>
              <button onClick={() => invite("community", c.id)}>
                {t("text.invite_members")}
              </button>
            </div>
          ))}
        </Card>
      </PageBlock>
    </>
  );
}
