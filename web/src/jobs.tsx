import { unreadable } from "./identity";
import { useEffect, useState } from "react";
import { readJob, date, jobTitle, type Data } from "./api";
import { t, translateError } from "./i18n";
import { useApp } from "./App";
import { Modal, Status } from "./ui";

export const terminal = (state?: string) =>
  ["succeeded", "failed", "cancelled"].includes(state ?? "");
export const jobTarget = (job: Data) =>
  job.target_name || job.server_name || job.server_id || "";
export function JobResponse({ result }: { result: Data }) {
  return (
    <div className="job-response">
      {result.lines && (
        <pre className="output">
          {result.lines.join("\n") || t("No output returned.")}
        </pre>
      )}
      {result.preview_hash && (
        <>
          <p>
            {result.clear
              ? t("Ready to place.")
              : t("Something is in the way. Clear the area and try again.")}
          </p>
          <dl className="details-list">
            <div>
              <dt>{t("Preview confirmation")}</dt>
              <dd>
                <code>{result.preview_hash}</code>
              </dd>
            </div>
          </dl>
        </>
      )}
      {result.message && <p>{result.message}</p>}
      {result.error && (
        <p role="alert" className="error">
          {translateError(result.error)}
        </p>
      )}
      {result.path && (
        <p>
          {t("File")}: <code>{result.path}</code>
        </p>
      )}
      {result.effect === "committed" && (
        <p>{t("The change was saved on the server.")}</p>
      )}
      {result.effective === "next_start" && (
        <p>
          {result.operator
            ? t("Operator grant saved; effective on next start.")
            : t("Operator removal saved; effective on next start.")}
        </p>
      )}
      {result.transferred === true && <p>{t("Transfer completed.")}</p>}
      {Object.keys(result).filter((key) => key !== "lines").length > 0 && (
        <details>
          <summary>{t("Full response")}</summary>
          <pre>
            {JSON.stringify(
              Object.fromEntries(
                Object.entries(result).filter(([key]) => key !== "lines"),
              ),
              null,
              2,
            )}
          </pre>
        </details>
      )}
      {!Object.keys(result).length && (
        <p>{t("The operation completed without additional output.")}</p>
      )}
    </div>
  );
}
export function JobDetail(props: {
  id: string;
  hint?: Data;
  onClose: () => void;
}) {
  return <ScopedJobDetail key={props.id} {...props} />;
}
function ScopedJobDetail({
  id,
  hint,
  onClose,
}: {
  id: string;
  hint?: Data;
  onClose: () => void;
}) {
  const [job, setJob] = useState<Data | null>(null);
  const [error, setError] = useState("");
  const [revision, setRevision] = useState(0);
  useEffect(() => {
    let alive = true,
      running = false;
    let timer: ReturnType<typeof setTimeout>;
    const controller = new AbortController();
    async function load() {
      if (running || !alive) return;
      if (document.hidden) {
        timer = setTimeout(load, 2500);
        return;
      }
      running = true;
      try {
        const next = await readJob(id, controller.signal);
        if (alive) {
          setJob((v) => ({ ...hint, ...v, ...next }));
          setError("");
        }
        if (alive && !terminal(next.state)) timer = setTimeout(load, 2500);
      } catch (e) {
        if (alive) {
          if (unreadable(e)) setJob(null);
          setError((e as Error).message);
        }
      } finally {
        running = false;
      }
    }
    void load();
    return () => {
      alive = false;
      clearTimeout(timer);
      controller.abort();
    };
  }, [id, revision]);
  const titleJob = job ?? (error ? {} : hint) ?? {};
  return (
    <Modal
      title={`${jobTitle(titleJob)}${jobTarget(titleJob) ? " · " + jobTarget(titleJob) : ""}`}
      onClose={onClose}
    >
      {error && (
        <p className="error" role="alert">
          {error} {job && t("Previously loaded data is still shown.")}{" "}
          <button onClick={() => setRevision((v) => v + 1)}>
            {t("Retry")}
          </button>
        </p>
      )}
      {!job ? (
        !error && <p role="status">{t("Loading…")}</p>
      ) : (
        <>
          <Status value={job.state} />
          {job.updated_at && (
            <p>
              <small>
                {t("Last updated")}: {date(job.updated_at)}
              </small>
            </p>
          )}
          {job.progress?.message && (
            <p role="status">{translateError(job.progress.message)}</p>
          )}
          {typeof job.progress?.percent === "number" && (
            <progress
              aria-label={t("Progress")}
              max={100}
              value={job.progress.percent}
            />
          )}
          {job.progress &&
            Object.keys(job.progress).some(
              (k) => !["message", "percent"].includes(k),
            ) && <pre>{JSON.stringify(job.progress, null, 2)}</pre>}
          {job.error && (
            <p className="error" role="alert">
              {translateError(
                typeof job.error === "string"
                  ? job.error
                  : JSON.stringify(job.error),
              )}
            </p>
          )}
          {job.result && <JobResponse result={job.result} />}
          {!terminal(job.state) && (
            <p>
              {t(
                "This action is still in progress. You can close this window and check it from Timeline.",
              )}
            </p>
          )}
          {job.server_id && (
            <a href={`#/servers/${job.server_id}`} onClick={onClose}>
              {t("Open server")}
            </a>
          )}
        </>
      )}
    </Modal>
  );
}
const noticeNames: Record<string, string> = {
  invitation: "New invitation",
  invitation_response: "Invitation response",
  friend_request: "Friend request",
  friend_response: "Friend request response",
  message: "New message",
  transfer: "Coins received",
  market_sale: "Listing sold",
  achievement: "Achievement unlocked",
  job_finished: "Action completed",
  link_candidate: "Account linking confirmation",
};
export function noticeTitle(notice: Data) {
  const body = notice.body ?? {};
  const action =
    body.job_kind ??
    body.operation ??
    (notice.kind === "job_finished" ? body.kind : undefined);
  const name = action
    ? jobTitle({ kind: action })
    : t(noticeNames[notice.kind] ?? "New update");
  const target =
    body.server_name ??
    body.room_name ??
    body.target_name ??
    body.name ??
    body.server_id;
  return target ? `${name} · ${target}` : name;
}
export const noticeJobId = (notice: Data) =>
  notice.body?.job_id ??
  (notice.kind === "job_finished" ? notice.body?.id : undefined);
export function noticeLink(notice: Data): string | undefined {
  const body = notice.body ?? {};
  if (body.room_id || body.room)
    return "/timeline?room=" + encodeURIComponent(body.room_id ?? body.room);
  if (body.server_id) return "/servers/" + body.server_id;
  if (notice.kind.startsWith("invitation")) return "/home/invitations";
  if (notice.kind === "friend_request") return "/friends/incoming";
  if (notice.kind === "friend_response") return "/friends";
  if (notice.kind === "link_candidate") return "/account/linking";
}
export function NotificationItem({ notice }: { notice: Data }) {
  const { showJob, showNotice } = useApp();
  const link = noticeLink(notice);
  return (
    <div className="list-row">
      <div className="grow">
        <strong>{noticeTitle(notice)}</strong>
        <small>{date(notice.created_at)}</small>
        {notice.body?.state && <Status value={notice.body.state} />}
        {notice.body?.amount != null && (
          <p>
            {notice.body.amount} {t(" coins")}
          </p>
        )}
      </div>
      {!notice.read_at && (
        <span className="unread-dot" aria-label={t("Unread")} />
      )}
      <div className="actions">
        {link && <a href={"#" + link}>{t("Open")}</a>}
        <button
          onClick={() =>
            noticeJobId(notice)
              ? showJob(noticeJobId(notice), {
                  kind: notice.body.job_kind ?? notice.body.kind,
                  server_name: notice.body.server_name,
                  server_id: notice.body.server_id,
                })
              : showNotice(notice)
          }
        >
          {t("View details")}
        </button>
      </div>
    </div>
  );
}
