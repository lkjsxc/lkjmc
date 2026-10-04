import {
  identityEpoch,
  subscribeIdentity,
  assertIdentity,
  unreadable,
  resetResource,
} from "./identity";
import { flushSync } from "react-dom";
import {
  t,
  useLanguage,
  setLanguage,
  languages,
  getLanguage,
  translateError,
  message,
  messageError,
  renderSystemMessage,
  type SystemMessage,
} from "./i18n";
import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useRef,
  useState,
  type ReactNode,
} from "react";
import {
  api,
  jobTitle,
  readJob,
  ApiError,
  command,
  date,
  type Me,
  type Data,
} from "./api";
import { Icon, Modal, ActionForm, Status, type Field } from "./ui";
import { Timeline } from "./timeline";
import {
  PlayHub,
  Worlds,
  WorldOverview,
  WorldToolNavigation,
  PeopleNavigation,
  Expeditions,
} from "./playerViews";
import { ServerTools } from "./serverTools";
import {
  JobDetail,
  JobResponse,
  jobTarget,
  noticeTitle,
  noticeLink,
  terminal,
} from "./jobs";
import {
  resolveRoute,
  normalize,
  topPages,
  childPages,
  type Route,
} from "./routes";
import { ManagedList, CreateServer, AdminHome, PageNavigation } from "./pages";
import { Inbox, Social, Life, Market, Settings, Admin } from "./views";

type DialogSpec = {
  title: string | SystemMessage;
  fields?: Field[];
  type?: string;
  values?: Data;
  submit?: string | SystemMessage;
  note?: ReactNode | (() => ReactNode);
  action?: (data: Data) => Promise<unknown>;
};
type Context = {
  me: Me;
  route: Route;
  send: (type: string, values?: Data) => Promise<Data>;
  act: (type: string, values?: Data) => void;
  open: (spec: DialogSpec) => void;
  refresh: () => void;
  go: (page: string) => void;
  showJob: (id: string, hint?: Data) => void;
  showNotice: (notice: Data) => void;
  jobs: Data[];
  isWorking: (type: string, values?: Data) => boolean;
  panelsVisible: boolean;
};
const AppContext = createContext<Context | null>(null);
export const useApp = () => useContext(AppContext)!;
export function PageBlock({
  id,
  children,
}: {
  id: string | string[];
  children: ReactNode;
}) {
  const { route } = useApp();
  const ids = Array.isArray(id) ? id : [id];
  return ids.includes(route.section) ? <>{children}</> : null;
}

export function App() {
  useLanguage();
  const [epoch, setEpoch] = useState(identityEpoch);
  const [me, setMe] = useState<Me | null>(null);
  const [loading, setLoading] = useState(true);
  const [fatal, setFatal] = useState<unknown>(null);
  useEffect(
    () =>
      subscribeIdentity(() =>
        flushSync(() => {
          setEpoch(identityEpoch());
          setMe(null);
        }),
      ),
    [],
  );
  useEffect(() => {
    let alive = true,
      running = false;
    async function sync() {
      if (running || document.hidden) return;
      running = true;
      const language = getLanguage();
      try {
        const value = await api<Me>("/api/v1/me");
        if (alive) {
          setMe(value);
          setFatal("");
          if (language === getLanguage())
            setLanguage(value.account.language ?? "en");
        }
      } catch (e) {
        if (alive && !(e instanceof ApiError && e.status === 401)) setFatal(e);
      } finally {
        running = false;
        if (alive) setLoading(false);
      }
    }
    void sync();
    const timer = setInterval(sync, 15000);
    window.addEventListener("focus", sync);
    return () => {
      alive = false;
      clearInterval(timer);
      window.removeEventListener("focus", sync);
    };
  }, []);
  if (loading)
    return (
      <div className="full-state">
        <p>{t("text.checking_connection")}</p>
      </div>
    );
  if (!me) return <Landing error={fatal ? messageError(fatal) : ""} />;
  return <SessionApp key={epoch} me={me} setMe={setMe} />;
}
function SessionApp({ me, setMe }: { me: Me; setMe: (value: Me) => void }) {
  const language = useLanguage();
  const pages = topPages();
  const [page, setPage] = useState(
    normalize(location.hash.slice(1) || "/play"),
  );
  const route = resolveRoute(page);
  const menuButton = useRef<HTMLButtonElement>(null);
  const sidebar = useRef<HTMLElement>(null);
  const [compact, setCompact] = useState(
    () => window.matchMedia("(max-width: 850px)").matches,
  );
  const [menu, setMenu] = useState(false);
  const [loadedData, setData] = useState<Data | null>(null);
  const [dataPath, setDataPath] = useState("");
  const data = dataPath === page ? loadedData : null;
  const [error, setError] = useState<unknown>(null);
  const [revision, setRevision] = useState(0);
  const [dialog, setDialog] = useState<DialogSpec | null>(null);
  const [toast, setToast] = useState<string | SystemMessage>("");
  const [jobs, setJobs] = useState<Data[]>([]);
  const jobsRef = useRef(jobs);
  jobsRef.current = jobs;
  const [jobDetail, setJobDetail] = useState<{
    id: string;
    hint?: Data;
  } | null>(null);
  const [noticeDetail, setNoticeDetail] = useState<Data | null>(null);
  const [toastDetail, setToastDetail] = useState<Data | null>(null);
  const actionRequests = useRef(new Map<string, Promise<Data>>());
  const [pendingActions, setPendingActions] = useState<string[]>([]);
  const [actionErrors, setActionErrors] = useState<Data[]>([]);
  const serial = useRef(0);
  const refresh = useCallback(() => setRevision((n) => n + 1), []);
  useEffect(() => {
    if (location.hash.slice(1) !== page)
      history.replaceState(null, "", "#" + page);
  }, [page]);
  useEffect(() => {
    const change = () => {
      setPage(normalize(location.hash.slice(1) || "/play"));
      setMenu(false);
    };
    window.addEventListener("hashchange", change);
    return () => window.removeEventListener("hashchange", change);
  }, []);
  useEffect(() => {
    setData(null);
    setError("");
  }, [page]);
  useEffect(() => {
    if (!me) return;
    let alive = true;
    const seq = ++serial.current;
    const controller = new AbortController();
    let running = false;
    let timer: ReturnType<typeof setTimeout>;
    const load = async () => {
      if (running || !alive) return;
      if (document.hidden) {
        timer = setTimeout(load, 15000);
        return;
      }
      running = true;
      try {
        const v = route.api
          ? await api(route.api, { signal: controller.signal })
          : {};
        if (alive && seq === serial.current) {
          if (v.server && !v.server.can_manage) resetResource(v.server.id);
          else if (v.server && !v.server.can_administer)
            resetResource(v.server.id + "/files");
          if (
            v.server &&
            route.component === "managed-server" &&
            (!v.server.can_manage ||
              (!v.server.can_administer &&
                [
                  "manage-files",
                  "manage-backups",
                  "manage-members",
                  "manage-settings",
                ].includes(route.section)))
          ) {
            setDialog(null);
            setJobDetail(null);
            setNoticeDetail(null);
            setToast("");
            setToastDetail(null);
            setJobs([]);
            setActionErrors([]);
          }
          setData(v);
          setDataPath(page);
          if (route.id === "official" && v.server?.id) {
            const canonical = route.path.replace(
              "/servers/official",
              "/worlds/" + v.server.id,
            );
            history.replaceState(null, "", "#" + canonical);
            setPage(canonical);
          }
          setError("");
        }
      } catch (e) {
        if (alive) {
          if (unreadable(e)) {
            if (route.id) resetResource(route.id);
            setData(null);
            setDialog(null);
            setJobDetail(null);
            setNoticeDetail(null);
            setToast("");
            setToastDetail(null);
            setJobs([]);
            setActionErrors([]);
          }
          setError(e);
        }
      } finally {
        running = false;
        if (alive && route.api) timer = setTimeout(load, 15000);
      }
    };
    void load();
    return () => {
      alive = false;
      clearTimeout(timer);
      controller.abort();
    };
  }, [me?.account.id, page, revision]);
  useEffect(() => {
    let alive = true;
    let timer: ReturnType<typeof setTimeout>;
    const controller = new AbortController();
    async function poll() {
      if (!alive) return;
      if (!document.hidden) {
        const active = jobsRef.current.filter(
          (j) =>
            !terminal(j.state) &&
            (j.origin === page || j.server_id === route.id) &&
            j.id !== jobDetail?.id,
        );
        await Promise.allSettled(
          active.map(async (job) => {
            try {
              const v = await readJob(job.id);
              if (!alive) return;
              const next = { ...job, ...v };
              setJobs((all) => all.map((j) => (j.id === job.id ? next : j)));
              if (terminal(v.state)) {
                setToast(
                  `${jobTitle(next)}${jobTarget(next) ? " · " + jobTarget(next) : ""}: ${v.state === "succeeded" ? t("text.completed") : v.state === "cancelled" ? t("text.cancelled") : t("text.failed")}`,
                );
                setToastDetail({ job_id: next.id, hint: next });
                refresh();
              }
            } catch (error) {
              if (alive && unreadable(error))
                setJobs((all) => all.filter((j) => j.id !== job.id));
              /* Detail view offers an explicit retry; background reads stay quiet. */
            }
          }),
        );
      }
      if (alive) timer = setTimeout(poll, 2500);
    }
    timer = setTimeout(poll, 2500);
    return () => {
      alive = false;
      clearTimeout(timer);
      controller.abort();
    };
  }, [page, jobDetail?.id, refresh]);
  useEffect(() => {
    if (!toast) return;
    const timer = setTimeout(() => setToast(""), 8000);
    return () => clearTimeout(timer);
  }, [toast]);
  useEffect(() => {
    const media = window.matchMedia("(max-width: 850px)");
    const change = () => {
      setCompact(media.matches);
      setMenu(false);
    };
    media.addEventListener("change", change);
    return () => media.removeEventListener("change", change);
  }, []);
  useEffect(() => {
    if (!menu || !compact) return;
    const previous = document.activeElement as HTMLElement | null;
    const overflow = document.body.style.overflow;
    document.body.style.overflow = "hidden";
    const elements = () =>
      Array.from(
        sidebar.current?.querySelectorAll<HTMLElement>(
          "a[href], button, select",
        ) ?? [],
      );
    elements()[0]?.focus();
    const key = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        setMenu(false);
        event.preventDefault();
      }
      if (event.key === "Tab") {
        const nodes = elements();
        const first = nodes[0];
        const last = nodes[nodes.length - 1];
        if (event.shiftKey && document.activeElement === first) {
          last?.focus();
          event.preventDefault();
        } else if (!event.shiftKey && document.activeElement === last) {
          first?.focus();
          event.preventDefault();
        }
      }
    };
    document.addEventListener("keydown", key);
    return () => {
      document.body.style.overflow = overflow;
      document.removeEventListener("keydown", key);
      previous?.focus();
    };
  }, [menu, compact]);
  function send(type: string, values: Data = {}): Promise<Data> {
    const epoch = identityEpoch();
    const origin = location.hash;
    const operationKey = JSON.stringify([type, values]);
    const existing = actionRequests.current.get(operationKey);
    if (existing) return existing;
    const queued = jobsRef.current.find(
      (j) => j.operationKey === operationKey && !terminal(j.state),
    );
    if (queued) return Promise.resolve({ job_id: queued.id });
    setPendingActions((all) => [...all, operationKey]);
    const request = (async () => {
      const result = await command(type, values);
      assertIdentity(epoch);
      const server =
        data?.server ?? data?.servers?.find((s: Data) => s.id === values.id);
      const target =
        server?.name ?? values.name ?? values.path ?? values.target ?? "";
      const hint = {
        operationKey,
        id: result.job_id,
        kind: type,
        state: "queued",
        server_id:
          server?.id ?? (type.startsWith("server_") ? values.id : undefined),
        target_name: target,
        member_id: values.member,
        operator: values.operator,
        open: values.open,
        origin: page,
      };
      const named = `${jobTitle(hint)}${target ? " · " + target : ""}`;
      if (result.job_id) {
        setJobs((j) =>
          [hint, ...j.filter((v) => v.id !== result.job_id)].slice(0, 30),
        );
        if (location.hash === origin)
          setToast(
            t("text.0_request_accepted_open_details_to_follow_progress", named),
          );
        if (location.hash === origin)
          setToastDetail({ job_id: result.job_id, hint });
      } else {
        if (location.hash === origin) setToast(t("text.0_saved", named));
        if (location.hash === origin)
          setToastDetail({ kind: "action_result", title: named, body: result });
      }
      if (type === "language" || type === "privacy") {
        const value = await api<Me>("/api/v1/me");
        setMe(value);
        setLanguage(value.account.language ?? "en");
      }
      refresh();
      if (location.hash !== origin)
        throw new DOMException("Page changed", "AbortError");
      return result;
    })()
      .catch((error) => {
        if (
          epoch === identityEpoch() &&
          unreadable(error) &&
          values.id === route.id &&
          location.hash === origin
        ) {
          resetResource(values.id);
          setData(null);
          setError(error);
          setDialog(null);
          setJobDetail(null);
          setNoticeDetail(null);
          setJobs([]);
          setToast("");
          setToastDetail(null);
        }
        throw error;
      })
      .finally(() => {
        actionRequests.current.delete(operationKey);
        setPendingActions((all) => all.filter((key) => key !== operationKey));
      });
    actionRequests.current.set(operationKey, request);
    return request;
  }
  function act(type: string, values: Data = {}) {
    const origin = location.hash;
    const epoch = identityEpoch();
    void send(type, values).catch((e) => {
      if (epoch !== identityEpoch() || location.hash !== origin) return;
      const server =
        data?.servers?.find((s: Data) => s.id === values.id) ?? data?.server;
      const named = `${jobTitle({ kind: type })}${server?.name ? " · " + server.name : ""}`;
      setActionErrors((all) => [
        ...all.filter((v) => v.operation !== type || v.target !== values.id),
        {
          operation: type,
          target: values.id,
          title: named,
          error: e,
          origin: page,
        },
      ]);
      setToast(
        message("text.the_action_needs_attention_open_details_to_review_it"),
      );
      setToastDetail({
        kind: "action_result",
        title: named,
        body: { error: e.systemMessage ?? e },
      });
    });
  }
  function go(page: string) {
    location.hash = normalize(page);
  }
  const routeJobs = jobs.filter(
    (job) =>
      !terminal(job.state) &&
      (job.origin === page || job.server_id === route.id),
  );
  const current = {
    name:
      ["world", "managed-server"].includes(route.component) &&
      ["overview", "manage-overview"].includes(route.section) &&
      data?.server?.name
        ? data.server.name
        : route.area === "people" &&
            route.section === "team" &&
            data?.team?.name
          ? data.team.name
          : t(route.title),
    description: t(route.description),
  };
  useEffect(() => {
    document.title = current.name + " · lkjmc";
  }, [current.name]);
  useEffect(() => {
    document.getElementById("page-title")?.focus({ preventScroll: true });
  }, [route.path.split("?")[0]]);
  const section = route.area === "expeditions" ? "worlds" : route.area;
  const available = pages.filter(
    (p) => p.id !== "admin" || me.account.administrator,
  );
  const context: Context = {
    me,
    route,
    send,
    act,
    open: setDialog,
    refresh,
    go,
    jobs,
    isWorking: (type, values = {}) => {
      const key = JSON.stringify([type, values]);
      return (
        pendingActions.includes(key) ||
        jobs.some((j) => j.operationKey === key && !terminal(j.state))
      );
    },
    panelsVisible: !dialog && !jobDetail && !noticeDetail && !menu,
    showJob: (id, hint) => setJobDetail({ id, hint }),
    showNotice: setNoticeDetail,
  };
  const components: Record<string, ReactNode> = {
    "play-hub": <PlayHub data={data ?? {}} />,
    worlds: <Worlds data={data ?? {}} />,
    world: <WorldOverview data={data ?? {}} />,
    expeditions: <Expeditions data={data ?? {}} />,
    feed: <Inbox data={data ?? {}} />,

    social: <Social data={data ?? {}} />,
    life: (
      <>
        <WorldToolNavigation />
        <Life data={data ?? {}} />
      </>
    ),
    market: (
      <>
        <WorldToolNavigation />
        <Market data={data ?? {}} />
      </>
    ),

    "managed-list": <ManagedList data={data ?? {}} />,
    "create-server": <CreateServer data={data ?? {}} />,
    "managed-server": <ServerTools data={data ?? {}} />,
    timeline: <Timeline />,
    settings: <Settings data={data ?? {}} />,
    admin: <Admin data={data ?? {}} />,
    "admin-home": <AdminHome data={data ?? {}} />,
  };
  const children = childPages(route, data?.server, data?.team);
  return (
    <AppContext.Provider value={context}>
      <a
        className="skip-link"
        href="#main"
        onClick={(event) => {
          event.preventDefault();
          document.getElementById("main")?.focus();
        }}
      >
        {t("text.skip_to_content")}
      </a>
      <div className="app-shell">
        {compact && menu && (
          <button
            className="menu-backdrop"
            tabIndex={-1}
            aria-label={t("text.close_menu")}
            onClick={() => setMenu(false)}
          />
        )}
        <aside
          id="primary-navigation"
          ref={sidebar}
          inert={compact && !menu}
          className={menu ? "sidebar open" : "sidebar"}
        >
          {compact && (
            <button className="drawer-close" onClick={() => setMenu(false)}>
              {t("text.close_menu")} ×
            </button>
          )}

          <a className="wordmark" href="#/play">
            <span className="brand-symbol">◈</span> lkjmc
          </a>
          <p className="side-caption">{t("text.a_place_to_play_together")}</p>
          <nav aria-label={t("text.main_menu")}>
            {available
              .filter((p) =>
                ["play", "worlds", "people", "timeline"].includes(p.id),
              )
              .map(({ id, name, path, icon }) => (
                <a
                  key={id}
                  href={"#" + path}
                  className={section === id ? "active" : ""}
                  aria-current={section === id ? "page" : undefined}
                >
                  <Icon name={icon} />
                  <span>{name}</span>
                  {section === id && <span className="nav-dot" />}
                </a>
              ))}
          </nav>
          <nav
            className="workspace-navigation"
            aria-label={t("text.workspaces")}
          >
            {available
              .filter((p) => ["hosting", "admin"].includes(p.id))
              .map((p) => (
                <a
                  key={p.id}
                  href={"#" + p.path}
                  className={section === p.id ? "active" : ""}
                  aria-current={section === p.id ? "page" : undefined}
                >
                  <Icon name={p.icon} />
                  <span>{p.name}</span>
                </a>
              ))}
          </nav>
          <a
            className="sidebar-foot"
            href="#/account"
            aria-label={t("text.account_settings_for_0", me.account.name)}
            aria-current={route.area === "account" ? "page" : undefined}
          >
            <span className="avatar">{me.account.name?.slice(0, 1)}</span>
            <div>
              <strong>{me.account.name}</strong>
              <small>
                {me.account.rank.name_message
                  ? renderSystemMessage(me.account.rank.name_message)
                  : me.account.rank.name}
              </small>
            </div>
          </a>
        </aside>
        <div className="workspace">
          <header className="topbar">
            <div className="topbar-left">
              <button
                className="icon-button mobile-menu"
                ref={menuButton}
                aria-controls="primary-navigation"
                onClick={() => setMenu(!menu)}
                aria-label={t("text.open_menu")}
                aria-expanded={menu}
              >
                <Icon name="menu" />
              </button>
              <span>{current.name}</span>
            </div>
            <button
              className="connection"
              disabled={!me.game_address}
              onClick={() =>
                navigator.clipboard
                  .writeText(me.game_address ?? "")
                  .then(() => {
                    setToastDetail(null);
                    setToast(message("text.server_address_copied"));
                  })
                  .catch(() => {
                    setToastDetail(null);
                    setToast(me.game_address ?? "");
                  })
              }
            >
              <span className="connection-dot" />
              {me.game_address ?? t("text.connection_unavailable")}
              <span className="copy-label">{t("text.copy")}</span>
            </button>
          </header>
          <main id="main" tabIndex={-1}>
            {route.path.split("?")[0] !== "/" + route.area && (
              <nav className="breadcrumbs" aria-label={t("text.breadcrumbs")}>
                <a
                  href={
                    "#" +
                    (route.area === "hosting"
                      ? "/hosting/servers"
                      : "/" + route.area)
                  }
                >
                  {pages.find((p) => p.id === route.area)?.name ??
                    t("text.account")}
                </a>
                {route.id && ["worlds", "hosting"].includes(route.area) && (
                  <>
                    <span aria-hidden="true">/</span>
                    <a
                      href={
                        "#" +
                        (route.area === "hosting"
                          ? "/hosting/servers/"
                          : "/worlds/") +
                        route.id
                      }
                    >
                      {data?.server?.name ?? t("text.server")}
                    </a>
                  </>
                )}
                <span aria-hidden="true">/</span>
                <span aria-current="page">{current.name}</span>
              </nav>
            )}
            <div className="page-heading">
              <div>
                <p className="eyebrow">{current.description}</p>
                <h1 tabIndex={-1} id="page-title">
                  {current.name}
                </h1>
              </div>
            </div>
            {route.area === "people" && <PeopleNavigation />}
            {children.length > 0 && (
              <nav className="section-nav" aria-label={t("text.page_menu")}>
                {children.map((child) => (
                  <a
                    key={child.path}
                    href={"#" + child.path}
                    aria-current={
                      route.path.split("?")[0] === child.path
                        ? "page"
                        : undefined
                    }
                  >
                    {child.name}
                  </a>
                ))}
              </nav>
            )}
            {me.development && (
              <div className="dev-banner">
                {t(
                  "text.development_environment_public_server_access_requires_a_c0eff5d651",
                )}
              </div>
            )}
            {actionErrors
              .filter((v) => v.origin === page)
              .map((v) => (
                <div
                  className="error"
                  role="alert"
                  key={v.operation + v.target}
                >
                  {jobTitle({ kind: v.operation })}: {messageError(v.error)}{" "}
                  <button
                    onClick={() =>
                      setActionErrors((all) => all.filter((e) => e !== v))
                    }
                  >
                    {t("text.dismiss_error")}
                  </button>
                </div>
              ))}
            {!!error && (
              <div className="error" role="alert">
                {messageError(error)}
                {data && " " + t("text.previously_loaded_data_is_still_shown")}
                <button onClick={refresh}>{t("text.reload")}</button>
              </div>
            )}
            {!data && !error ? (
              <div className="loading" role="status">
                {t("text.loading")}
              </div>
            ) : data ? (
              <div
                key={
                  route.component === "timeline"
                    ? "timeline"
                    : route.path.split("?")[0]
                }
              >
                {components[route.component] ?? (
                  <p>{t("text.this_page_could_not_be_found")}</p>
                )}
              </div>
            ) : null}
            <PageNavigation data={data ?? {}} />
            {routeJobs.length > 0 && (
              <section
                className="route-progress"
                aria-label={t("text.action_progress")}
              >
                {routeJobs.slice(0, 3).map((j) => (
                  <div key={j.id} className="list-row">
                    <div className="grow">
                      <strong>
                        {jobTitle(j)}
                        {jobTarget(j) ? " · " + jobTarget(j) : ""}
                      </strong>
                      {j.progress?.message && (
                        <small>{translateError(j.progress.message)}</small>
                      )}
                      {j.error && (
                        <p className="error" role="alert">
                          {translateError(j.error)}
                        </p>
                      )}
                    </div>
                    <Status value={j.state} />
                    <button onClick={() => setJobDetail({ id: j.id, hint: j })}>
                      {t("text.view_details")}
                    </button>
                  </div>
                ))}
              </section>
            )}
          </main>
        </div>
      </div>
      <nav
        className="mobile-navigation"
        aria-label={t("text.quick_navigation")}
      >
        {available
          .filter((p) =>
            ["play", "worlds", "people", "timeline"].includes(p.id),
          )
          .map((p) => (
            <a
              key={p.id}
              href={"#" + p.path}
              aria-current={section === p.id ? "page" : undefined}
            >
              <Icon name={p.icon} />
              <span>{p.name}</span>
            </a>
          ))}
      </nav>
      {toast && (
        <div role="status" className="toast">
          <span>
            {typeof toast === "string" ? toast : renderSystemMessage(toast)}
          </span>
          {toastDetail && (
            <button
              onClick={() =>
                toastDetail.job_id
                  ? setJobDetail({
                      id: toastDetail.job_id,
                      hint: toastDetail.hint,
                    })
                  : setNoticeDetail(toastDetail)
              }
            >
              {t("text.view_details")}
            </button>
          )}
          <button
            aria-label={t("text.dismiss_notification")}
            onClick={() => setToast("")}
          >
            ×
          </button>
        </div>
      )}
      {jobDetail && (
        <JobDetail
          key={jobDetail.id}
          id={jobDetail.id}
          hint={jobDetail.hint}
          onClose={() => setJobDetail(null)}
        />
      )}
      {noticeDetail && (
        <Modal
          title={noticeDetail.title ?? noticeTitle(noticeDetail)}
          onClose={() => setNoticeDetail(null)}
        >
          {noticeDetail.created_at && <p>{date(noticeDetail.created_at)}</p>}
          {noticeDetail.body && (
            <JobResponse
              result={
                typeof noticeDetail.body === "object"
                  ? noticeDetail.body
                  : { message: noticeDetail.body }
              }
            />
          )}
          {noticeLink(noticeDetail) && (
            <a
              href={"#" + noticeLink(noticeDetail)}
              onClick={() => setNoticeDetail(null)}
            >
              {t("text.open")}
            </a>
          )}
        </Modal>
      )}
      {dialog && (
        <Modal
          title={
            typeof dialog.title === "string"
              ? dialog.title
              : renderSystemMessage(dialog.title)
          }
          onClose={() => setDialog(null)}
        >
          {dialog.note && (
            <div className="modal-note">
              {typeof dialog.note === "function" ? dialog.note() : dialog.note}
            </div>
          )}
          <ActionForm
            fields={dialog.fields ?? []}
            submit={dialog.submit ?? message("text.confirm")}
            onSubmit={async (values) => {
              await (dialog.action
                ? dialog.action(values)
                : send(dialog.type!, { ...dialog.values, ...values }));
              setDialog((current) => (current === dialog ? null : current));
            }}
          />
        </Modal>
      )}
    </AppContext.Provider>
  );
}
function Landing({ error }: { error: string }) {
  const [ready, setReady] = useState<boolean | null>(null);
  const [address, setAddress] = useState("");
  useEffect(() => {
    let alive = true;
    api("/health/ready")
      .then((value) => {
        if (alive) {
          setReady(value.login_configured);
          setAddress(value.game_address ?? "");
        }
      })
      .catch(() => {
        if (alive) setReady(false);
      });
    return () => {
      alive = false;
    };
  }, []);
  return (
    <div className="landing">
      <header>
        <a className="wordmark" href="#/play">
          <span className="brand-symbol">◈</span> lkjmc
        </a>
        <LanguagePicker />
      </header>
      <main>
        <div>
          <span className="pill">{t("text.a_place_to_play_together")}</span>
          <h1>{t("text.a_world_is_better_with_people")}</h1>
          <p className="lead">
            {t(
              "text.build_a_home_find_your_people_set_off_on_an_adventure_a_8275765dc1",
            )}
          </p>
          <div className="landing-actions">
            {ready ? (
              <a className="button primary" href="/auth/login">
                {t("text.join_the_community")}
                <Icon name="arrow" />
              </a>
            ) : (
              <p className="notice">
                {ready === null
                  ? t("text.checking_connection")
                  : t("text.web_sign_in_is_being_set_up")}
              </p>
            )}
            {address && (
              <div>
                <small>{t("text.minecraft_address")}</small>
                <code>{address}</code>
                <small>Java · Bedrock</small>
              </div>
            )}
          </div>
          {error && (
            <p role="alert" className="error">
              {error}
            </p>
          )}
        </div>
        <div className="world-art world-art-official" aria-hidden="true">
          <span className="world-orbit" />
          <span className="block block-one" />
          <span className="block block-two" />
          <span className="block block-three" />
        </div>
      </main>
    </div>
  );
}

export function LanguagePicker({
  save,
}: {
  save?: (value: string) => Promise<unknown>;
}) {
  const language = useLanguage();
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<unknown>(null);
  return (
    <div className="language-picker">
      <label>
        <span>{t("text.language")}</span>
        <select
          aria-label={t("text.language")}
          value={language}
          disabled={busy}
          onChange={async (event) => {
            const next = event.target.value;
            setBusy(true);
            setError("");
            try {
              if (save) await save(next);
              setLanguage(next);
            } catch (failure) {
              setError(failure);
            } finally {
              setBusy(false);
            }
          }}
        >
          {languages.map((entry) => (
            <option key={entry.code} value={entry.code}>
              {entry.name}
            </option>
          ))}
        </select>
      </label>
      {!!error && <p role="alert">{messageError(error)}</p>}
    </div>
  );
}
