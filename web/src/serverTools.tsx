import { PrivateCache, onResourceReset } from "./identity";
import { useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import { api, ApiError, date, jobTitle, money, type Data } from "./api";
import { useApp } from "./App";
import {
  message,
  messageError,
  renderSystemMessage,
  t,
  translateError,
  type SystemMessage,
} from "./i18n";
import { ActionForm, Card, Empty, Status } from "./ui";
import { clearServerReads, useServerRead, waitForJob } from "./serverReads";
import {
  hostingActionReasons,
  hostingStatus,
  type HostingStatus,
} from "./hostingStatus";

const stopped = (s: Data) =>
  s.observed === "stopped" &&
  s.desired === "stopped" &&
  (!s.maintenance || s.inspection?.state === "ready");
const readAvailable = (s: Data) =>
  s.can_manage && !["unprovisioned", "provisioning"].includes(s.observed);
const consoleDrafts = new PrivateCache<string>();
const fileDrafts = new PrivateCache<{ text: string; sha: string | null }>(16);
onResourceReset((id) => {
  if (!id.endsWith("/files")) consoleDrafts.delete(id);
  id = id.replace(/\/files$/, "");
  for (const key of fileDrafts.keys())
    if (key.startsWith(id + "/")) fileDrafts.delete(key);
});
const todayUTC = () => new Date().toISOString().slice(0, 10);
function useLifetime(blocked = false) {
  const controller = useMemo(() => new AbortController(), [blocked]);
  useLayoutEffect(() => {
    if (blocked) controller.abort();
    return () => controller.abort();
  }, [controller, blocked]);
  return controller.signal;
}
export function ServerTools({ data }: { data: Data }) {
  const { route, open, act, send } = useApp();
  const s = { ...data.server, ...data.servers?.[0] };
  useEffect(() => {
    if (!s.can_manage || !s.can_administer) {
      clearServerReads(s.can_manage ? s.id + "/files" : s.id);
      for (const key of fileDrafts.keys())
        if (key.startsWith(s.id + "/")) fileDrafts.delete(key);
      if (!s.can_manage) consoleDrafts.delete(s.id);
    }
  }, [s.id, s.can_manage, s.can_administer]);
  if (!s.id) return <Empty>{t("This page could not be found.")}</Empty>;
  if (!s.can_manage)
    return (
      <p role="alert">
        {t("You no longer have permission to manage this server.")}
      </p>
    );
  const section = route.section;
  const admin = s.can_administer;
  if (
    [
      "manage-files",
      "manage-backups",
      "manage-members",
      "manage-settings",
    ].includes(section) &&
    !admin
  )
    return (
      <p role="alert">
        {t("Administrator permission is required for this page.")}
      </p>
    );
  return (
    <>
      {s.error && (
        <p className="error" role="alert">
          {translateError(s.error)}
        </p>
      )}
      {section === "manage-overview" && <HostingOverview server={s} />}
      {section === "manage-console" && <Console key={s.id} server={s} />}
      {section === "manage-logs" && <Logs key={s.id} server={s} />}
      {section === "manage-files" &&
        (s.kind === "custom" ? (
          <Files key={s.id} server={s} />
        ) : (
          <Empty>
            {t("File tools are available for custom servers only.")}
          </Empty>
        ))}
      {section === "manage-settings" && (
        <Card title={s.name}>
          <ActionForm
            fields={[
              { name: "name", label: t("Name"), value: s.name, max: 64 },
              {
                name: "visibility",
                label: t("Visibility"),
                type: "select",
                value: s.visibility,
                options: [
                  { value: "private", label: t("You and administrators") },
                  { value: "invite", label: t("Invited players") },
                  { value: "public", label: t("Public") },
                ],
              },
            ]}
            onSubmit={(v) => send("server_configure", { id: s.id, ...v })}
          />
        </Card>
      )}
      {section === "manage-members" && <Members server={s} />}
      {section === "manage-backups" && (
        <Card title={t("Backups")}>
          <p>
            {t(
              "Restoring replaces the current world with the backup. Stop the server first.",
            )}
          </p>
          {s.kind === "custom" ? (
            <button
              disabled={
                s.maintenance || !["running", "stopped"].includes(s.observed)
              }
              onClick={() => act("server_backup", { id: s.id })}
            >
              {t("Create backup")}
            </button>
          ) : (
            <p>{t("Use Administration to manage official backups.")}</p>
          )}
          {!s.backups?.length && <Empty>{t("No backups yet.")}</Empty>}
          {s.backups?.map((b: Data) => (
            <div className="list-row" key={b.id}>
              <div className="grow">
                <strong>{date(b.created_at)}</strong>
              </div>
              <Status value={b.state} />
              <button
                disabled={
                  b.state !== "ready" || !stopped(s) || s.kind !== "custom"
                }
                onClick={() =>
                  open({
                    title: message("Restore a backup"),
                    type: "server_restore",
                    values: { id: s.id, backup: b.id },
                    note: () => (
                      <p>
                        {t(
                          "Restore “{0}” to {1}? Back up the current world first if you want to keep it.",
                          s.name,
                          date(b.created_at),
                        )}
                      </p>
                    ),
                    submit: message("Restore to this point"),
                  })
                }
              >
                {t("Restore")}
              </button>
            </div>
          ))}
        </Card>
      )}
    </>
  );
}
function runtimeHeading(status: HostingStatus) {
  switch (status.game_state) {
    case "running":
      return t("Minecraft is running");
    case "stopped":
      return t("Minecraft is stopped");
    case "starting":
      return t("Minecraft is starting");
    case "stopping":
      return t("Minecraft is stopping");
    case "unprovisioned":
      return t("Creating your server");
    case "error":
      return t("Your server needs attention");
    default:
      return t("Checking your server");
  }
}
function runtimeDescription(status: HostingStatus, server: Data) {
  if (!status.observation_fresh && status.game_state !== "unprovisioned")
    return t(
      "A current Minecraft observation is needed before power can change.",
    );
  if (server.inspection)
    return t(
      "File access can keep the host awake while Minecraft stays stopped.",
    );
  if (status.joinable) return t("Players can connect now.");
  switch (status.game_state) {
    case "running":
      return t("New connections are paused while server work finishes.");
    case "stopped":
      return t("Start Minecraft when you are ready to play.");
    case "starting":
      return t("Players can connect once Minecraft finishes starting.");
    case "stopping":
      return t("Player data is being saved before the server stops.");
    case "unprovisioned":
      return t("The host and Minecraft are being prepared.");
    case "error":
      return t(
        "Review the server error and current operation before trying again.",
      );
    default:
      return t("Power controls will be available after the server is checked.");
  }
}
function HostingOverview({ server: s }: { server: Data }) {
  const { act, open, me, isWorking, jobs, showJob } = useApp();
  const status = hostingStatus(s);
  const base = `#/hosting/servers/${s.id}`;
  const localOperation = jobs.find(
    (job) =>
      job.server_id === s.id &&
      !["succeeded", "failed", "cancelled"].includes(job.state) &&
      !["server.logs", "server.files", "server.file.read"].includes(
        String(job.kind).replaceAll("_", "."),
      ),
  );
  const recentOperation =
    s.operation_status &&
    (s.operation_status.outcome === "delivery_unknown" ||
      s.operation_status.state === "failed")
      ? s.operation_status
      : null;
  const operation = s.active_operation ?? localOperation ?? recentOperation;
  const submittedPower =
    isWorking("server_start", { id: s.id }) ||
    isWorking("server_stop", { id: s.id });
  const actionable =
    status.game_state === "running"
      ? status.actions.stop
      : status.actions.start;
  const blockedReason =
    !actionable.allowed &&
    actionable.reason &&
    hostingActionReasons[actionable.reason];
  const canInspect =
    operation?.can_inspect === true ||
    (!!operation && jobs.some((job) => job.id === operation.id));
  const capacity = (value: unknown, divisor: number) =>
    typeof value === "number" && Number.isFinite(value)
      ? money(value / divisor)
      : "—";
  const hostLabel =
    status.machine_state === "running"
      ? t("Host is awake")
      : status.machine_state === "stopped"
        ? t("Host is asleep")
        : t("Host status unknown");
  const gameLabel =
    status.game_state === "running"
      ? t("Running")
      : status.game_state === "stopped"
        ? t("Stopped")
        : status.game_state === "starting"
          ? t("Starting")
          : status.game_state === "stopping"
            ? t("Stopping")
            : status.game_state === "unprovisioned"
              ? t("Creating")
              : status.game_state === "error"
                ? t("Needs attention")
                : t("Checking status");
  return (
    <>
      <Card
        title={t("Server status")}
        action={
          <span className={`status status-${status.game_state}`}>
            {gameLabel}
          </span>
        }
      >
        <div className="hosting-status">
          <div className="status-summary" aria-live="polite">
            <h3>{runtimeHeading(status)}</h3>
            <p>{runtimeDescription(status, s)}</p>
          </div>
          <dl className="runtime-phases">
            <div>
              <dt>{t("Host")}</dt>
              <dd>{hostLabel}</dd>
            </div>
            <div>
              <dt>{t("Minecraft")}</dt>
              <dd>{gameLabel}</dd>
            </div>
            <div>
              <dt>{t("Connections")}</dt>
              <dd>
                {status.joinable ? t("Ready for players") : t("Unavailable")}
              </dd>
            </div>
          </dl>
          {operation && (
            <section
              className="operation-card"
              aria-label={t("Current operation")}
            >
              <div className="card-head">
                <h3>{jobTitle(operation)}</h3>
                <Status value={operation.state} />
              </div>
              {operation.progress?.message && (
                <p role="status">
                  {translateError(operation.progress.message)}
                </p>
              )}
              {operation.outcome === "delivery_unknown" ? (
                <p role="alert">
                  {t(
                    "This command may have been delivered. Check the logs before sending a new command.",
                  )}
                </p>
              ) : operation.state === "failed" ? (
                <p>
                  {t(
                    "The operation failed. Review its details before trying again.",
                  )}
                </p>
              ) : (
                <p>
                  {t(
                    "This operation is in progress. Power controls follow the confirmed server state.",
                  )}
                </p>
              )}
              <div className="actions">
                {canInspect && (
                  <button
                    onClick={() =>
                      showJob(operation.id, {
                        ...operation,
                        server_name: s.name,
                      })
                    }
                  >
                    {t("View details")}
                  </button>
                )}
                <a href={`${base}/logs`}>{t("Open logs")}</a>
              </div>
            </section>
          )}
          {!operation && s.inspection && (
            <section className="operation-card" aria-label={t("File access")}>
              <h3>{t("File access")}</h3>
              <p>
                {s.inspection.state === "ready"
                  ? t(
                      "Files are available until {0}. Minecraft remains stopped.",
                      date(s.inspection.expires_at),
                    )
                  : t("Preparing or closing files. Your draft is kept.")}
              </p>
              {s.can_administer && (
                <a href={`${base}/files`}>{t("Open files")}</a>
              )}
            </section>
          )}
          <div className="actions hosting-actions">
            <button
              className={
                status.game_state === "running" ? undefined : "primary"
              }
              disabled={submittedPower || !status.actions.start.allowed}
              onClick={() => act("server_start", { id: s.id })}
            >
              {t("Start Minecraft")}
            </button>
            <button
              className={
                status.game_state === "running" ? "primary" : undefined
              }
              disabled={submittedPower || !status.actions.stop.allowed}
              onClick={() =>
                open({
                  title: message("Stop server"),
                  type: "server_stop",
                  values: { id: s.id },
                  note: () => (
                    <p>
                      {t(
                        "Save and stop “{0}”? Connected players will be disconnected.",
                        s.name,
                      )}
                    </p>
                  ),
                  submit: message("Save and stop"),
                })
              }
            >
              {t("Save and stop")}
            </button>
            <a href={`${base}/console`}>{t("Open console")}</a>
          </div>
          {submittedPower ? (
            <p role="status">{t("Your power request is being processed.")}</p>
          ) : (
            blockedReason && <p className="notice">{t(blockedReason)}</p>
          )}
          <small>
            {t("Last checked")}:{" "}
            {s.last_observed_at
              ? date(s.last_observed_at)
              : t("Not observed yet")}
          </small>
        </div>
      </Card>
      <Card title={t("Allocated resources")}>
        <dl className="resource-grid">
          <div className="metric">
            <dt>{t("Players online")}</dt>
            <dd>
              {status.observation_fresh && status.game_state === "running"
                ? money(s.players)
                : "—"}
            </dd>
          </div>
          <div className="metric">
            <dt>{t("Memory")}</dt>
            <dd>
              {capacity(s.memory_mib, 1024)} <small>GiB</small>
            </dd>
          </div>
          <div className="metric">
            <dt>{t("CPU")}</dt>
            <dd>
              {capacity(s.cpu_millis, 1000)} <small>vCPU</small>
            </dd>
          </div>
          <div className="metric">
            <dt>{t("Storage")}</dt>
            <dd>
              {capacity(s.storage_mib, 1024)} <small>GiB</small>
            </dd>
          </div>
        </dl>
      </Card>
      <Card title={t("Server details")}>
        <dl className="details-list">
          <div>
            <dt>{t("Server software")}</dt>
            <dd>
              {s.software} {s.version}
            </dd>
          </div>
          <div>
            <dt>{t("Visibility")}</dt>
            <dd>
              {t(
                s.visibility === "public"
                  ? "Public"
                  : s.visibility === "invite"
                    ? "Invited players"
                    : "You and administrators",
              )}
            </dd>
          </div>
          <div>
            <dt>{t("Connection")}</dt>
            <dd>
              <code>{me.game_address}</code>
            </dd>
          </div>
          <div>
            <dt>{t("Power target")}</dt>
            <dd>
              {s.desired === "running"
                ? t("Running")
                : s.desired === "stopped"
                  ? t("Stopped")
                  : t("Checking status")}
            </dd>
          </div>
        </dl>
        <a href={`#/servers/${s.id}`}>{t("Connection details")}</a>
      </Card>
    </>
  );
}
function ReadState({ read }: { read: ReturnType<typeof useServerRead> }) {
  return (
    <>
      {read.busy && (
        <p role="status">
          {read.progress?.message
            ? translateError(read.progress.message)
            : t("Reading from the server…")}
        </p>
      )}
      {!!read.error && (
        <p role="alert" className="error">
          {messageError(read.error)}{" "}
          {read.result && t("Previously loaded data is still shown.")}{" "}
          <button onClick={read.refresh}>{t("Retry")}</button>
        </p>
      )}
    </>
  );
}
function Unavailable({ server: s }: { server: Data }) {
  return (
    <p>
      {s.observed === "unprovisioned" || s.observed === "provisioning"
        ? t("Files and logs are available after server creation completes.")
        : t("You no longer have permission to read this server.")}
    </p>
  );
}
function Output({ lines, tail = false }: { lines?: string[]; tail?: boolean }) {
  const output = useRef<HTMLPreElement>(null);
  const pinned = useRef(true);
  useLayoutEffect(() => {
    if (
      tail &&
      pinned.current &&
      output.current &&
      !output.current.contains(document.activeElement)
    )
      output.current.scrollTop = output.current.scrollHeight;
  }, [lines]);
  return (
    <pre
      className="output"
      ref={output}
      tabIndex={0}
      aria-label={t("Server output")}
      onScroll={() => {
        const el = output.current;
        if (el)
          pinned.current =
            el.scrollHeight - el.scrollTop - el.clientHeight < 40;
      }}
    >
      {lines?.length ? lines.join("\n") : t("No output returned.")}
    </pre>
  );
}
function Console({ server: s }: { server: Data }) {
  const { send } = useApp();
  const read = useServerRead(
    "server_logs",
    { id: s.id },
    readAvailable(s),
    true,
  );
  const [draft, setDraft] = useState(consoleDrafts.get(s.id) ?? "");
  useEffect(() => {
    if (read.revoked) {
      consoleDrafts.delete(s.id);
      setDraft("");
    }
  }, [read.revoked, s.id]);
  const [error, setError] = useState<Error | null>(null);
  const [busy, setBusy] = useState(false);
  const signal = useLifetime(!!read.revoked);
  return (
    <Card title={t("Console")}>
      <p>
        {t(
          "Live output updates while this page is visible. Commands require a running server.",
        )}
      </p>
      {!readAvailable(s) ? (
        <Unavailable server={s} />
      ) : (
        <>
          <ReadState read={read} />
          {read.result && (
            <>
              <Output lines={read.result.lines} tail />
              {read.result.truncated && (
                <p>{t("Only the latest output is shown.")}</p>
              )}
              <small>
                {t("Last updated")}:{" "}
                {date(new Date(read.updated ?? Date.now()).toISOString())}
              </small>
            </>
          )}
        </>
      )}
      <form
        className="action-form"
        onSubmit={async (e) => {
          e.preventDefault();
          if (busy || !draft.trim()) return;
          setBusy(true);
          setError(null);
          const submitted = draft;
          try {
            const result = await send("server_console", {
              id: s.id,
              line: submitted,
            });
            if (result.job_id)
              await waitForJob(result.job_id, undefined, signal);
            if (!signal.aborted) {
              if (consoleDrafts.get(s.id) === submitted) {
                setDraft("");
                consoleDrafts.set(s.id, "");
              }
              read.refresh();
            }
          } catch (e) {
            if (!signal.aborted) setError(e as Error);
          } finally {
            if (!signal.aborted) setBusy(false);
          }
        }}
      >
        <label className="field">
          {t("Console command")}
          <input
            value={read.revoked ? "" : draft}
            maxLength={1024}
            disabled={busy || read.revoked}
            onChange={(e) => {
              setDraft(e.target.value);
              consoleDrafts.set(s.id, e.target.value);
            }}
          />
        </label>
        <button
          className="primary"
          disabled={
            read.revoked ||
            busy ||
            s.observed !== "running" ||
            s.desired !== "running" ||
            s.maintenance ||
            !draft.trim()
          }
        >
          {busy ? t("Working…") : t("Send command")}
        </button>
        {error && (
          <p role="alert" className="error">
            {messageError(error)}
          </p>
        )}
      </form>
    </Card>
  );
}
const logDates = new PrivateCache<string>();
function Logs({ server: s }: { server: Data }) {
  const [selected, setSelected] = useState(logDates.get(s.id) ?? todayUTC());
  const read = useServerRead(
    "server_logs",
    { id: s.id, date: selected },
    readAvailable(s) && /^\d{4}-\d{2}-\d{2}$/.test(selected),
  );
  return (
    <Card title={t("Logs")}>
      <label className="field">
        {t("Log date (UTC)")}
        <input
          type="date"
          value={selected}
          max={todayUTC()}
          onChange={(e) => {
            setSelected(e.target.value);
            logDates.set(s.id, e.target.value);
          }}
        />
      </label>
      <p>
        {t(
          "Historical logs use UTC dates. This view does not switch to live output.",
        )}
      </p>
      {(read.result?.dates?.length ?? 0) > 0 && (
        <div className="actions" aria-label={t("Available log dates")}>
          {read.result!.dates.map((d: string) => (
            <button
              key={d}
              aria-pressed={d === selected}
              onClick={() => {
                setSelected(d);
                logDates.set(s.id, d);
              }}
            >
              {d}
            </button>
          ))}
        </div>
      )}
      {!readAvailable(s) ? (
        <Unavailable server={s} />
      ) : (
        <ReadState read={read} />
      )}
      {selected && read.result && (
        <>
          <h3>{read.result.date ?? selected} UTC</h3>
          <Output lines={read.result.lines} />
          {read.result.truncated && (
            <p>{t("This log is truncated to the response limit.")}</p>
          )}
        </>
      )}
    </Card>
  );
}
const folders = new PrivateCache<string>();
function Files({ server: s }: { server: Data }) {
  const { open, send, refresh } = useApp();
  const [path, setPath] = useState(folders.get(s.id) ?? "");
  const [file, setFile] = useState<string | null>(null);
  const [creating, setCreating] = useState(false);
  const [uploading, setUploading] = useState(false);
  const [uploadError, setUploadError] = useState<Error | null>(null);
  const [sessionBusy, setSessionBusy] = useState(false);
  const [sessionError, setSessionError] = useState<Error | null>(null);
  const fileReady =
    readAvailable(s) &&
    (s.desired === "running" || s.inspection?.state === "ready");
  async function filesSession(opening: boolean) {
    setSessionBusy(true);
    setSessionError(null);
    try {
      const result = await send("server_inspection", {
        id: s.id,
        open: opening,
      });
      if (result.job_id) await waitForJob(result.job_id, undefined, signal);
      refresh();
    } catch (e) {
      if (!signal.aborted) setSessionError(e as Error);
    } finally {
      if (!signal.aborted) setSessionBusy(false);
    }
  }
  const read = useServerRead("server_files", { id: s.id, path }, fileReady);
  const writable = stopped(s) && fileReady && s.can_administer && !read.revoked;
  useEffect(() => {
    if (read.revoked) {
      setFile(null);
      setCreating(false);
      setUploadError(null);
      for (const key of fileDrafts.keys())
        if (key.startsWith(s.id + "/")) fileDrafts.delete(key);
    }
  }, [read.revoked, s.id]);
  const parts = path.split("/").filter(Boolean);
  const navigate = (next: string) => {
    setPath(next);
    folders.set(s.id, next);
    setFile(null);
    setCreating(false);
  };
  const signal = useLifetime(!!read.revoked);
  async function upload(file: File, input: HTMLInputElement) {
    setUploading(true);
    setUploadError(null);
    const directory = path;
    try {
      const form = new FormData();
      form.append("file", file);
      const artifact = await api(`/api/v1/servers/${s.id}/artifacts`, {
        method: "POST",
        body: form,
        signal,
      });
      if (signal.aborted) return;
      refresh();
      open({
        title: message("Apply an uploaded file"),
        fields: [
          {
            name: "path",
            label: message("Destination in server"),
            value: [directory, artifact.name ?? file.name]
              .filter(Boolean)
              .join("/"),
          },
        ],
        note: () => (
          <p>
            {t(
              "The uploaded file is saved. Stop the server before applying it. World ZIPs are extracted into the specified folder.",
            )}
          </p>
        ),
        submit: message("Apply file"),
        action: async (v) => {
          const result = await send("server_install", {
            id: s.id,
            artifact: artifact.id,
            path: v.path,
          });
          if (result.job_id) await waitForJob(result.job_id, undefined, signal);
          read.refresh();
        },
      });
    } catch (e) {
      if (!signal.aborted) setUploadError(e as Error);
    } finally {
      if (!signal.aborted) {
        setUploading(false);
        input.value = "";
      }
    }
  }
  return (
    <Card title={t("Files")}>
      {s.desired === "stopped" && s.can_administer && (
        <div className="section-toolbar">
          {s.inspection?.state === "ready" ? (
            <>
              <span role="status">
                {t(
                  "Files are available until {0}. Minecraft remains stopped.",
                  date(s.inspection.expires_at),
                )}
              </span>
              <button
                disabled={sessionBusy}
                onClick={() => void filesSession(false)}
              >
                {t("Close files")}
              </button>
            </>
          ) : (
            <>
              <p role="status">
                {s.inspection
                  ? t("Preparing or closing files. Your draft is kept.")
                  : t(
                      "Open files to start the guest without starting Minecraft.",
                    )}
              </p>
              <button
                disabled={sessionBusy || !!s.inspection || !readAvailable(s)}
                onClick={() => void filesSession(true)}
              >
                {t("Open files")}
              </button>
            </>
          )}
        </div>
      )}
      {sessionError && (
        <p role="alert" className="error">
          {messageError(sessionError)}
        </p>
      )}
      <nav className="file-breadcrumbs" aria-label={t("File location")}>
        <button
          onClick={() => navigate("")}
          aria-current={!path ? "location" : undefined}
        >
          {t("Server root")}
        </button>
        {parts.map((part, i) => (
          <span key={i}>
            {" "}
            /{" "}
            <button
              onClick={() => navigate(parts.slice(0, i + 1).join("/"))}
              aria-current={i === parts.length - 1 ? "location" : undefined}
            >
              {part}
            </button>
          </span>
        ))}
      </nav>
      {!writable && (
        <p className="notice">
          {t(
            "Stop this custom server before saving, uploading, creating or deleting files.",
          )}
        </p>
      )}
      {!readAvailable(s) ? (
        <Unavailable server={s} />
      ) : fileReady ? (
        <ReadState read={read} />
      ) : null}
      {read.result && (
        <>
          <div className="file-list" aria-label={t("Directory entries")}>
            {path && (
              <button onClick={() => navigate(parts.slice(0, -1).join("/"))}>
                {t("Parent folder")}
              </button>
            )}
            {!read.result.entries?.length && (
              <p>{t("This folder is empty.")}</p>
            )}
            {read.result.entries?.map((entry: Data) => (
              <div className="list-row" key={entry.path}>
                <button
                  className="file-name"
                  onClick={() => {
                    if (entry.kind === "directory") navigate(entry.path);
                    else {
                      setFile(entry.path);
                      setCreating(false);
                    }
                  }}
                >
                  {entry.kind === "directory"
                    ? t("Folder: {0}", entry.name)
                    : entry.name}
                </button>
                <small>
                  {entry.kind === "file" && entry.bytes != null
                    ? `${money(entry.bytes)} ${t("bytes")}`
                    : ""}{" "}
                  {entry.modified_at ? date(entry.modified_at) : ""}
                </small>
              </div>
            ))}
          </div>
          {read.result.truncated && (
            <p>{t("This directory has more entries than can be shown.")}</p>
          )}
        </>
      )}
      <div className="actions">
        <button
          disabled={!writable || uploading}
          onClick={() => {
            setFile(null);
            setCreating(true);
          }}
        >
          {t("New text file")}
        </button>
        <button
          disabled={!writable || uploading}
          onClick={() =>
            open({
              title: message("Create folder"),
              fields: [
                { name: "name", label: message("Folder name"), max: 128 },
              ],
              submit: message("Create"),
              action: async (v) => {
                if (
                  !v.name.trim() ||
                  /[\\/]/.test(v.name) ||
                  [".", ".."].includes(v.name)
                )
                  throw new ApiError(
                    400,
                    message("Enter a single file or folder name."),
                  );
                const result = await send("server_directory_create", {
                  id: s.id,
                  path: [path, v.name].filter(Boolean).join("/"),
                });
                if (result.job_id)
                  await waitForJob(result.job_id, undefined, signal);
                read.refresh();
              },
            })
          }
        >
          {t("Create folder")}
        </button>
      </div>
      <label className="field upload-zone">
        {uploading
          ? t("Uploading…")
          : t("Upload into {0}", path || t("Server root"))}
        <input
          type="file"
          disabled={!writable || uploading}
          onChange={(e) => {
            const f = e.currentTarget.files?.[0];
            if (f) void upload(f, e.currentTarget);
          }}
        />
      </label>
      {uploadError && (
        <p className="error" role="alert">
          {messageError(uploadError)}
        </p>
      )}
      {s.artifacts?.length > 0 && (
        <details>
          <summary>{t("Uploaded files")}</summary>
          <p>
            {t(
              "These uploads can be applied into the selected folder. They are not the server directory contents.",
            )}
          </p>
          {s.artifacts.map((a: Data) => (
            <div className="list-row" key={a.id}>
              <div className="grow">
                <strong>{a.name}</strong>
                <small>
                  {money(a.bytes)} {t("bytes")}
                </small>
              </div>
              <button
                disabled={!writable || uploading}
                onClick={() =>
                  open({
                    title: message("Apply file"),
                    fields: [
                      {
                        name: "path",
                        label: message("Destination in server"),
                        value: [path, a.name].filter(Boolean).join("/"),
                      },
                    ],
                    submit: message("Apply file"),
                    action: async (v) => {
                      const result = await send("server_install", {
                        id: s.id,
                        artifact: a.id,
                        path: v.path,
                      });
                      if (result.job_id)
                        await waitForJob(result.job_id, undefined, signal);
                      read.refresh();
                      refresh();
                    },
                  })
                }
              >
                {t("Apply")}
              </button>
            </div>
          ))}
        </details>
      )}
      {!read.revoked && (file || creating) && (
        <FileEditor
          key={file ?? "new/" + path}
          server={s}
          readable={fileReady}
          path={file}
          directory={path}
          writable={writable}
          onChanged={read.refresh}
          onClose={() => {
            setFile(null);
            setCreating(false);
          }}
        />
      )}
    </Card>
  );
}
function FileEditor({
  server: s,
  readable,
  path,
  directory,
  writable,
  onChanged,
  onClose,
}: {
  server: Data;
  readable: boolean;
  path: string | null;
  directory: string;
  writable: boolean;
  onChanged: () => void;
  onClose: () => void;
}) {
  const { send, open } = useApp();
  const key = `${s.id}/${path ?? directory + "/new"}`;
  const saved = fileDrafts.get(key);
  const [text, setText] = useState(saved?.text ?? "");
  const [sha, setSha] = useState<string | null>(
    path ? (saved?.sha ?? null) : null,
  );
  const [name, setName] = useState("");
  const [initialized, setInitialized] = useState(!!saved || !path);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<Error | null>(null);
  const [outcome, setOutcome] = useState<SystemMessage | null>(null);
  const read = useServerRead(
    "server_file_read",
    { id: s.id, path: path ?? "" },
    !!path && readable,
  );
  const signal = useLifetime(!!read.revoked);
  useEffect(() => {
    if (read.revoked) {
      fileDrafts.delete(key);
      setText("");
      setSha(null);
      setInitialized(false);
      setError(null);
      setOutcome(null);
    }
  }, [read.revoked, key]);
  const dirty = path
    ? sha !== read.result?.sha256 || text !== read.result?.text
    : !!text;
  useEffect(() => {
    if (read.result && !initialized) {
      setText(read.result.text);
      setSha(read.result.sha256);
      setInitialized(true);
    }
  }, [read.result, initialized]);
  const destination = path ?? [directory, name].filter(Boolean).join("/");
  async function save() {
    setBusy(true);
    setError(null);
    setOutcome(null);
    try {
      if (
        !path &&
        (!name.trim() || /[\\/]/.test(name) || [".", ".."].includes(name))
      )
        throw new ApiError(400, message("Enter a single file or folder name."));
      if (new TextEncoder().encode(text).length > 65536)
        throw new ApiError(400, message("Text files must be at most 64 KiB."));
      const result = await send("server_file_write", {
        id: s.id,
        path: destination,
        text,
        expected_sha256: sha,
      });
      const final = result.job_id
        ? await waitForJob(result.job_id, undefined, signal)
        : result;
      if (signal.aborted) return;
      if (final.effect !== "committed" || !final.sha256)
        throw new ApiError(
          409,
          message(
            "The save outcome is uncertain. Open action details before trying again.",
          ),
        );
      setSha(final.sha256);
      fileDrafts.set(key, { text, sha: final.sha256 });
      setOutcome(message("Saved {0}.", destination));
      onChanged();
      read.refresh();
      if (!path) {
        fileDrafts.delete(key);
        onClose();
      }
    } catch (e) {
      if (!signal.aborted) setError(e as Error);
    } finally {
      if (!signal.aborted) setBusy(false);
    }
  }
  return (
    <section className="file-editor" aria-label={t("Text editor")}>
      <div className="card-head">
        <h3>{path ?? t("New text file")}</h3>
        <button disabled={busy} onClick={onClose}>
          {t("Close editor")}
        </button>
      </div>
      {path && (
        <>
          <ReadState read={read} />
          <button disabled={read.busy || busy} onClick={read.refresh}>
            {t("Read current file version")}
          </button>
          {read.result && (
            <details>
              <summary>{t("Version last read from server")}</summary>
              <pre className="output">{read.result.text}</pre>
            </details>
          )}
        </>
      )}
      {read.result?.bytes > 65536 ? (
        <p role="alert">{t("Text files must be at most 64 KiB.")}</p>
      ) : (
        initialized &&
        !read.revoked &&
        (!path || !!read.result) && (
          <>
            {!path && (
              <label className="field">
                {t("File name")}
                <input
                  value={name}
                  maxLength={128}
                  disabled={busy}
                  onChange={(e) => setName(e.target.value)}
                />
              </label>
            )}
            {path &&
              read.result &&
              !read.busy &&
              !read.error &&
              sha !== read.result.sha256 && (
                <p role="alert" className="error">
                  {t(
                    "The file changed on the server. Your draft is retained. Reopen or review the current version before saving.",
                  )}
                </p>
              )}
            <label className="field">
              {t("File text")}
              <textarea
                aria-label={t("File text")}
                className="code-editor"
                value={text}
                maxLength={65536}
                rows={16}
                disabled={busy}
                onChange={(e) => {
                  setText(e.target.value);
                  fileDrafts.set(key, { text: e.target.value, sha });
                }}
              />
            </label>
            <p>
              <small>
                {t(
                  "Bounded UTF-8 text only. Saving checks the original file version.",
                )}
              </small>
            </p>
            <div className="actions">
              <button
                className="primary"
                disabled={
                  !writable ||
                  busy ||
                  !initialized ||
                  (!!path && !sha) ||
                  (!path && !name)
                }
                onClick={() => void save()}
              >
                {busy ? t("Saving…") : t("Save file")}
              </button>
              {path && (
                <button
                  className="danger"
                  disabled={!writable || busy || !sha}
                  onClick={() =>
                    open({
                      title: message("Delete file"),
                      note: () => (
                        <p>
                          {t(
                            "Delete {0}? This removes only this file. Unsaved edits will be discarded after deletion.",
                            path,
                          )}
                        </p>
                      ),
                      submit: message("Confirm deletion"),
                      action: async () => {
                        const result = await send("server_file_delete", {
                          id: s.id,
                          path,
                          expected_sha256: sha,
                        });
                        if (result.job_id)
                          await waitForJob(result.job_id, undefined, signal);
                        fileDrafts.delete(key);
                        onChanged();
                        onClose();
                      },
                    })
                  }
                >
                  {t("Delete file")}
                </button>
              )}
              {path && dirty && (
                <button
                  disabled={busy || !read.result}
                  onClick={() =>
                    open({
                      title: message("Discard local edits"),
                      note: () => (
                        <p>
                          {t(
                            "Replace this draft with the version last read from the server?",
                          )}
                        </p>
                      ),
                      submit: message("Discard local edits"),
                      action: async () => {
                        setText(read.result!.text);
                        setSha(read.result!.sha256);
                        fileDrafts.delete(key);
                        setError(null);
                      },
                    })
                  }
                >
                  {t("Discard local edits")}
                </button>
              )}
            </div>
          </>
        )
      )}
      {error && (
        <p role="alert" className="error">
          {messageError(error)}
        </p>
      )}
      {outcome && <p role="status">{renderSystemMessage(outcome)}</p>}
    </section>
  );
}
function Members({ server: s }: { server: Data }) {
  const { open, send, showJob, jobs } = useApp();
  const roles = [
    { value: "guest", label: t("Member") },
    { value: "administrator", label: t("Administrator") },
  ];
  return (
    <Card title={t("Members")}>
      <ActionForm
        fields={[
          { name: "member", label: t("Player"), type: "player" },
          { name: "role", label: t("Role"), type: "select", options: roles },
        ]}
        submit={t("Add member")}
        onSubmit={(v) => send("server_member", { id: s.id, ...v })}
      />
      {!s.members?.length && <Empty>{t("No additional members.")}</Empty>}
      {s.members?.map((m: Data) => {
        const submitted = jobs.find(
          (j) =>
            j.server_id === s.id &&
            j.member_id === m.account_id &&
            j.kind?.replaceAll("_", ".") === "server.operator",
        );
        const recorded = m.minecraft_operator_job;
        const op =
          submitted && (!recorded || submitted.id === recorded.id)
            ? { ...recorded, ...submitted }
            : recorded;
        const pending =
          op && !["succeeded", "failed", "cancelled"].includes(op.state);
        const applied =
          op?.state === "succeeded" && (op.result?.operator ?? op.operator);
        const supported = s.kind === "custom" && s.software === "paper";
        const verified = m.minecraft_identity?.ready === true;
        const owner = m.is_owner || m.account_id === s.owner;
        return (
          <div className="member-row" key={m.account_id}>
            <strong>{m.name}</strong>{" "}
            {owner ? (
              <span>{t("Owner · Administrator")}</span>
            ) : (
              <ActionForm
                fields={[
                  {
                    name: "role",
                    label: t("Role"),
                    type: "select",
                    value: m.role,
                    options:
                      m.role === "operator"
                        ? [
                            ...roles,
                            {
                              value: "operator",
                              label: t("Legacy power and logs access"),
                            },
                          ]
                        : roles,
                  },
                ]}
                submit={t("Save role")}
                onSubmit={(v) =>
                  send("server_member", {
                    id: s.id,
                    member: m.account_id,
                    ...v,
                  })
                }
              />
            )}
            <div className="operator-control">
              <h3>{t("Minecraft operator")}</h3>
              <p>
                {op
                  ? pending
                    ? t("Operator change pending.")
                    : op.state === "failed"
                      ? t(
                          "Operator change failed. Open details before retrying.",
                        )
                      : op.state === "succeeded"
                        ? applied
                          ? t("Operator grant saved; effective on next start.")
                          : t(
                              "Operator removal saved; effective on next start.",
                            )
                        : t("Operator state is unknown.")
                  : t(
                      "No operator change recorded. Hosting roles do not grant Minecraft OP.",
                    )}
              </p>
              {op?.id && (
                <button
                  onClick={() =>
                    showJob(op.id, {
                      kind: "server.operator",
                      server_name: s.name,
                    })
                  }
                >
                  {t("View details")}
                </button>
              )}
              {supported && verified ? (
                <>
                  <button
                    disabled={!stopped(s) || pending}
                    onClick={() =>
                      open({
                        title: message("Grant Minecraft operator"),
                        note: () => (
                          <p>
                            {t(
                              "Grant Minecraft OP to {0} on {1}? It takes effect on next start and requires a verified Java identity.",
                              m.name,
                              s.name,
                            )}
                          </p>
                        ),
                        type: "server_operator",
                        values: {
                          id: s.id,
                          member: m.account_id,
                          operator: true,
                        },
                        submit: message("Grant operator"),
                      })
                    }
                  >
                    {t("Grant operator")}
                  </button>
                  <button
                    disabled={!stopped(s) || pending}
                    onClick={() =>
                      open({
                        title: message("Remove Minecraft operator"),
                        type: "server_operator",
                        values: {
                          id: s.id,
                          member: m.account_id,
                          operator: false,
                        },
                        note: () => (
                          <p>
                            {t(
                              "Remove Minecraft OP from {0} on {1} on next start?",
                              m.name,
                              s.name,
                            )}
                          </p>
                        ),
                        submit: message("Remove operator"),
                      })
                    }
                  >
                    {t("Remove operator")}
                  </button>
                  {!stopped(s) && (
                    <small>
                      {t(
                        "Stop this Paper server before changing Minecraft OP.",
                      )}
                    </small>
                  )}
                </>
              ) : (
                <p>
                  {supported
                    ? m.minecraft_identity?.reason
                      ? translateError(m.minecraft_identity.reason)
                      : t(
                          "Link and verify a Java account before changing Minecraft OP.",
                        )
                    : t(
                        "Minecraft OP changes are supported only on custom Paper servers.",
                      )}
                </p>
              )}
            </div>
            {!owner && (
              <button
                onClick={() =>
                  open({
                    title: message("Remove member"),
                    type: "server_member",
                    values: { id: s.id, member: m.account_id, role: null },
                    note: () => (
                      <p>
                        {t(
                          "Remove {0} from {1}? Hosting membership and Minecraft OP are separate.",
                          m.name,
                          s.name,
                        )}
                      </p>
                    ),
                    submit: message("Remove member"),
                  })
                }
              >
                {t("Remove member")}
              </button>
            )}
          </div>
        );
      })}
    </Card>
  );
}
