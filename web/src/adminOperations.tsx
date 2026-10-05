import { useApp } from "./App";
import { date, jobTitle, money, type Data } from "./api";
import { t, renderSystemMessage } from "./i18n";
import { Card, Empty, Status } from "./ui";

const filters = [
  ["active", "text.operations_in_progress", "text.no_operations_in_progress"],
  [
    "failed",
    "text.operations_failed_or_uncertain",
    "text.no_failed_or_uncertain_operations",
  ],
  ["history", "text.operations_history", "text.no_operation_history"],
] as const;

export function AdminOperations({ data }: { data: Data }) {
  const { showJob, route, refresh } = useApp();
  const current =
    filters.find(([value]) => value === data.filter) ?? filters[0];
  const base = "#/admin/operations?filter=" + current[0];
  const operations: Data[] = data.operations ?? [];
  return (
    <Card title={t("text.operations")}>
      <div className="section-toolbar">
        <nav className="pagination" aria-label={t("text.operations")}>
          {filters.map(([value, title]) => (
            <a
              key={value}
              href={"#/admin/operations?filter=" + value}
              aria-current={value === current[0] ? "page" : undefined}
            >
              {t(title)} · {money(data.counts?.[value] ?? 0)}
            </a>
          ))}
        </nav>
        <button onClick={refresh}>{t("text.refresh")}</button>
      </div>
      {operations.length ? (
        <div className="list">
          {operations.map((operation) => {
            const detail = operation.error ?? operation.progress?.message;
            return (
              <article className="list-row" key={operation.id}>
                <div>
                  <strong>{jobTitle(operation)}</strong>
                  {operation.server_name && <p>{operation.server_name}</p>}
                  {detail != null && <p>{renderSystemMessage(detail)}</p>}
                  <small>{date(operation.updated_at)}</small>
                </div>
                <div className="actions">
                  <Status value={operation.state} />
                  <button onClick={() => showJob(operation.id, operation)}>
                    {t("text.view_details")}
                  </button>
                </div>
              </article>
            );
          })}
        </div>
      ) : (
        <Empty>{t(current[2])}</Empty>
      )}
      {(route.cursor || data.next_cursor) && (
        <nav className="pagination" aria-label={t("text.history_pages")}>
          {route.cursor && <a href={base}>{t("text.latest")}</a>}
          {data.next_cursor && (
            <a href={base + "&cursor=" + encodeURIComponent(data.next_cursor)}>
              {t("text.older")}
            </a>
          )}
        </nav>
      )}
    </Card>
  );
}
