import { t, message, messageError } from "./i18n";
import { useState } from "react";
import type { Data } from "./api";
import { useApp, PageBlock } from "./App";
import { Card, Empty, Icon } from "./ui";

export function Social({ data }: { data: Data }) {
  const { me, open, act, send, route, go, isWorking } = useApp();
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
        <PageBlock id={["party", "party-members"]}>
          {data.party ? (
            <section className="party-workspace" aria-label={data.party.name}>
              <div className="actions">
                {data.party.leader === me.account.id && (
                  <button onClick={() => invite("party", data.party.id)}>
                    {t("text.invite_member")}
                  </button>
                )}
                <a
                  className="button"
                  href={"#/timeline?room=" + data.party.room_id}
                >
                  {t("text.chat_room")}
                </a>
                {data.party.leader === me.account.id && (
                  <button
                    className="quiet"
                    onClick={() =>
                      open({
                        title: message("text.party_name"),
                        type: "party_rename",
                        values: { party: data.party.id },
                        fields: [
                          {
                            name: "name",
                            label: message("text.party_name"),
                            value: data.party.name,
                            max: 80,
                          },
                        ],
                        submit: message("text.save"),
                      })
                    }
                  >
                    {t("text.party_name")}
                  </button>
                )}
                <button
                  className="quiet"
                  disabled={isWorking("party_leave", { party: data.party.id })}
                  onClick={() => act("party_leave", { party: data.party.id })}
                >
                  {t("text.leave_party")}
                </button>
              </div>
              <ul className="team-member-list">
                {(data.party.members ?? []).map((m: Data) => (
                  <li className="team-member-row" key={m.account_id}>
                    <div className="grow">
                      <strong>{m.name}</strong>
                      {m.account_id === data.party.leader && (
                        <small>{t("text.leader_bc9cfa8a")}</small>
                      )}
                    </div>
                    {me.account.id === data.party.leader &&
                      m.account_id !== me.account.id && (
                        <button
                          className="quiet"
                          disabled={isWorking("party_transfer", {
                            party: data.party.id,
                            target: m.account_id,
                          })}
                          onClick={() =>
                            act("party_transfer", {
                              party: data.party.id,
                              target: m.account_id,
                            })
                          }
                        >
                          {t("text.make_leader")}
                        </button>
                      )}
                  </li>
                ))}
              </ul>
            </section>
          ) : (
            <button
              className="primary"
              disabled={isWorking("party_create")}
              onClick={() => act("party_create")}
            >
              {t("text.create_party")}
            </button>
          )}
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
