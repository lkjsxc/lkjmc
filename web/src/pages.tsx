import { useState } from "react";
import { useApp } from "./App";
import { jobTitle, api, date, money, type Data } from "./api";
import { t, message, translateError } from "./i18n";
import { ActionForm, Card, Empty, Status, type Field } from "./ui";
import { jobTarget } from "./jobs";

export function PageNavigation({ data }: { data: Data }) {
  const { route } = useApp();
  if (route.component !== "feed") return null;
  const base = "/play/" + route.section;
  const query = route.unread ? "unread=true&" : "";
  return (
    <nav className="pagination" aria-label={t("text.history_pages")}>
      {route.section === "notifications" && (
        <>
          <a
            href="#/play/notifications"
            aria-current={!route.unread ? "page" : undefined}
          >
            {t("text.all")}
          </a>
          <a
            href="#/play/notifications?unread=true"
            aria-current={route.unread ? "page" : undefined}
          >
            {t("text.unread")}
          </a>
        </>
      )}
      {route.cursor && (
        <a href={"#" + base + (route.unread ? "?unread=true" : "")}>
          {t("text.latest")}
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
          {t("text.older")}
        </a>
      )}
    </nav>
  );
}
export function ManagedList({ data }: { data: Data }) {
  const { me } = useApp();
  const r = me.account.rank;
  return (
    <>
      <div className="section-toolbar">
        <p>
          {t("text.server_allowance")}
          {r.server_count}
          {t("text.servers_running_at_once")}
          {r.concurrent_servers}
          {t("text.servers_a1bf4fae")}
        </p>
        <a className="button primary" href="#/hosting/servers/new">
          {t("text.create_a_server")}
        </a>
      </div>
      {(data.servers ?? []).length ? (
        <div className="server-list">
          {data.servers.map((s: Data) => (
            <article className="server-row" key={s.id}>
              <div className="grow">
                <h2>
                  <a href={"#/hosting/servers/" + s.id}>{s.name}</a>
                </h2>
                <p>
                  {s.software} {s.version} · {money(s.memory_mib)} MiB · CPU{" "}
                  {s.cpu_millis / 1000}
                </p>
                <small>
                  {s.last_observed_at
                    ? date(s.last_observed_at)
                    : t("text.not_observed_yet")}
                </small>
              </div>
              <Status value={s.observed} />
            </article>
          ))}
        </div>
      ) : (
        <Empty>
          {t(
            "text.no_servers_to_manage_create_one_to_manage_its_power_fil_4287b25dbd",
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
          "text.your_current_tier_does_not_allow_server_creation_ask_an_22df4fb669",
        )}
      </Empty>
    );
  const custom = preset === "custom" || !selected;
  const fields: Field[] = [
    { name: "name", label: message("text.name"), max: 64 },
    ...(custom
      ? [
          {
            name: "version",
            label: message("text.minecraft_version"),
            hint: message("text.match_this_to_the_jar_you_will_use"),
          },
        ]
      : []),
    {
      name: "memory_mib",
      label: message("text.memory_mib"),
      type: "number",
      min: 512,
      max: r.memory_mib,
      value: Math.min(2048, r.memory_mib),
    },
    {
      name: "cpu_millis",
      label: message("text.cpu_1_core_1000"),
      type: "number",
      min: 1000,
      step: 1000,
      max: r.cpu_millis,
      value: 1000,
    },
    {
      name: "storage_mib",
      label: message("text.storage_mib"),
      type: "number",
      min: minimumStorage,
      max: r.storage_mib,
      value: Math.min(Math.max(16384, minimumStorage), r.storage_mib),
    },
    {
      name: "visibility",
      label: message("text.visibility"),
      type: "select",
      options: [
        { value: "private", label: message("text.you_and_administrators") },
        { value: "invite", label: message("text.invited_players") },
        { value: "public", label: message("text.public") },
      ],
    },
  ];
  return (
    <Card title={t("text.create_a_server")}>
      <label className="field">
        {t("text.server_software")}
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
          <option value="custom">{t("text.custom_jar")}</option>
        </select>
      </label>
      <p>
        {custom
          ? t(
              "text.create_an_isolated_server_and_upload_your_own_jar_afterward",
            )
          : t(
              "text.the_selected_software_and_java_runtime_are_installed_au_d141a95789",
            )}
      </p>
      <p>
        {t("text.minimum_storage")}: {money(minimumStorage)} MiB
      </p>
      {r.storage_mib < minimumStorage ? (
        <p>
          {t(
            "text.your_storage_allowance_is_below_the_minimum_needed_to_c_c61c30dc1f",
          )}
        </p>
      ) : (
        <ActionForm
          key={custom ? "custom" : selected?.version}
          fields={fields}
          submit={t("text.create_server")}
          onSubmit={async (v) => {
            const result = await send("server_create", {
              community: null,
              ...v,
              software: custom ? "custom" : (selected?.software ?? "custom"),
              version: custom ? v.version : selected?.version,
            });
            if (result.server_id) go("/hosting/servers/" + result.server_id);
            else go("/hosting/servers");
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
  const { showJob } = useApp();
  jobs = jobs.filter(
    (j) =>
      !["server.logs", "server.files", "server.file.read"].includes(j.kind),
  );
  return jobs.length ? (
    <div className="list">
      {jobs.map((j) => (
        <div className="list-row" key={j.id}>
          <div className="grow">
            <strong>
              {jobTitle(j)}
              {jobTarget(j) ? " · " + jobTarget(j) : ""}
            </strong>
            <small>{date(j.updated_at ?? j.created_at)}</small>
            {j.error && <p className="error">{translateError(j.error)}</p>}
          </div>
          <Status value={j.state} />
          <button onClick={() => showJob(j.id, j)}>
            {t("text.view_details")}
          </button>
        </div>
      ))}
    </div>
  ) : (
    <Empty>{t("text.no_recent_actions")}</Empty>
  );
}
