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
import {
  ServerInfo,
  ManagedList,
  CreateServer,
  AdminHome,
  PageNavigation,
} from "./pages";
import {
  Home,
  Play,
  Smp,
  Social,
  Life,
  Market,
  Adventure,
  Settings,
  Admin,
} from "./views";

type DialogSpec = {
  title: string;
  fields?: Field[];
  type?: string;
  values?: Data;
  submit?: string;
  note?: ReactNode;
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
  const [epoch, setEpoch] = useState(identityEpoch);
  const [me, setMe] = useState<Me | null>(null);
  const [loading, setLoading] = useState(true);
  const [fatal, setFatal] = useState("");
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
        if (alive && !(e instanceof ApiError && e.status === 401))
          setFatal((e as Error).message);
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
        <p>{t("Checking connection…")}</p>
      </div>
    );
  if (!me) return <Landing error={fatal} />;
  return <SessionApp key={epoch} me={me} setMe={setMe} />;
}
function SessionApp({ me, setMe }: { me: Me; setMe: (value: Me) => void }) {
  const language = useLanguage();
  const pages = topPages();
  const [page, setPage] = useState(normalize(location.hash.slice(1) || "home"));
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
  const [error, setError] = useState("");
  const [revision, setRevision] = useState(0);
  const [dialog, setDialog] = useState<DialogSpec | null>(null);
  const [toast, setToast] = useState("");
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
      setPage(normalize(location.hash.slice(1) || "home"));
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
              "/servers/" + v.server.id,
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
          setError((e as Error).message);
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
                  `${jobTitle(next)}${jobTarget(next) ? " · " + jobTarget(next) : ""}: ${v.state === "succeeded" ? t("Completed") : v.state === "cancelled" ? t("Cancelled") : t("Failed")}`,
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
        origin: page,
      };
      const named = `${jobTitle(hint)}${target ? " · " + target : ""}`;
      if (result.job_id) {
        setJobs((j) =>
          [hint, ...j.filter((v) => v.id !== result.job_id)].slice(0, 30),
        );
        if (location.hash === origin)
          setToast(
            t("{0}: request accepted. Open details to follow progress.", named),
          );
        if (location.hash === origin)
          setToastDetail({ job_id: result.job_id, hint });
      } else {
        if (location.hash === origin) setToast(t("{0}: saved.", named));
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
          setError(error.message);
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
          error: e.message,
          origin: page,
        },
      ]);
      setToast(`${named}: ${e.message}`);
      setToastDetail({
        kind: "action_result",
        title: named,
        body: { error: e.message },
      });
    });
  }
  function go(page: string) {
    location.hash = normalize(page);
  }
  const current = {
    name:
      route.area === "teams" && route.section === "team" && data?.team?.name
        ? data.team.name
        : t(route.title),
    description: t(route.description),
  };
  const section = route.area;
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
    home: <Home data={data ?? {}} />,
    feed: <Home data={data ?? {}} />,
    play: <Play data={data ?? {}} />,
    server: <ServerInfo data={data ?? {}} />,
    social: <Social data={data ?? {}} />,
    life: <Life data={data ?? {}} />,
    market: <Market data={data ?? {}} />,
    adventure: <Adventure data={data ?? {}} />,
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
        {t("Skip to content")}
      </a>
      <div className="app-shell">
        {compact && menu && (
          <button
            className="menu-backdrop"
            tabIndex={-1}
            aria-label={t("Close menu")}
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
              {t("Close menu")} ×
            </button>
          )}

          <nav aria-label={t("Main menu")}>
            {available.map(({ id, name, path, icon }) => (
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
          <a
            className="sidebar-foot"
            href="#/account"
            aria-label={t("Account settings for {0}", me.account.name)}
            aria-current={route.area === "account" ? "page" : undefined}
          >
            <span className="avatar">{me.account.name?.slice(0, 1)}</span>
            <div>
              <strong>{me.account.name}</strong>
              <small>{me.account.rank.name}</small>
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
                aria-label={t("Open menu")}
                aria-expanded={menu}
              >
                <Icon name="menu" />
              </button>
              <span>{current.name}</span>
            </div>
            <button
              className="connection"
              onClick={() =>
                navigator.clipboard
                  .writeText(me.game_address)
                  .then(() => {
                    setToastDetail(null);
                    setToast(t("Server address copied."));
                  })
                  .catch(() => {
                    setToastDetail(null);
                    setToast(me.game_address);
                  })
              }
            >
              <span className="connection-dot" />
              {me.game_address}
              <span className="copy-label">{t("Copy")}</span>
            </button>
          </header>
          <main id="main" tabIndex={-1}>
            {route.path.split("?")[0] !== "/" + route.area && (
              <nav className="breadcrumbs" aria-label={t("Breadcrumbs")}>
                <a
                  href={
                    "#" +
                    (route.area === "manage"
                      ? "/manage/servers"
                      : "/" + route.area)
                  }
                >
                  {pages.find((p) => p.id === route.area)?.name ?? t("Account")}
                </a>
                {route.id && ["servers", "manage"].includes(route.area) && (
                  <>
                    <span aria-hidden="true">/</span>
                    <a
                      href={
                        "#" +
                        (route.area === "manage"
                          ? "/manage/servers/"
                          : "/servers/") +
                        route.id
                      }
                    >
                      {data?.server?.name ?? t("Server")}
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
                <h1>{current.name}</h1>
              </div>
            </div>
            {children.length > 0 && (
              <nav className="section-nav" aria-label={t("Page menu")}>
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
                  "Development environment — public server access requires a separate check.",
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
                  {v.title}: {v.error}{" "}
                  <button
                    onClick={() =>
                      setActionErrors((all) => all.filter((e) => e !== v))
                    }
                  >
                    {t("Dismiss error")}
                  </button>
                </div>
              ))}
            {error && (
              <div className="error" role="alert">
                {error}
                {data && " " + t("Previously loaded data is still shown.")}
                <button onClick={refresh}>{t("Reload")}</button>
              </div>
            )}
            {!data && !error ? (
              <div className="loading" role="status">
                {t("Loading…")}
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
                  <p>{t("This page could not be found.")}</p>
                )}
              </div>
            ) : null}
            <PageNavigation data={data ?? {}} />
            {jobs.filter((j) => j.origin === page || j.server_id === route.id)
              .length > 0 && (
              <section
                className="route-progress"
                aria-label={t("Action progress")}
              >
                {jobs
                  .filter((j) => j.origin === page || j.server_id === route.id)
                  .slice(0, 3)
                  .map((j) => (
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
                      <button
                        onClick={() => setJobDetail({ id: j.id, hint: j })}
                      >
                        {t("View details")}
                      </button>
                    </div>
                  ))}
              </section>
            )}
          </main>
        </div>
      </div>
      {toast && (
        <div role="status" className="toast">
          <span>{toast}</span>
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
              {t("View details")}
            </button>
          )}
          <button
            aria-label={t("Dismiss notification")}
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
              {t("Open")}
            </a>
          )}
        </Modal>
      )}
      {dialog && (
        <Modal title={dialog.title} onClose={() => setDialog(null)}>
          {dialog.note && <div className="modal-note">{dialog.note}</div>}
          <ActionForm
            fields={dialog.fields ?? []}
            submit={dialog.submit ?? t("Confirm")}
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
  useEffect(() => {
    api("/health/ready")
      .then((v) => setReady(v.login_configured))
      .catch(() => setReady(false));
  }, []);
  return (
    <div className="landing">
      <header>
        <LanguagePicker />
      </header>
      <main>
        <h1>{t("Minecraft community")}</h1>
        <p className="lead">
          {t(
            "Join servers, keep in touch with friends, and trade land and buildings.",
          )}
          <br />
          {t("Create an account or sign in to get started.")}
        </p>
        <div className="landing-actions">
          {ready ? (
            <a className="button primary" href="/auth/login">
              {t("Sign up / Sign in")}
              <Icon name="arrow" />
            </a>
          ) : (
            <p className="notice">
              {ready === null
                ? t("Checking connection…")
                : t("Web sign-in is being set up.")}
            </p>
          )}
          <div>
            <small>{t("Server address")}</small>
            <code>lkjsxc.com:25591</code>
            <small>Java / Bedrock</small>
          </div>
        </div>
        {error && (
          <p role="alert" className="error">
            {error}
          </p>
        )}
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
  const [error, setError] = useState("");
  return (
    <div className="language-picker">
      <label>
        <span>{t("Language")}</span>
        <select
          aria-label={t("Language")}
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
              setError(
                failure instanceof Error
                  ? failure.message
                  : t("Could not save language."),
              );
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
      {error && <p role="alert">{error}</p>}
    </div>
  );
}
