import { t } from "./i18n";
import { useState } from "react";
import type { Data } from "./api";
import { useApp, PageBlock } from "./App";
import { Card, Empty, Icon } from "./ui";

export function Social({ data }: { data: Data }) {
  const { me, open, act, send, route, go } = useApp();
  const [error, setError] = useState("");
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
      {error && (
        <p role="alert" className="error">
          {error}
        </p>
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
                        {t("Message")}
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
          <Card title={data.team?.name ?? t("Teams")}>
            <div className="group-section">
              {data.team ? (
                <>
                  <PageBlock id="team">
                    <p>
                      {t("Share land, coins, and buildings with your team.")}
                    </p>
                    <a href={"#/timeline?room=" + data.team.room_id}>
                      {t("Team chat")}
                    </a>
                  </PageBlock>
                  <PageBlock id="team-members">
                    {(me.account.id === data.team.leader ||
                      data.team.members?.find(
                        (m: Data) => m.account_id === me.account.id,
                      )?.can_manage_members) && (
                      <button onClick={() => invite("team", data.team.id)}>
                        {t("Invite member")}
                      </button>
                    )}
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
