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
  if (!s.id) return <Empty>{t("text.this_page_could_not_be_found")}</Empty>;
  if (!s.can_manage)
    return (
      <p role="alert">
        {t("text.you_no_longer_have_permission_to_manage_this_server")}
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
        {t("text.administrator_permission_is_required_for_this_page")}
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
            {t("text.file_tools_are_available_for_custom_servers_only")}
          </Empty>
        ))}
      {section === "manage-settings" && (
        <Card title={s.name}>
          <ActionForm
            fields={[
              { name: "name", label: t("text.name"), value: s.name, max: 64 },
              {
                name: "visibility",
                label: t("text.visibility"),
                type: "select",
                value: s.visibility,
                options: [
                  { value: "private", label: t("text.you_and_administrators") },
                  { value: "invite", label: t("text.invited_players") },
                  { value: "public", label: t("text.public") },
                ],
              },
            ]}
            onSubmit={(v) => send("server_configure", { id: s.id, ...v })}
          />
        </Card>
      )}
      {section === "manage-members" && <Members server={s} />}
      {section === "manage-backups" && (
        <Card title={t("text.backups")}>
          <p>
            {t(
              "text.restoring_replaces_the_current_world_with_the_backup_st_ccb17e658d",
            )}
          </p>
          {s.kind === "custom" ? (
            <button
              disabled={
                s.maintenance || !["running", "stopped"].includes(s.observed)
              }
              onClick={() => act("server_backup", { id: s.id })}
            >
              {t("text.create_backup")}
            </button>
          ) : (
            <p>{t("text.use_administration_to_manage_official_backups")}</p>
          )}
          {!s.backups?.length && <Empty>{t("text.no_backups_yet")}</Empty>}
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
                    title: message("text.restore_a_backup"),
                    type: "server_restore",
                    values: { id: s.id, backup: b.id },
                    note: () => (
                      <p>
                        {t(
                          "text.restore_0_to_1_back_up_the_current_world_first_if_you_w_102542b4ec",
                          s.name,
                          date(b.created_at),
                        )}
                      </p>
                    ),
                    submit: message("text.restore_to_this_point"),
                  })
                }
              >
                {t("text.restore")}
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
      return t("text.minecraft_is_running");
    case "stopped":
      return t("text.minecraft_is_stopped");
    case "starting":
      return t("text.minecraft_is_starting");
    case "stopping":
      return t("text.minecraft_is_stopping");
    case "unprovisioned":
      return t("text.creating_your_server");
    case "error":
      return t("text.your_server_needs_attention");
    default:
      return t("text.checking_your_server");
  }
}
function runtimeDescription(status: HostingStatus, server: Data) {
  if (!status.observation_fresh && status.game_state !== "unprovisioned")
    return t(
      "text.a_current_minecraft_observation_is_needed_before_power_can_change",
    );
  if (server.inspection)
    return t(
      "text.file_access_can_keep_the_host_awake_while_minecraft_stays_stopped",
    );
  if (status.joinable) return t("text.players_can_connect_now");
  switch (status.game_state) {
    case "running":
      return t("text.new_connections_are_paused_while_server_work_finishes");
    case "stopped":
      return t("text.start_minecraft_when_you_are_ready_to_play");
    case "starting":
      return t("text.players_can_connect_once_minecraft_finishes_starting");
    case "stopping":
      return t("text.player_data_is_being_saved_before_the_server_stops");
    case "unprovisioned":
      return t("text.the_host_and_minecraft_are_being_prepared");
    case "error":
      return t(
        "text.review_the_server_error_and_current_operation_before_trying_again",
      );
    default:
      return t(
        "text.power_controls_will_be_available_after_the_server_is_checked",
      );
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
      ? t("text.host_is_awake")
      : status.machine_state === "stopped"
        ? t("text.host_is_asleep")
        : t("text.host_status_unknown");
  const gameLabel =
    status.game_state === "running"
      ? t("text.running")
      : status.game_state === "stopped"
        ? t("text.stopped")
        : status.game_state === "starting"
          ? t("text.starting")
          : status.game_state === "stopping"
            ? t("text.stopping")
            : status.game_state === "unprovisioned"
              ? t("text.creating")
              : status.game_state === "error"
                ? t("text.needs_attention")
                : t("text.checking_status");
  return (
    <>
      <Card
        title={t("text.server_status")}
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
              <dt>{t("text.host")}</dt>
              <dd>{hostLabel}</dd>
            </div>
            <div>
              <dt>{t("text.minecraft")}</dt>
              <dd>{gameLabel}</dd>
            </div>
            <div>
              <dt>{t("text.connections")}</dt>
              <dd>
                {status.joinable
                  ? t("text.ready_for_players")
                  : t("text.unavailable")}
              </dd>
            </div>
          </dl>
          {operation && (
            <section
              className="operation-card"
              aria-label={t("text.current_operation")}
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
                    "text.this_command_may_have_been_delivered_check_the_logs_bef_388094c092",
                  )}
                </p>
              ) : operation.state === "failed" ? (
                <p>
                  {t(
                    "text.the_operation_failed_review_its_details_before_trying_again",
                  )}
                </p>
              ) : (
                <p>
                  {t(
                    "text.this_operation_is_in_progress_power_controls_follow_the_d9d53a51cf",
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
                    {t("text.view_details")}
                  </button>
                )}
                <a href={`${base}/logs`}>{t("text.open_logs")}</a>
              </div>
            </section>
          )}
          {!operation && s.inspection && (
            <section
              className="operation-card"
              aria-label={t("text.file_access")}
            >
              <h3>{t("text.file_access")}</h3>
              <p>
                {s.inspection.state === "ready"
                  ? t(
                      "text.files_are_available_until_0_minecraft_remains_stopped",
                      date(s.inspection.expires_at),
                    )
                  : t("text.preparing_or_closing_files_your_draft_is_kept")}
              </p>
              {s.can_administer && (
                <a href={`${base}/files`}>{t("text.open_files")}</a>
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
              {t("text.start_minecraft")}
            </button>
            <button
              className={
                status.game_state === "running" ? "primary" : undefined
              }
              disabled={submittedPower || !status.actions.stop.allowed}
              onClick={() =>
                open({
                  title: message("text.stop_server"),
                  type: "server_stop",
                  values: { id: s.id },
                  note: () => (
                    <p>
                      {t(
                        "text.save_and_stop_0_connected_players_will_be_disconnected",
                        s.name,
                      )}
                    </p>
                  ),
                  submit: message("text.save_and_stop"),
                })
              }
            >
              {t("text.save_and_stop")}
            </button>
            <a href={`${base}/console`}>{t("text.open_console")}</a>
          </div>
          {submittedPower ? (
            <p role="status">
              {t("text.your_power_request_is_being_processed")}
            </p>
          ) : (
            blockedReason && <p className="notice">{t(blockedReason)}</p>
          )}
          <small>
            {t("text.last_checked")}:{" "}
            {s.last_observed_at
              ? date(s.last_observed_at)
              : t("text.not_observed_yet")}
          </small>
        </div>
      </Card>
      <Card title={t("text.allocated_resources")}>
        <dl className="resource-grid">
          <div className="metric">
            <dt>{t("text.players_online_ae9bb529")}</dt>
            <dd>
              {status.observation_fresh && status.game_state === "running"
                ? money(s.players)
                : "—"}
            </dd>
          </div>
          <div className="metric">
            <dt>{t("text.memory")}</dt>
            <dd>
              {capacity(s.memory_mib, 1024)} <small>GiB</small>
            </dd>
          </div>
          <div className="metric">
            <dt>{t("text.cpu")}</dt>
            <dd>
              {capacity(s.cpu_millis, 1000)} <small>vCPU</small>
            </dd>
          </div>
          <div className="metric">
            <dt>{t("text.storage")}</dt>
            <dd>
              {capacity(s.storage_mib, 1024)} <small>GiB</small>
            </dd>
          </div>
        </dl>
      </Card>
      <Card title={t("text.server_details")}>
        <dl className="details-list">
          <div>
            <dt>{t("text.server_software")}</dt>
            <dd>
              {s.software} {s.version}
            </dd>
          </div>
          <div>
            <dt>{t("text.visibility")}</dt>
            <dd>
              {t(
                s.visibility === "public"
                  ? "text.public"
                  : s.visibility === "invite"
                    ? "text.invited_players"
                    : "text.you_and_administrators",
              )}
            </dd>
          </div>
          <div>
            <dt>{t("text.connection")}</dt>
            <dd>
              <code>{me.game_address}</code>
            </dd>
          </div>
          <div>
            <dt>{t("text.power_target")}</dt>
            <dd>
              {s.desired === "running"
                ? t("text.running")
                : s.desired === "stopped"
                  ? t("text.stopped")
                  : t("text.checking_status")}
            </dd>
          </div>
        </dl>
        <a href={`#/worlds/${s.id}`}>{t("text.connection_details")}</a>
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
            : t("text.reading_from_the_server")}
        </p>
      )}
      {!!read.error && (
        <p role="alert" className="error">
          {messageError(read.error)}{" "}
          {read.result && t("text.previously_loaded_data_is_still_shown")}{" "}
          <button onClick={read.refresh}>{t("text.retry")}</button>
        </p>
      )}
    </>
  );
}
function Unavailable({ server: s }: { server: Data }) {
  return (
    <p>
      {s.observed === "unprovisioned" || s.observed === "provisioning"
        ? t("text.files_and_logs_are_available_after_server_creation_completes")
        : t("text.you_no_longer_have_permission_to_read_this_server")}
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
      aria-label={t("text.server_output")}
      onScroll={() => {
        const el = output.current;
        if (el)
          pinned.current =
            el.scrollHeight - el.scrollTop - el.clientHeight < 40;
      }}
    >
      {lines?.length ? lines.join("\n") : t("text.no_output_returned")}
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
    <Card title={t("text.console")}>
      <p>
        {t(
          "text.live_output_updates_while_this_page_is_visible_commands_2de2e9dd2c",
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
                <p>{t("text.only_the_latest_output_is_shown")}</p>
              )}
              <small>
                {t("text.last_updated")}:{" "}
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
          {t("text.console_command")}
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
          {busy ? t("text.working") : t("text.send_command")}
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
    <Card title={t("text.logs")}>
      <label className="field">
        {t("text.log_date_utc")}
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
          "text.historical_logs_use_utc_dates_this_view_does_not_switch_28f66ddb87",
        )}
      </p>
      {(read.result?.dates?.length ?? 0) > 0 && (
        <div className="actions" aria-label={t("text.available_log_dates")}>
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
            <p>{t("text.this_log_is_truncated_to_the_response_limit")}</p>
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
        title: message("text.apply_an_uploaded_file"),
        fields: [
          {
            name: "path",
            label: message("text.destination_in_server"),
            value: [directory, artifact.name ?? file.name]
              .filter(Boolean)
              .join("/"),
          },
        ],
        note: () => (
          <p>
            {t(
              "text.the_uploaded_file_is_saved_stop_the_server_before_apply_ad6df23a14",
            )}
          </p>
        ),
        submit: message("text.apply_file"),
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
    <Card title={t("text.files")}>
      {s.desired === "stopped" && s.can_administer && (
        <div className="section-toolbar">
          {s.inspection?.state === "ready" ? (
            <>
              <span role="status">
                {t(
                  "text.files_are_available_until_0_minecraft_remains_stopped",
                  date(s.inspection.expires_at),
                )}
              </span>
              <button
                disabled={sessionBusy}
                onClick={() => void filesSession(false)}
              >
                {t("text.close_files")}
              </button>
            </>
          ) : (
            <>
              <p role="status">
                {s.inspection
                  ? t("text.preparing_or_closing_files_your_draft_is_kept")
                  : t(
                      "text.open_files_to_start_the_guest_without_starting_minecraft",
                    )}
              </p>
              <button
                disabled={sessionBusy || !!s.inspection || !readAvailable(s)}
                onClick={() => void filesSession(true)}
              >
                {t("text.open_files")}
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
      <nav className="file-breadcrumbs" aria-label={t("text.file_location")}>
        <button
          onClick={() => navigate("")}
          aria-current={!path ? "location" : undefined}
        >
          {t("text.server_root")}
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
            "text.stop_this_custom_server_before_saving_uploading_creatin_68b9378f0d",
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
          <div className="file-list" aria-label={t("text.directory_entries")}>
            {path && (
              <button onClick={() => navigate(parts.slice(0, -1).join("/"))}>
                {t("text.parent_folder")}
              </button>
            )}
            {!read.result.entries?.length && (
              <p>{t("text.this_folder_is_empty")}</p>
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
                    ? t("text.folder_0", entry.name)
                    : entry.name}
                </button>
                <small>
                  {entry.kind === "file" && entry.bytes != null
                    ? `${money(entry.bytes)} ${t("text.bytes")}`
                    : ""}{" "}
                  {entry.modified_at ? date(entry.modified_at) : ""}
                </small>
              </div>
            ))}
          </div>
          {read.result.truncated && (
            <p>{t("text.this_directory_has_more_entries_than_can_be_shown")}</p>
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
          {t("text.new_text_file")}
        </button>
        <button
          disabled={!writable || uploading}
          onClick={() =>
            open({
              title: message("text.create_folder"),
              fields: [
                { name: "name", label: message("text.folder_name"), max: 128 },
              ],
              submit: message("text.create"),
              action: async (v) => {
                if (
                  !v.name.trim() ||
                  /[\\/]/.test(v.name) ||
                  [".", ".."].includes(v.name)
                )
                  throw new ApiError(
                    400,
                    message("text.enter_a_single_file_or_folder_name"),
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
          {t("text.create_folder")}
        </button>
      </div>
      <label className="field upload-zone">
        {uploading
          ? t("text.uploading")
          : t("text.upload_into_0", path || t("text.server_root"))}
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
          <summary>{t("text.uploaded_files")}</summary>
          <p>
            {t(
              "text.these_uploads_can_be_applied_into_the_selected_folder_t_9fafced573",
            )}
          </p>
          {s.artifacts.map((a: Data) => (
            <div className="list-row" key={a.id}>
              <div className="grow">
                <strong>{a.name}</strong>
                <small>
                  {money(a.bytes)} {t("text.bytes")}
                </small>
              </div>
              <button
                disabled={!writable || uploading}
                onClick={() =>
                  open({
                    title: message("text.apply_file"),
                    fields: [
                      {
                        name: "path",
                        label: message("text.destination_in_server"),
                        value: [path, a.name].filter(Boolean).join("/"),
                      },
                    ],
                    submit: message("text.apply_file"),
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
                {t("text.apply")}
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
        throw new ApiError(
          400,
          message("text.enter_a_single_file_or_folder_name"),
        );
      if (new TextEncoder().encode(text).length > 65536)
        throw new ApiError(
          400,
          message("text.text_files_must_be_at_most_64_kib"),
        );
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
            "text.the_save_outcome_is_uncertain_open_action_details_befor_83e461c9c2",
          ),
        );
      setSha(final.sha256);
      fileDrafts.set(key, { text, sha: final.sha256 });
      setOutcome(message("text.saved_0", destination));
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
    <section className="file-editor" aria-label={t("text.text_editor")}>
      <div className="card-head">
        <h3>{path ?? t("text.new_text_file")}</h3>
        <button disabled={busy} onClick={onClose}>
          {t("text.close_editor")}
        </button>
      </div>
      {path && (
        <>
          <ReadState read={read} />
          <button disabled={read.busy || busy} onClick={read.refresh}>
            {t("text.read_current_file_version")}
          </button>
          {read.result && (
            <details>
              <summary>{t("text.version_last_read_from_server")}</summary>
              <pre className="output">{read.result.text}</pre>
            </details>
          )}
        </>
      )}
      {read.result?.bytes > 65536 ? (
        <p role="alert">{t("text.text_files_must_be_at_most_64_kib")}</p>
      ) : (
        initialized &&
        !read.revoked &&
        (!path || !!read.result) && (
          <>
            {!path && (
              <label className="field">
                {t("text.file_name")}
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
                    "text.the_file_changed_on_the_server_your_draft_is_retained_r_8dd2b2396d",
                  )}
                </p>
              )}
            <label className="field">
              {t("text.file_text")}
              <textarea
                aria-label={t("text.file_text")}
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
                  "text.bounded_utf_8_text_only_saving_checks_the_original_file_version",
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
                {busy ? t("text.saving_23e39291") : t("text.save_file")}
              </button>
              {path && (
                <button
                  className="danger"
                  disabled={!writable || busy || !sha}
                  onClick={() =>
                    open({
                      title: message("text.delete_file"),
                      note: () => (
                        <p>
                          {t(
                            "text.delete_0_this_removes_only_this_file_unsaved_edits_will_d2046e8d01",
                            path,
                          )}
                        </p>
                      ),
                      submit: message("text.confirm_deletion"),
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
                  {t("text.delete_file")}
                </button>
              )}
              {path && dirty && (
                <button
                  disabled={busy || !read.result}
                  onClick={() =>
                    open({
                      title: message("text.discard_local_edits"),
                      note: () => (
                        <p>
                          {t(
                            "text.replace_this_draft_with_the_version_last_read_from_the_server",
                          )}
                        </p>
                      ),
                      submit: message("text.discard_local_edits"),
                      action: async () => {
                        setText(read.result!.text);
                        setSha(read.result!.sha256);
                        fileDrafts.delete(key);
                        setError(null);
                      },
                    })
                  }
                >
                  {t("text.discard_local_edits")}
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
    { value: "guest", label: t("text.member") },
    { value: "administrator", label: t("text.administrator") },
  ];
  return (
    <Card title={t("text.members")}>
      <ActionForm
        fields={[
          { name: "member", label: t("text.player"), type: "player" },
          {
            name: "role",
            label: t("text.role"),
            type: "select",
            options: roles,
          },
        ]}
        submit={t("text.add_member")}
        onSubmit={(v) => send("server_member", { id: s.id, ...v })}
      />
      {!s.members?.length && <Empty>{t("text.no_additional_members")}</Empty>}
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
              <span>{t("text.owner_administrator")}</span>
            ) : (
              <ActionForm
                fields={[
                  {
                    name: "role",
                    label: t("text.role"),
                    type: "select",
                    value: m.role,
                    options:
                      m.role === "operator"
                        ? [
                            ...roles,
                            {
                              value: "operator",
                              label: t("text.legacy_power_and_logs_access"),
                            },
                          ]
                        : roles,
                  },
                ]}
                submit={t("text.save_role")}
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
              <h3>{t("text.minecraft_operator")}</h3>
              <p>
                {op
                  ? pending
                    ? t("text.operator_change_pending")
                    : op.state === "failed"
                      ? t(
                          "text.operator_change_failed_open_details_before_retrying",
                        )
                      : op.state === "succeeded"
                        ? applied
                          ? t(
                              "text.operator_grant_saved_effective_on_next_start",
                            )
                          : t(
                              "text.operator_removal_saved_effective_on_next_start",
                            )
                        : t("text.operator_state_is_unknown")
                  : t(
                      "text.no_operator_change_recorded_hosting_roles_do_not_grant_984e075385",
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
                  {t("text.view_details")}
                </button>
              )}
              {supported && verified ? (
                <>
                  <button
                    disabled={!stopped(s) || pending}
                    onClick={() =>
                      open({
                        title: message("text.grant_minecraft_operator"),
                        note: () => (
                          <p>
                            {t(
                              "text.grant_minecraft_op_to_0_on_1_it_takes_effect_on_next_st_4d6f77a129",
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
                        submit: message("text.grant_operator"),
                      })
                    }
                  >
                    {t("text.grant_operator")}
                  </button>
                  <button
                    disabled={!stopped(s) || pending}
                    onClick={() =>
                      open({
                        title: message("text.remove_minecraft_operator"),
                        type: "server_operator",
                        values: {
                          id: s.id,
                          member: m.account_id,
                          operator: false,
                        },
                        note: () => (
                          <p>
                            {t(
                              "text.remove_minecraft_op_from_0_on_1_on_next_start",
                              m.name,
                              s.name,
                            )}
                          </p>
                        ),
                        submit: message("text.remove_operator"),
                      })
                    }
                  >
                    {t("text.remove_operator")}
                  </button>
                  {!stopped(s) && (
                    <small>
                      {t(
                        "text.stop_this_paper_server_before_changing_minecraft_op",
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
                          "text.link_and_verify_a_java_account_before_changing_minecraft_op",
                        )
                    : t(
                        "text.minecraft_op_changes_are_supported_only_on_custom_paper_servers",
                      )}
                </p>
              )}
            </div>
            {!owner && (
              <button
                onClick={() =>
                  open({
                    title: message("text.remove_member"),
                    type: "server_member",
                    values: { id: s.id, member: m.account_id, role: null },
                    note: () => (
                      <p>
                        {t(
                          "text.remove_0_from_1_hosting_membership_and_minecraft_op_are_separate",
                          m.name,
                          s.name,
                        )}
                      </p>
                    ),
                    submit: message("text.remove_member"),
                  })
                }
              >
                {t("text.remove_member")}
              </button>
            )}
          </div>
        );
      })}
    </Card>
  );
}
