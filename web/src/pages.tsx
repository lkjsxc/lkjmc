import { useState, type FormEvent } from "react";
import { useApp } from "./App";
import { jobTitle, api, date, money, type Data } from "./api";
import { t, messageError, renderSystemMessage, translateError } from "./i18n";
import { Card, Empty, Status } from "./ui";
import { jobTarget } from "./jobs";
import { HostingRuntimeBadge } from "./hostingStatus";

export function PageNavigation({ data }: { data: Data }) {
  const { route } = useApp();
  const expeditionJournal =
    route.area === "expeditions" && route.section === "journal";
  if (route.component !== "feed" && !expeditionJournal) return null;
  const base = expeditionJournal
    ? "/expeditions/journal"
    : "/play/" + route.section;
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
  const hosting = data.hosting;
  const servers: Data[] = data.servers ?? [];
  const canCreate = hosting?.can_create === true;
  return (
    <section className="hosting-collection">
      <div className="hosting-collection-toolbar">
        <HostingCapacity hosting={hosting} />
        {canCreate ? (
          <a className="button primary" href="#/hosting/servers/new">
            {t("text.create_a_server")}
          </a>
        ) : (
          <button disabled aria-describedby="hosting-creation-reason">
            {t("text.create_a_server")}
          </button>
        )}
      </div>
      {!canCreate && (
        <p className="hosting-eligibility" id="hosting-creation-reason">
          {creationReason(hosting)}
        </p>
      )}
      {servers.length ? (
        <table className="hosting-inventory">
          <thead>
            <tr>
              <th scope="col">{t("hosting.inventory_name")}</th>
              <th scope="col">{t("hosting.inventory_runtime")}</th>
              <th scope="col">{t("hosting.inventory_software")}</th>
              <th scope="col">{t("hosting.inventory_allocation")}</th>
              <th scope="col">{t("hosting.inventory_observed")}</th>
            </tr>
          </thead>
          <tbody>
            {servers.map((server) => (
              <tr className="server-row" key={server.id}>
                <th scope="row">
                  <a href={"#/hosting/servers/" + server.id}>{server.name}</a>
                </th>
                <td className="inventory-runtime">
                  <HostingRuntimeBadge server={server} />
                </td>
                <td className="inventory-software">
                  {server.software} {server.version}
                </td>
                <td className="inventory-allocation">
                  <span>
                    <small>{t("text.memory")}</small>
                    {allocated(server.memory_mib, 1024, "GiB")}
                  </span>
                  <span>
                    <small>{t("text.cpu")}</small>
                    {allocated(server.cpu_millis, 1000, "vCPU")}
                  </span>
                  <span>
                    <small>{t("text.storage")}</small>
                    {allocated(server.storage_mib, 1024, "GiB")}
                  </span>
                </td>
                <td className="inventory-observed">
                  {server.last_observed_at ? (
                    <time dateTime={server.last_observed_at}>
                      {date(server.last_observed_at)}
                    </time>
                  ) : (
                    t("text.not_observed_yet")
                  )}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      ) : (
        <div className="hosting-empty">
          <h2>{t("hosting.inventory_empty_title")}</h2>
          <p>
            {canCreate
              ? t("hosting.inventory_empty_detail")
              : t("hosting.inventory_access_detail")}
          </p>
        </div>
      )}
    </section>
  );
}
function allocated(value: unknown, divisor: number, unit: string) {
  return typeof value === "number" && Number.isFinite(value)
    ? `${money(value / divisor)} ${unit}`
    : "—";
}
function creationReason(hosting?: Data) {
  return hosting?.creation_blocked_reason
    ? renderSystemMessage(hosting.creation_blocked_reason)
    : t("hosting.creation_not_allowed");
}
function HostingCapacity({ hosting }: { hosting?: Data }) {
  if (!hosting) return null;
  const limits = hosting.limits ?? {};
  const owned = hosting.owned ?? {};
  const reserved = hosting.reserved ?? {};
  const facts = [
    [
      "hosting.capacity_servers",
      money(owned.server_count),
      money(limits.server_count),
    ],
    [
      "hosting.capacity_storage",
      allocated(owned.storage_mib, 1024, "GiB"),
      allocated(limits.storage_mib, 1024, "GiB"),
    ],
    [
      "hosting.capacity_running",
      money(reserved.server_count),
      money(limits.concurrent_servers),
    ],
    [
      "hosting.capacity_memory",
      allocated(reserved.memory_mib, 1024, "GiB"),
      allocated(limits.memory_mib, 1024, "GiB"),
    ],
    [
      "hosting.capacity_cpu",
      allocated(reserved.cpu_millis, 1000, "vCPU"),
      allocated(limits.cpu_millis, 1000, "vCPU"),
    ],
  ];
  return (
    <dl className="hosting-capacity" aria-label={t("text.server_allowance")}>
      {facts.map(([label, used, limit]) => (
        <div key={label}>
          <dt>{t(label)}</dt>
          <dd>{t("hosting.capacity_used_limit", { used, limit })}</dd>
        </div>
      ))}
    </dl>
  );
}
export function CreateServer({ data }: { data: Data }) {
  const { send, go } = useApp();
  const hosting: Data | undefined = data.hosting;
  const limits = hosting?.limits ?? {};
  const minimumMemory = Number(hosting?.minimum_server_memory_mib ?? 0) / 1024;
  const minimumCPU = Number(hosting?.minimum_server_cpu_millis ?? 0) / 1000;
  const cpuStep = Number(hosting?.server_cpu_step_millis ?? 0) / 1000;
  const minimumStorage =
    Number(hosting?.minimum_server_storage_mib ?? 0) / 1024;
  const maximumMemory = Number(limits.memory_mib ?? 0) / 1024;
  const maximumCPU = Number(limits.cpu_millis ?? 0) / 1000;
  const maximumStorage = Number(hosting?.remaining?.storage_mib ?? 0) / 1024;
  const [preset, setPreset] = useState("");
  const [draft, setDraft] = useState(() => ({
    name: "",
    version: "",
    memory: String(Math.min(2, maximumMemory)),
    cpu: String(minimumCPU),
    storage: String(minimumStorage),
    visibility: "private",
  }));
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<unknown>(null);
  const choices: Data[] = data.presets ?? [];
  const selected =
    choices.find((p) => p.software + "/" + p.version === preset) ?? choices[0];
  const custom = preset === "custom" || !selected;
  const canCreate = hosting?.can_create === true;
  const update = (field: keyof typeof draft, value: string) =>
    setDraft((old) => ({ ...old, [field]: value }));
  const access = [
    ["private", "text.you_and_administrators"],
    ["invite", "text.invited_players"],
    ["public", "text.public"],
  ];
  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (busy || !canCreate) return;
    setBusy(true);
    setError(null);
    try {
      const result = await send("server_create", {
        community: null,
        name: draft.name,
        software: custom ? "custom" : selected.software,
        version: custom ? draft.version : selected.version,
        memory_mib: Number(draft.memory) * 1024,
        cpu_millis: Number(draft.cpu) * 1000,
        storage_mib: Number(draft.storage) * 1024,
        visibility: draft.visibility,
      });
      go(
        result.server_id
          ? "/hosting/servers/" + result.server_id
          : "/hosting/servers",
      );
    } catch (cause) {
      setError(cause);
    } finally {
      setBusy(false);
    }
  }
  return (
    <div className="hosting-create">
      <HostingCapacity hosting={hosting} />
      <p className="hosting-capacity-note">{t("hosting.capacity_note")}</p>
      {!canCreate && (
        <p className="hosting-eligibility" role="status">
          {creationReason(hosting)}
        </p>
      )}
      <form
        className="hosting-create-form"
        onSubmit={(event) => void submit(event)}
      >
        <fieldset disabled={busy || !canCreate}>
          <section className="hosting-form-section">
            <h2>{t("hosting.create_identity")}</h2>
            <label className="field">
              {t("text.name")}
              <input
                required
                name="name"
                maxLength={64}
                value={draft.name}
                onChange={(event) => update("name", event.target.value)}
              />
            </label>
          </section>
          <section className="hosting-form-section">
            <h2>{t("hosting.create_software")}</h2>
            <label className="field">
              {t("text.server_software")}
              <select
                value={
                  custom ? "custom" : selected.software + "/" + selected.version
                }
                onChange={(event) => setPreset(event.target.value)}
              >
                {choices.map((choice) => (
                  <option
                    key={choice.software + "/" + choice.version}
                    value={choice.software + "/" + choice.version}
                  >
                    {choice.software} {choice.version} · Java {choice.java}
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
            {custom && (
              <label className="field">
                {t("hosting.custom_version")}
                <input
                  required
                  name="version"
                  maxLength={32}
                  value={draft.version}
                  onChange={(event) => update("version", event.target.value)}
                />
                <small>{t("text.match_this_to_the_jar_you_will_use")}</small>
              </label>
            )}
          </section>
          <section className="hosting-form-section">
            <h2>{t("hosting.create_resources")}</h2>
            <div className="hosting-resource-fields">
              <label className="field">
                {t("hosting.memory_gib")}
                <input
                  required
                  type="number"
                  name="memory"
                  min={minimumMemory}
                  max={maximumMemory}
                  step={1 / 1024}
                  value={draft.memory}
                  onChange={(event) => update("memory", event.target.value)}
                />
                <small>
                  {t("hosting.resource_max", {
                    amount: `${money(maximumMemory)} GiB`,
                  })}
                </small>
              </label>
              <label className="field">
                {t("hosting.cpu_vcpu")}
                <input
                  required
                  type="number"
                  name="cpu"
                  min={minimumCPU}
                  max={maximumCPU}
                  step={cpuStep || 1}
                  value={draft.cpu}
                  onChange={(event) => update("cpu", event.target.value)}
                />
                <small>
                  {t("hosting.resource_max", {
                    amount: `${money(maximumCPU)} vCPU`,
                  })}
                </small>
              </label>
              <label className="field">
                {t("hosting.storage_gib")}
                <input
                  required
                  type="number"
                  name="storage"
                  min={minimumStorage}
                  max={maximumStorage}
                  step={1 / 1024}
                  value={draft.storage}
                  onChange={(event) => update("storage", event.target.value)}
                />
                <small>
                  {t("hosting.storage_minimum", {
                    amount: `${money(minimumStorage)} GiB`,
                  })}{" "}
                  ·{" "}
                  {t("hosting.resource_max", {
                    amount: `${money(maximumStorage)} GiB`,
                  })}
                </small>
              </label>
            </div>
          </section>
          <section className="hosting-form-section">
            <h2>{t("hosting.create_access")}</h2>
            <label className="field">
              {t("text.visibility")}
              <select
                value={draft.visibility}
                onChange={(event) => update("visibility", event.target.value)}
              >
                {access.map(([value, label]) => (
                  <option key={value} value={value}>
                    {t(label)}
                  </option>
                ))}
              </select>
            </label>
          </section>
        </fieldset>
        <aside
          className="hosting-create-summary"
          aria-label={t("hosting.create_summary")}
        >
          <h2>{t("hosting.create_summary")}</h2>
          <dl className="compact-details">
            <div>
              <dt>{t("text.name")}</dt>
              <dd>{draft.name || "—"}</dd>
            </div>
            <div>
              <dt>{t("text.server_software")}</dt>
              <dd>
                {custom
                  ? t("text.custom_jar")
                  : `${selected.software} ${selected.version}`}
              </dd>
            </div>
            <div>
              <dt>{t("text.memory")}</dt>
              <dd>
                {draft.memory ? `${money(Number(draft.memory))} GiB` : "—"}
              </dd>
            </div>
            <div>
              <dt>{t("text.cpu")}</dt>
              <dd>{draft.cpu ? `${money(Number(draft.cpu))} vCPU` : "—"}</dd>
            </div>
            <div>
              <dt>{t("text.storage")}</dt>
              <dd>
                {draft.storage ? `${money(Number(draft.storage))} GiB` : "—"}
              </dd>
            </div>
            <div>
              <dt>{t("text.visibility")}</dt>
              <dd>
                {t(access.find(([value]) => value === draft.visibility)![1])}
              </dd>
            </div>
          </dl>
          <button
            className="primary"
            disabled={busy || !canCreate}
            type="submit"
          >
            {busy ? t("hosting.create_loading") : t("text.create_server")}
          </button>
          {!!error && (
            <p className="error" role="alert">
              {messageError(error)}
            </p>
          )}
        </aside>
      </form>
    </div>
  );
}
export function AdminHome({ data }: { data: Data }) {
  return (
    <div className="overview-links">
      {[
        ["reports", "text.reports"],
        ["ranks", "text.hosting_access_tiers"],
        ["backups", "text.official_backups"],
        ["operations", "text.operations"],
        ["audit", "text.audit_log"],
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
