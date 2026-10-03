import { useState } from "react";
import { useApp } from "./App";
import { jobTitle, api, date, money, type Data } from "./api";
import { t, translateError } from "./i18n";
import { ActionForm, Card, Empty, Status, type Field } from "./ui";
import { Smp } from "./views";

export function PageNavigation({ data }: { data: Data }) {
  const { route } = useApp();
  if (route.component !== "feed") return null;
  const base = "/home/" + route.section;
  const query = route.unread ? "unread=true&" : "";
  return (
    <nav className="pagination" aria-label={t("History pages")}>
      {route.section === "notifications" && (
        <>
          <a
            href="#/home/notifications"
            aria-current={!route.unread ? "page" : undefined}
          >
            {t("All")}
          </a>
          <a
            href="#/home/notifications?unread=true"
            aria-current={route.unread ? "page" : undefined}
          >
            {t("Unread")}
          </a>
        </>
      )}
      {route.cursor && (
        <a href={"#" + base + (route.unread ? "?unread=true" : "")}>
          {t("Latest")}
        </a>
      )}
      {data.next_cursor && (
        <a
          href={
            "#" +
            base +
            "?" +
            query +
            "cursor=" +
            encodeURIComponent(data.next_cursor)
          }
        >
          {t("Older")}
        </a>
      )}
    </nav>
  );
}
export function ServerInfo({ data }: { data: Data }) {
  const { me } = useApp();
  const s = data.server;
  if (!s) return <Empty>{t("This page could not be found.")}</Empty>;
  return (
    <>
      <Card title={s.name} action={<Status value={s.observed} />}>
        <dl className="details-list">
          <div>
            <dt>{t("Server software")}</dt>
            <dd>
              {s.software} {s.version}
            </dd>
          </div>
          <div>
            <dt>{t("Players online")}</dt>
            <dd>{money(s.players)}</dd>
          </div>
          <div>
            <dt>{t("Connection")}</dt>
            <dd>
              {me.game_address} ·{" "}
              {s.capabilities?.bedrock ? "Java / Bedrock" : "Java"}
            </dd>
          </div>
          <div>
            <dt>{t("Last checked")}</dt>
            <dd>
              {s.last_observed_at
                ? date(s.last_observed_at)
                : t("Not observed yet")}
            </dd>
          </div>
        </dl>
        {s.can_manage && (
          <a className="button" href={"#/manage/servers/" + s.id}>
            {t("Manage this server")}
          </a>
        )}
        {s.capabilities?.client_mods && (
          <p className="notice">
            {t("The specified client mods are required.")}
          </p>
        )}
      </Card>
      {s.kind === "official" ? (
        <Smp data={{ servers: [s] }} />
      ) : (
        <JoinButton server={s} />
      )}
    </>
  );
}
export function JoinButton({ server: s }: { server: Data }) {
  const { act } = useApp();
  return (
    <button
      className="primary"
      disabled={!s.capabilities?.proxy_join || s.maintenance}
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
  );
}
export function ManagedList({ data }: { data: Data }) {
  const { me } = useApp();
  const r = me.account.rank;
  return (
    <>
      <div className="section-toolbar">
        <p>
          {t("Server allowance")}
          {r.server_count}
          {t(" servers · Running at once ")}
          {r.concurrent_servers}
          {t(" servers")}
        </p>
        <a className="button primary" href="#/manage/servers/new">
          {t("Create a server")}
        </a>
      </div>
      {(data.servers ?? []).length ? (
        <div className="server-list">
          {data.servers.map((s: Data) => (
            <article className="server-row" key={s.id}>
              <div className="grow">
                <h2>
                  <a href={"#/manage/servers/" + s.id}>{s.name}</a>
                </h2>
                <p>
                  {s.software} {s.version} · {money(s.memory_mib)} MiB · CPU{" "}
                  {s.cpu_millis / 1000}
                </p>
                <small>
                  {s.last_observed_at
                    ? date(s.last_observed_at)
                    : t("Not observed yet")}
                </small>
              </div>
              <Status value={s.observed} />
              <a className="button" href={"#/manage/servers/" + s.id}>
                {t("Details")}
              </a>
            </article>
          ))}
        </div>
      ) : (
        <Empty>
          {t(
            "No servers to manage. Create one to manage its power, files, and backups here.",
          )}
        </Empty>
      )}
    </>
  );
}
export function CreateServer({ data }: { data: Data }) {
  const { me, send, go } = useApp();
  const r = me.account.rank;
  const minimumStorage = Number(data.minimum_storage_mib ?? 16384);
  const [preset, setPreset] = useState("");
  const choices: Data[] = data.presets ?? [];
  const selected =
    choices.find((p) => p.software + "/" + p.version === preset) ?? choices[0];
  if (!r.server_count)
    return (
      <Empty>
        {t(
          "Your current tier does not allow server creation. Ask an administrator to approve a tier for your intended setup.",
        )}
      </Empty>
    );
  const custom = preset === "custom" || !selected;
  const fields: Field[] = [
    { name: "name", label: t("Name"), max: 64 },
    ...(custom
      ? [
          {
            name: "version",
            label: t("Minecraft version"),
            hint: t("Match this to the JAR you will use."),
          },
        ]
      : []),
    {
      name: "memory_mib",
      label: t("Memory (MiB)"),
      type: "number",
      min: 512,
      max: r.memory_mib,
      value: Math.min(2048, r.memory_mib),
    },
    {
      name: "cpu_millis",
      label: t("CPU (1 core = 1000)"),
      type: "number",
      min: 1000,
      step: 1000,
      max: r.cpu_millis,
      value: 1000,
    },
    {
      name: "storage_mib",
      label: t("Storage (MiB)"),
      type: "number",
      min: minimumStorage,
      max: r.storage_mib,
      value: Math.min(Math.max(16384, minimumStorage), r.storage_mib),
    },
    {
      name: "visibility",
      label: t("Visibility"),
      type: "select",
      options: [
        { value: "private", label: t("You and administrators") },
        { value: "invite", label: t("Invited players") },
        { value: "public", label: t("Public") },
      ],
    },
  ];
  return (
    <Card title={t("Create a server")}>
      <label className="field">
        {t("Server software")}
        <select
          value={
            custom
              ? "custom"
              : selected
                ? selected.software + "/" + selected.version
                : "custom"
          }
          onChange={(e) => setPreset(e.target.value)}
        >
          {choices.map((p) => (
            <option
              key={p.software + "/" + p.version}
              value={p.software + "/" + p.version}
            >
              {p.software} {p.version} · Java {p.java}
            </option>
          ))}
          <option value="custom">{t("Custom JAR")}</option>
        </select>
      </label>
      <p>
        {custom
          ? t("Create an isolated server and upload your own JAR afterward.")
          : t(
              "The selected software and Java runtime are installed automatically.",
            )}
      </p>
      <p>
        {t("Minimum storage")}: {money(minimumStorage)} MiB
      </p>
      {r.storage_mib < minimumStorage ? (
        <p>
          {t(
            "Your storage allowance is below the minimum needed to create a server.",
          )}
        </p>
      ) : (
        <ActionForm
          key={custom ? "custom" : selected?.version}
          fields={fields}
          submit={t("Create server")}
          onSubmit={async (v) => {
            const result = await send("server_create", {
              community: null,
              ...v,
              software: custom ? "custom" : (selected?.software ?? "custom"),
              version: custom ? v.version : selected?.version,
            });
            if (result.server_id) go("/manage/servers/" + result.server_id);
            else go("/manage/servers");
          }}
        />
      )}
    </Card>
  );
}
export function AdminHome({ data }: { data: Data }) {
  return (
    <div className="overview-links">
      {[
        ["reports", "Reports"],
        ["ranks", "Hosting access tiers"],
        ["backups", "Official backups"],
        ["jobs", "Actions needing attention"],
        ["audit", "Audit log"],
      ].map(([key, label]) => (
        <a className="card overview-link" key={key} href={"#/admin/" + key}>
          <strong>{t(label)}</strong>
          {data.counts?.[key] != null && <span>{money(data.counts[key])}</span>}
        </a>
      ))}
    </div>
  );
}
export function JobList({ jobs = [] }: { jobs?: Data[] }) {
  return jobs.length ? (
    <div className="list">
      {jobs.map((j) => (
        <div className="list-row" key={j.id}>
          <div className="grow">
            <strong>{jobTitle(j)}</strong>
            <small>{date(j.updated_at ?? j.created_at)}</small>
            {j.error && <p className="error">{translateError(j.error)}</p>}
            {j.result?.lines && <pre>{j.result.lines.join("\n")}</pre>}
          </div>
          <Status value={j.state} />
        </div>
      ))}
    </div>
  ) : (
    <Empty>{t("No recent actions.")}</Empty>
  );
}
