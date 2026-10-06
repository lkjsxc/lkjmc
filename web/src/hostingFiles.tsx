import {
  useEffect,
  useId,
  useLayoutEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import { api, ApiError, command, date, type Data } from "./api";
import { useApp } from "./App";
import { PrivateCache, onResourceReset } from "./identity";
import {
  getLocale,
  message,
  messageError,
  renderSystemMessage,
  t,
  translateError,
  type SystemMessage,
} from "./i18n";
import { hostingStatus, hostingActionReasons } from "./hostingStatus";
import { useServerRead, waitForJob } from "./serverReads";

const ENTRY_LIMIT = 256;
type FileDraft = {
  text: string;
  sha: string | null;
  name?: string;
  outcome?: SystemMessage;
};
const fileDrafts = new PrivateCache<FileDraft>(16);
export function clearFileDrafts(id: string) {
  for (const key of fileDrafts.keys())
    if (key.startsWith(id + "/")) fileDrafts.delete(key);
  for (const key of folderViews.keys())
    if (key.startsWith(id + "/")) folderViews.delete(key);
}
onResourceReset((id) => clearFileDrafts(id.replace(/\/files$/, "")));
const readAvailable = (s: Data) =>
  s.can_manage &&
  s.can_administer &&
  !["unprovisioned", "provisioning"].includes(s.observed);
const stopped = (s: Data) =>
  s.observed === "stopped" &&
  s.desired === "stopped" &&
  (!s.maintenance || s.inspection?.state === "ready");
function useLifetime(blocked = false) {
  const controller = useMemo(() => new AbortController(), [blocked]);
  useLayoutEffect(() => {
    if (blocked) controller.abort();
    return () => controller.abort();
  }, [controller, blocked]);
  return controller.signal;
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
function fileSize(value: unknown) {
  if (typeof value !== "number" || !Number.isFinite(value) || value < 0)
    return "—";
  const units = ["B", "KiB", "MiB", "GiB"];
  const index = Math.min(
    value < 1 ? 0 : Math.floor(Math.log(value) / Math.log(1024)),
    units.length - 1,
  );
  return `${new Intl.NumberFormat(getLocale(), { maximumFractionDigits: index ? 1 : 0 }).format(value / 1024 ** index)} ${units[index]}`;
}
function EntryIcon({ directory }: { directory: boolean }) {
  return (
    <svg
      className="file-entry-icon"
      viewBox="0 0 24 24"
      width="18"
      height="18"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
      aria-hidden="true"
      focusable="false"
    >
      {directory ? (
        <path d="M3 6a2 2 0 0 1 2-2h5l2 3h7a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2Z" />
      ) : (
        <>
          <path d="M14 3H6a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9Z" />
          <path d="M14 3v6h6" />
        </>
      )}
    </svg>
  );
}
type FolderView = {
  filter: string;
  sort: "name" | "size" | "modified";
  descending: boolean;
};
const folderViews = new PrivateCache<FolderView>(24);
export function Files({ server: s }: { server: Data }) {
  const { me, route, open, send, refresh } = useApp();
  const query = new URL(route.path, location.origin).searchParams;
  const path = query.get("path") ?? "";
  const file = query.get("file") || null;
  const creating = !file && query.get("new") === "1";
  const editing = !!file || creating;
  const viewKey = `${s.id}/${path}`;
  const cachedView = folderViews.get(viewKey);
  const [view, setView] = useState<{ key: string; value: FolderView }>({
    key: viewKey,
    value: cachedView ?? { filter: "", sort: "name", descending: false },
  });
  const currentView =
    view.key === viewKey
      ? view.value
      : (cachedView ?? {
          filter: "",
          sort: "name" as const,
          descending: false,
        });
  const updateView = (changes: Partial<FolderView>) => {
    const value = { ...currentView, ...changes };
    folderViews.set(viewKey, value);
    setView({ key: viewKey, value });
  };
  const [uploading, setUploading] = useState(false);
  const [uploadError, setUploadError] = useState<Error | null>(null);
  const [sessionBusy, setSessionBusy] = useState(false);
  const [sessionError, setSessionError] = useState<Error | null>(null);
  // A visit admits at most once automatically. Closing, expiry, a failed
  // attempt, or a revoked read must never turn a status poll into a new boot.
  const entryHandled = useRef(false);
  const sessionPending = useRef(false);
  const automaticRequest = useRef(crypto.randomUUID());
  const automaticJob = useRef<string | null>(null);
  const sessionAttempt = useRef({ opening: true, automatic: true });
  const status = hostingStatus(s);
  const fileReady =
    readAvailable(s) &&
    s.status?.actions?.files?.allowed === true &&
    (status.game_state === "running" ||
      (s.inspection?.state === "ready" && s.inspection.guest_ready === true));
  const read = useServerRead("server_files", { id: s.id, path }, fileReady);
  const writable =
    !!read.result &&
    !read.busy &&
    stopped(s) &&
    status.game_state === "stopped" &&
    fileReady &&
    !read.revoked;
  const signal = useLifetime(!!read.revoked);
  const links = useRef(new Map<string, HTMLAnchorElement>());
  const previousFile = useRef<string | null>(file);
  const deletedFile = useRef<string | null>(null);
  const filterInput = useRef<HTMLInputElement>(null);
  const uploadInput = useRef<HTMLInputElement>(null);
  const newMenu = useRef<HTMLDetailsElement>(null);
  useEffect(() => {
    const previous = previousFile.current;
    previousFile.current = file;
    if (previous && !editing && !read.revoked) {
      const deleted = deletedFile.current === previous;
      deletedFile.current = null;
      const frame = requestAnimationFrame(() =>
        (deleted
          ? filterInput.current
          : (links.current.get(previous) ?? filterInput.current)
        )?.focus(),
      );
      return () => cancelAnimationFrame(frame);
    }
  }, [file, editing, read.revoked]);
  useEffect(() => {
    if (read.revoked) {
      clearFileDrafts(s.id);
      setView({
        key: viewKey,
        value: { filter: "", sort: "name", descending: false },
      });
      setUploadError(null);
      setSessionError(null);
      setUploading(false);
      setSessionBusy(false);
    }
  }, [read.revoked, s.id, viewKey]);
  const href = (directory: string, selection?: string, fresh = false) => {
    const params = new URLSearchParams();
    if (directory) params.set("path", directory);
    if (selection) params.set("file", selection);
    if (fresh) params.set("new", "1");
    return `#/hosting/servers/${s.id}/files${params.size ? "?" + params.toString() : ""}`;
  };
  const closeHref = href(path);
  const parts = path.split("/").filter(Boolean);
  const entries: Data[] = (read.result?.entries ?? []).slice(0, ENTRY_LIMIT);
  const collator = new Intl.Collator(getLocale(), {
    numeric: true,
    sensitivity: "base",
  });
  const visibleEntries = entries
    .filter((entry) =>
      String(entry.name)
        .toLocaleLowerCase(getLocale())
        .includes(currentView.filter.toLocaleLowerCase(getLocale())),
    )
    .sort((a, b) => {
      if ((a.kind === "directory") !== (b.kind === "directory"))
        return a.kind === "directory" ? -1 : 1;
      const direction = currentView.descending ? -1 : 1;
      if (currentView.sort === "size")
        return (
          direction *
          ((a.bytes ?? -1) - (b.bytes ?? -1) ||
            collator.compare(a.name, b.name))
        );
      if (currentView.sort === "modified")
        return (
          direction *
          (String(a.modified_at ?? "").localeCompare(
            String(b.modified_at ?? ""),
          ) || collator.compare(a.name, b.name))
        );
      return direction * collator.compare(a.name, b.name);
    });
  async function filesSession(opening: boolean, automatic = false) {
    if (sessionPending.current || signal.aborted) return;
    entryHandled.current = true;
    sessionPending.current = true;
    sessionAttempt.current = { opening, automatic };
    setSessionBusy(true);
    setSessionError(null);
    try {
      const values = { id: s.id, open: opening };
      const result = automatic
        ? automaticJob.current
          ? { job_id: automaticJob.current }
          : await command("server_inspection", values, automaticRequest.current)
        : await send("server_inspection", values);
      if (
        automatic &&
        result.inspection?.actor &&
        result.inspection.actor !== me.account.id &&
        !me.account.administrator
      ) {
        // Another authorized operator won admission. The server projection
        // can expose readiness, but this account must not read their job.
        automaticJob.current = null;
        automaticRequest.current = crypto.randomUUID();
        if (!signal.aborted) refresh();
        return;
      }
      if (automatic && result.job_id) automaticJob.current = result.job_id;
      if (result.job_id)
        await waitForJob(
          result.job_id,
          (job) => {
            if (
              automatic &&
              ["succeeded", "failed", "cancelled", "delivery_unknown"].includes(
                job.state,
              )
            ) {
              automaticJob.current = null;
              automaticRequest.current = crypto.randomUUID();
            }
          },
          signal,
        );
      if (!signal.aborted) refresh();
    } catch (error) {
      if (!signal.aborted) setSessionError(error as Error);
    } finally {
      sessionPending.current = false;
      if (!signal.aborted) setSessionBusy(false);
    }
  }
  const projectedFileReason = s.status?.actions?.files?.reason;
  const admissionReason =
    !s.can_manage || !s.can_administer
      ? "permission_required"
      : projectedFileReason && projectedFileReason !== "files_closed"
        ? projectedFileReason
        : s.active_operation?.kind === "server.restore"
          ? "restore_in_progress"
          : s.status?.observation_fresh !== true
            ? "observation_stale"
            : !readAvailable(s)
              ? "provisioning"
              : s.maintenance && !s.inspection
                ? "maintenance"
                : status.game_state !== "stopped" || s.desired !== "stopped"
                  ? "game_not_ready"
                  : null;
  const canPrepare = !admissionReason && s.kind === "custom" && !s.inspection;
  useEffect(() => {
    if (read.revoked || s.inspection || fileReady) entryHandled.current = true;
    if (
      (fileReady && sessionAttempt.current.opening) ||
      (!s.inspection && !sessionAttempt.current.opening)
    )
      setSessionError(null);
    if (entryHandled.current || !canPrepare || signal.aborted) return;
    entryHandled.current = true;
    void filesSession(true, true);
  }, [s.id, canPrepare, fileReady, s.inspection, read.revoked, signal]);
  const applyArtifact = (artifact: Data, directory: string) =>
    open({
      title: message("text.apply_an_uploaded_file"),
      fields: [
        {
          name: "path",
          label: message("text.destination_in_server"),
          value: [directory, artifact.name].filter(Boolean).join("/"),
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
        if (signal.aborted || !writable) return;
        const result = await send("server_install", {
          id: s.id,
          artifact: artifact.id,
          path: v.path,
        });
        if (result.job_id) await waitForJob(result.job_id, undefined, signal);
        if (!signal.aborted) {
          read.refresh();
          refresh();
        }
      },
    });
  async function upload(file: File, input: HTMLInputElement) {
    if (!writable || uploading || signal.aborted) {
      input.value = "";
      return;
    }
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
      applyArtifact(
        { ...artifact, name: artifact.name ?? file.name },
        directory,
      );
    } catch (error) {
      if (!signal.aborted) setUploadError(error as Error);
    } finally {
      input.value = "";
      if (!signal.aborted) setUploading(false);
    }
  }
  function createFolder() {
    if (!writable || uploading) return;
    if (newMenu.current) newMenu.current.open = false;
    open({
      title: message("text.create_folder"),
      fields: [{ name: "name", label: message("text.folder_name"), max: 128 }],
      submit: message("text.create"),
      action: async (v) => {
        if (signal.aborted || !writable) return;
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
        if (result.job_id) await waitForJob(result.job_id, undefined, signal);
        if (!signal.aborted) read.refresh();
      },
    });
  }
  return (
    <section
      className="hosting-tool files-workspace"
      data-editing={editing && !read.revoked}
      aria-label={t("text.files")}
    >
      {s.inspection?.state === "ready" && (
        <div className="files-inspection">
          <small>
            {t("hosting.files.inspection_ready", {
              expires: date(s.inspection.expires_at),
            })}
          </small>
          <button
            disabled={sessionBusy}
            onClick={() => void filesSession(false)}
          >
            {t("text.close_files")}
          </button>
        </div>
      )}
      {!fileReady && !read.revoked && !sessionError && (
        <div className="files-inspection">
          {sessionBusy || s.inspection ? (
            <p role="status">
              {t("text.preparing_or_closing_files_your_draft_is_kept")}
            </p>
          ) : canPrepare && entryHandled.current ? (
            <button
              disabled={sessionBusy}
              onClick={() => void filesSession(true)}
            >
              {t("text.open_files")}
            </button>
          ) : s.kind !== "custom" && status.game_state === "stopped" ? (
            <p role="status">
              {t("text.file_inspection_is_only_available_on_custom_servers")}
            </p>
          ) : admissionReason ? (
            <p role="status">
              {t(
                hostingActionReasons[admissionReason] ??
                  "text.wait_for_minecraft_to_reach_a_confirmed_power_state",
              )}
            </p>
          ) : null}
        </div>
      )}
      {sessionError && (
        <p role="alert" className="error">
          {messageError(sessionError)}{" "}
          <button
            disabled={
              sessionBusy ||
              (sessionAttempt.current.opening
                ? !canPrepare && !automaticJob.current
                : !s.inspection)
            }
            onClick={() =>
              void filesSession(
                sessionAttempt.current.opening,
                sessionAttempt.current.automatic,
              )
            }
          >
            {t("text.retry")}
          </button>
        </p>
      )}
      <div className="files-layout">
        <div className="files-browser">
          <div className="files-toolbar">
            <nav
              className="file-breadcrumbs"
              aria-label={t("text.file_location")}
            >
              <a href={href("")} aria-current={!path ? "location" : undefined}>
                {t("text.server_root")}
              </a>
              {parts.map((part, i) => (
                <span key={i}>
                  <span aria-hidden="true"> / </span>
                  <a
                    href={href(parts.slice(0, i + 1).join("/"))}
                    aria-current={
                      i === parts.length - 1 ? "location" : undefined
                    }
                  >
                    {part}
                  </a>
                </span>
              ))}
            </nav>
            <div className="tool-toolbar">
              <details className="files-new-menu" ref={newMenu}>
                <summary
                  aria-disabled={!writable || uploading}
                  onClick={(event) => {
                    if (!writable || uploading) event.preventDefault();
                  }}
                >
                  {t("hosting.files.new")}
                </summary>
                <div className="files-new-options">
                  <a
                    href={href(path, undefined, true)}
                    aria-disabled={!writable || uploading}
                    onClick={(event) => {
                      if (!writable || uploading) event.preventDefault();
                      else if (newMenu.current) newMenu.current.open = false;
                    }}
                  >
                    {t("text.new_text_file")}
                  </a>
                  <button
                    disabled={!writable || uploading}
                    onClick={createFolder}
                  >
                    {t("text.create_folder")}
                  </button>
                </div>
              </details>
              <button
                disabled={!writable || uploading}
                onClick={() => uploadInput.current?.click()}
              >
                {uploading ? t("text.uploading") : t("hosting.files.upload")}
              </button>
              <input
                ref={uploadInput}
                type="file"
                hidden
                aria-label={t("hosting.files.upload")}
                disabled={!writable || uploading}
                onChange={(event) => {
                  const file = event.currentTarget.files?.[0];
                  if (file) void upload(file, event.currentTarget);
                }}
              />
              <button
                disabled={!fileReady || read.busy || read.revoked}
                onClick={read.refresh}
              >
                {t("text.refresh")}
              </button>
            </div>
          </div>
          {status.game_state === "running" && (
            <p className="notice">
              {t(
                "text.stop_this_custom_server_before_saving_uploading_creatin_68b9378f0d",
              )}
            </p>
          )}
          {!readAvailable(s) ? (
            <p>
              {t(
                "text.files_and_logs_are_available_after_server_creation_completes",
              )}
            </p>
          ) : (
            (fileReady || read.revoked) && <ReadState read={read} />
          )}
          {["unknown", "starting", "stopping"].includes(status.game_state) && (
            <p className="notice">
              {t("text.wait_for_minecraft_to_reach_a_confirmed_power_state")}
            </p>
          )}
          <div className="files-controls">
            <label className="field">
              {t("hosting.files.filter")}
              <input
                ref={filterInput}
                type="search"
                value={currentView.filter}
                onChange={(event) => updateView({ filter: event.target.value })}
              />
            </label>
            <label className="field">
              {t("hosting.files.sort")}
              <select
                value={currentView.sort}
                onChange={(event) =>
                  updateView({ sort: event.target.value as FolderView["sort"] })
                }
              >
                <option value="name">{t("hosting.files.sort_name")}</option>
                <option value="size">{t("hosting.files.sort_size")}</option>
                <option value="modified">
                  {t("hosting.files.sort_modified")}
                </option>
              </select>
            </label>
            <button
              aria-label={t("hosting.files.sort_direction")}
              aria-pressed={currentView.descending}
              onClick={() =>
                updateView({ descending: !currentView.descending })
              }
            >
              {currentView.descending
                ? t("hosting.files.descending")
                : t("hosting.files.ascending")}
            </button>
          </div>
          {read.result && (
            <>
              <table className="files-table">
                <caption>
                  {t("text.directory_entries")}: {path || t("text.server_root")}
                </caption>
                <thead>
                  <tr>
                    <th scope="col">{t("text.name")}</th>
                    <th scope="col">{t("hosting.files.size")}</th>
                    <th scope="col" className="file-modified">
                      {t("hosting.files.modified")}
                    </th>
                  </tr>
                </thead>
                <tbody>
                  {path && (
                    <tr className="linked-row">
                      <td>
                        <a
                          className="file-link row-link"
                          href={href(parts.slice(0, -1).join("/"))}
                        >
                          <EntryIcon directory />
                          {t("text.parent_folder")}
                        </a>
                      </td>
                      <td>—</td>
                      <td className="file-modified">—</td>
                    </tr>
                  )}
                  {visibleEntries.map((entry) => (
                    <tr
                      className="linked-row"
                      key={entry.path}
                      data-selected={file === entry.path || undefined}
                    >
                      <td>
                        <a
                          className="file-link row-link"
                          aria-label={entry.name}
                          href={
                            entry.kind === "directory"
                              ? href(entry.path)
                              : href(path, entry.path)
                          }
                          ref={(element) => {
                            if (element) links.current.set(entry.path, element);
                            else links.current.delete(entry.path);
                          }}
                          aria-current={
                            file === entry.path ? "true" : undefined
                          }
                        >
                          <EntryIcon directory={entry.kind === "directory"} />
                          <span className="file-entry-text">
                            <span>{entry.name}</span>
                            <small className="file-entry-meta">
                              {entry.modified_at
                                ? date(entry.modified_at)
                                : "—"}
                            </small>
                          </span>
                        </a>
                      </td>
                      <td>
                        {entry.kind === "directory"
                          ? t("hosting.files.directory")
                          : fileSize(entry.bytes)}
                      </td>
                      <td className="file-modified">
                        {entry.modified_at ? date(entry.modified_at) : "—"}
                      </td>
                    </tr>
                  ))}
                  {!visibleEntries.length && (
                    <tr>
                      <td colSpan={3}>
                        {entries.length
                          ? t("hosting.files.no_matches")
                          : t("text.this_folder_is_empty")}
                      </td>
                    </tr>
                  )}
                </tbody>
              </table>
              <div className="files-footer">
                <small>
                  {t("hosting.files.entry_count", {
                    count: visibleEntries.length,
                  })}
                </small>
                {(read.result.truncated ||
                  read.result.entries?.length > ENTRY_LIMIT) && (
                  <p role="status">
                    {t("hosting.files.entry_limit", { limit: ENTRY_LIMIT })}
                  </p>
                )}
              </div>
            </>
          )}
          {uploadError && (
            <p className="error" role="alert">
              {messageError(uploadError)}
            </p>
          )}
          {s.artifacts?.length > 0 && (
            <details className="files-artifacts">
              <summary>{t("text.uploaded_files")}</summary>
              <p>
                {t(
                  "text.these_uploads_can_be_applied_into_the_selected_folder_t_9fafced573",
                )}
              </p>
              {s.artifacts.map((artifact: Data) => (
                <div className="list-row" key={artifact.id}>
                  <span className="grow">
                    <strong>{artifact.name}</strong>{" "}
                    <small>{fileSize(artifact.bytes)}</small>
                  </span>
                  <button
                    disabled={!writable || uploading}
                    onClick={() => applyArtifact(artifact, path)}
                  >
                    {t("text.apply")}
                  </button>
                </div>
              ))}
            </details>
          )}
        </div>
        {!read.revoked && editing && (
          <FileEditor
            key={file ? "file:" + file : "new:" + path}
            server={s}
            readable={fileReady}
            path={file}
            directory={path}
            writable={writable}
            onChanged={read.refresh}
            onDeleted={() => {
              deletedFile.current = file;
            }}
            closeHref={closeHref}
          />
        )}
      </div>
    </section>
  );
}
function FileEditor({
  server: s,
  readable,
  path,
  directory,
  writable,
  onChanged,
  onDeleted,
  closeHref,
}: {
  server: Data;
  readable: boolean;
  path: string | null;
  directory: string;
  writable: boolean;
  onChanged: () => void;
  onDeleted: () => void;
  closeHref: string;
}) {
  const { send, open } = useApp();
  const textInputId = useId();
  const key = path ? `${s.id}/file/${path}` : `${s.id}/new/${directory}`;
  const saved = fileDrafts.get(key);
  const [text, setText] = useState(saved?.text ?? "");
  const [sha, setSha] = useState<string | null>(
    path ? (saved?.sha ?? null) : null,
  );
  const [name, setName] = useState(saved?.name ?? "");
  const [initialized, setInitialized] = useState(!!saved || !path);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<Error | null>(null);
  const [outcome, setOutcome] = useState<SystemMessage | null>(
    saved?.outcome ?? null,
  );
  const heading = useRef<HTMLHeadingElement>(null);
  useLayoutEffect(() => {
    heading.current?.focus();
  }, []);
  const read = useServerRead(
    "server_file_read",
    { id: s.id, path: path ?? "" },
    !!path && readable,
  );
  const signal = useLifetime(!!read.revoked);
  useEffect(() => {
    if (read.revoked) {
      fileDrafts.delete(key);
      setBusy(false);
      setText("");
      setSha(null);
      setName("");
      setInitialized(false);
      setError(null);
      setOutcome(null);
    }
  }, [read.revoked, key]);
  useEffect(() => {
    if (read.result && !initialized) {
      setText(read.result.text);
      setSha(read.result.sha256);
      setInitialized(true);
    }
  }, [read.result, initialized]);
  const dirty = path
    ? sha !== read.result?.sha256 || text !== read.result?.text
    : !!text || !!name;
  const destination = path ?? [directory, name].filter(Boolean).join("/");
  const canSave =
    writable &&
    !busy &&
    !read.busy &&
    !read.revoked &&
    initialized &&
    (path ? !!sha && !!read.result : !!name.trim());
  async function save() {
    if (!canSave || signal.aborted) return;
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
      const savedOutcome = message("text.saved_0", destination);
      fileDrafts.set(key, {
        text,
        sha: final.sha256,
        name,
        outcome: savedOutcome,
      });
      setOutcome(savedOutcome);
      onChanged();
      read.refresh();
      if (!path) {
        fileDrafts.delete(key);
        fileDrafts.set(`${s.id}/file/${destination}`, {
          text,
          sha: final.sha256,
          outcome: savedOutcome,
        });
        const params = new URLSearchParams();
        if (directory) params.set("path", directory);
        params.set("file", destination);
        location.hash = `/hosting/servers/${s.id}/files?${params}`;
      }
    } catch (error) {
      if (!signal.aborted) setError(error as Error);
    } finally {
      if (!signal.aborted) setBusy(false);
    }
  }
  return (
    <section className="file-editor" aria-label={t("text.text_editor")}>
      <div className="files-editor-toolbar">
        <a
          href={closeHref}
          aria-disabled={busy}
          onClick={(event) => {
            if (busy) event.preventDefault();
          }}
        >
          {t("hosting.files.back")}
        </a>
        <h2 ref={heading} tabIndex={-1}>
          {path ?? t("text.new_text_file")}
        </h2>
        {path && (
          <button
            disabled={!readable || read.busy || busy || read.revoked}
            onClick={read.refresh}
          >
            {t("text.read_current_file_version")}
          </button>
        )}
      </div>
      {path && <ReadState read={read} />}
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
                  onChange={(event) => {
                    setName(event.target.value);
                    fileDrafts.set(key, {
                      text,
                      sha,
                      name: event.target.value,
                    });
                  }}
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
            <div className="field">
              <label htmlFor={textInputId}>{t("text.file_text")}</label>
              <textarea
                id={textInputId}
                className="code-editor"
                value={text}
                maxLength={65536}
                rows={18}
                disabled={busy}
                onChange={(event) => {
                  setText(event.target.value);
                  fileDrafts.set(key, { text: event.target.value, sha, name });
                }}
              />
            </div>
            <div className="files-editor-toolbar">
              <button
                className="primary"
                disabled={!canSave}
                onClick={() => void save()}
              >
                {busy ? t("text.saving_23e39291") : t("text.save_file")}
              </button>
              {path && dirty && (
                <button
                  disabled={busy || read.busy || !read.result || read.revoked}
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
                        if (signal.aborted || !read.result) return;
                        setText(read.result.text);
                        setSha(read.result.sha256);
                        fileDrafts.delete(key);
                        setError(null);
                        setOutcome(null);
                      },
                    })
                  }
                >
                  {t("text.discard_local_edits")}
                </button>
              )}
              {path && (
                <button
                  className="danger"
                  disabled={
                    !writable ||
                    busy ||
                    !sha ||
                    !read.result ||
                    read.busy ||
                    read.revoked
                  }
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
                        if (signal.aborted || !writable) return;
                        const result = await send("server_file_delete", {
                          id: s.id,
                          path,
                          expected_sha256: sha,
                        });
                        if (result.job_id)
                          await waitForJob(result.job_id, undefined, signal);
                        if (signal.aborted) return;
                        fileDrafts.delete(key);
                        onDeleted();
                        onChanged();
                        location.hash = closeHref.slice(1);
                      },
                    })
                  }
                >
                  {t("text.delete_file")}
                </button>
              )}
            </div>
            <small>
              {t(
                "text.bounded_utf_8_text_only_saving_checks_the_original_file_version",
              )}
            </small>
          </>
        )
      )}
      {error && (
        <p role="alert" className="error">
          {messageError(error)}
        </p>
      )}
      {outcome && <p role="status">{renderSystemMessage(outcome)}</p>}
      {read.result && (
        <details className="file-current-version">
          <summary>{t("text.version_last_read_from_server")}</summary>
          <pre className="output" tabIndex={0}>
            {read.result.text}
          </pre>
        </details>
      )}
    </section>
  );
}
