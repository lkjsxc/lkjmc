import { unreadable } from "./identity";
import { useEffect, useState } from "react";
import { readJob, date, jobTitle, type Data } from "./api";
import { t, messageError, renderSystemMessage, translateError } from "./i18n";
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
          {result.lines.join("\n") || t("text.no_output_returned")}
        </pre>
      )}
      {result.preview_hash && (
        <>
          <p>
            {result.clear
              ? t("text.ready_to_place")
              : t("text.something_is_in_the_way_clear_the_area_and_try_again")}
          </p>
          <dl className="details-list">
            <div>
              <dt>{t("text.preview_confirmation")}</dt>
              <dd>
                <code>{result.preview_hash}</code>
              </dd>
            </div>
          </dl>
        </>
      )}
      {result.message && <p>{renderSystemMessage(result.message)}</p>}
      {result.error && (
        <p role="alert" className="error">
          {translateError(result.error)}
        </p>
      )}
      {result.path && (
        <p>
          {t("text.file")}: <code>{result.path}</code>
        </p>
      )}
      {result.effect === "committed" && (result.path || result.effective) && (
        <p>{t("text.the_change_was_saved_on_the_server")}</p>
      )}
      {result.effective === "next_start" && (
        <p>
          {result.operator
            ? t("text.operator_grant_saved_effective_on_next_start")
            : t("text.operator_removal_saved_effective_on_next_start")}
        </p>
      )}
      {result.effect === "committed" &&
        result.session_id &&
        result.actual_server_id &&
        result.actual_server_id === result.server_id && (
          <p>{t("text.transfer_completed")}</p>
        )}
      {Object.keys(result).filter((key) => key !== "lines").length > 0 && (
        <details>
          <summary>{t("text.full_response")}</summary>
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
        <p>{t("text.the_operation_completed_without_additional_output")}</p>
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
  const [error, setError] = useState<unknown>(null);
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
          setError(e);
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
      {!!error && (
        <p className="error" role="alert">
          {messageError(error)}{" "}
          {job && t("text.previously_loaded_data_is_still_shown")}{" "}
          <button onClick={() => setRevision((v) => v + 1)}>
            {t("text.retry")}
          </button>
        </p>
      )}
      {!job ? (
        !error && <p role="status">{t("text.loading")}</p>
      ) : (
        <>
          <Status value={job.state} />
          {job.updated_at && (
            <p>
              <small>
                {t("text.last_updated")}: {date(job.updated_at)}
              </small>
            </p>
          )}
          {job.progress?.message && (
            <p role="status">{translateError(job.progress.message)}</p>
          )}
          {typeof job.progress?.percent === "number" && (
            <progress
              aria-label={t("text.progress")}
              max={100}
              value={job.progress.percent}
            />
          )}
          {job.progress &&
            Object.keys(job.progress).some(
              (k) => !["message", "percent"].includes(k),
            ) && (
              <details>
                <summary>{t("text.progress")}</summary>
                <pre>{JSON.stringify(job.progress, null, 2)}</pre>
              </details>
            )}
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
                "text.this_action_is_still_in_progress_you_can_close_this_win_190d80423d",
              )}
            </p>
          )}
          {job.server_id && (
            <a href={`#/worlds/${job.server_id}`} onClick={onClose}>
              {t("text.open_server")}
            </a>
          )}
        </>
      )}
    </Modal>
  );
}
const noticeNames: Record<string, string> = {
  invitation: "text.new_invitation",
  invitation_response: "text.invitation_response",
  friend_request: "text.friend_request",
  friend_response: "text.friend_request_response",
  message: "text.new_message",
  transfer: "text.coins_received",
  market_sale: "text.listing_sold",
  achievement: "text.achievement_unlocked",
  job_finished: "text.action_completed",
  link_candidate: "text.account_linking_confirmation",
};
export function noticeTitle(notice: Data) {
  const body = notice.body ?? {};
  const action =
    body.job_kind ??
    body.operation ??
    (notice.kind === "job_finished" ? body.kind : undefined);
  const name = action
    ? jobTitle({ kind: action, open: body.open })
    : t(noticeNames[notice.kind] ?? "text.new_update");
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
  if (body.server_id) return "/worlds/" + body.server_id;
  if (notice.kind.startsWith("invitation")) return "/play/invitations";
  if (notice.kind === "friend_request") return "/people/friends/incoming";
  if (notice.kind === "friend_response") return "/people/friends";
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
            {notice.body.amount} {t("text.coins")}
          </p>
        )}
      </div>
      {!notice.read_at && (
        <span className="unread-dot" aria-label={t("text.unread")} />
      )}
      <div className="actions">
        {link && <a href={"#" + link}>{t("text.open")}</a>}
        <button
          onClick={() =>
            noticeJobId(notice)
              ? showJob(noticeJobId(notice), {
                  kind: notice.body.job_kind ?? notice.body.kind,
                  open: notice.body.open,
                  server_name: notice.body.server_name,
                  server_id: notice.body.server_id,
                })
              : showNotice(notice)
          }
        >
          {t("text.view_details")}
        </button>
      </div>
    </div>
  );
}
