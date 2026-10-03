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
  ApiError,
  command,
  setCsrf,
  date,
  type Me,
  type Data,
} from "./api";
import { Icon, Modal, ActionForm, Status, type Field } from "./ui";
import {
  Home,
  Play,
  Smp,
  Social,
  Life,
  Market,
  Adventure,
  Servers,
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
  send: (type: string, values?: Data) => Promise<Data>;
  act: (type: string, values?: Data) => void;
  open: (spec: DialogSpec) => void;
  refresh: () => void;
  go: (page: string) => void;
};
const AppContext = createContext<Context | null>(null);
export const useApp = () => useContext(AppContext)!;
const getPages = () => [
  ["home", t("Home"), t("Invitations and activity")],
  ["play", t("Servers"), t("Status and connection details")],
  ["smp", "SMP", t("Status, joining and in-game tools")],
  ["social", t("Friends & chat"), t("People and groups")],
  ["life", t("Land & assets"), t("Claims, homes and balance")],
  ["market", t("Market"), t("Buy and sell")],
  ["adventure", t("Private End"), t("Create and join a private world")],
  ["servers", t("Manage servers"), t("Settings, files and backups")],
  ["settings", t("Account"), t("Linked accounts and privacy")],
  ["admin", t("Administration"), t("Access, reports and service status")],
];
const smpPages = ["smp", "life", "market", "adventure"];

export function App() {
  const language = useLanguage();
  const pages = getPages();
  const [me, setMe] = useState<Me | null>(null);
  const [loading, setLoading] = useState(true);
  const [fatal, setFatal] = useState("");
  const [page, setPage] = useState(location.hash.slice(1) || "home");
  const menuButton = useRef<HTMLButtonElement>(null);
  const sidebar = useRef<HTMLElement>(null);
  const [compact, setCompact] = useState(
    () => window.matchMedia("(max-width: 850px)").matches,
  );
  const [menu, setMenu] = useState(false);
  const [data, setData] = useState<Data | null>(null);
  const [error, setError] = useState("");
  const [revision, setRevision] = useState(0);
  const [dialog, setDialog] = useState<DialogSpec | null>(null);
  const [toast, setToast] = useState("");
  const [jobs, setJobs] = useState<Data[]>([]);
  const serial = useRef(0);
  const refresh = useCallback(() => setRevision((n) => n + 1), []);
  useEffect(() => {
    api<Me>("/api/v1/me")
      .then((v) => {
        setMe(v);
        setCsrf(v.csrf);
        setLanguage(v.account.language ?? "en");
      })
      .catch((e) => {
        if (!(e instanceof ApiError && e.status === 401)) setFatal(e.message);
      })
      .finally(() => setLoading(false));
  }, []);
  useEffect(() => {
    const change = () => {
      setPage(location.hash.slice(1) || "home");
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
    const load = () =>
      api(`/api/v1/view/${page === "smp" ? "play" : page}`)
        .then((v) => {
          if (alive && seq === serial.current) {
            setData(v);
            setError("");
          }
        })
        .catch((e) => {
          if (alive) setError(e.message);
        });
    void load();
    const timer = setInterval(load, 8000);
    return () => {
      alive = false;
      clearInterval(timer);
    };
  }, [me, page, revision]);
  useEffect(() => {
    if (
      !jobs.some((j) => !["succeeded", "failed", "cancelled"].includes(j.state))
    )
      return;
    const timer = setInterval(() => {
      for (const job of jobs.filter(
        (j) => !["succeeded", "failed", "cancelled"].includes(j.state),
      ))
        api(`/api/v1/jobs/${job.id}`)
          .then((v) => {
            setJobs((all) => all.map((j) => (j.id === v.id ? v : j)));
            if (["succeeded", "failed"].includes(v.state)) refresh();
          })
          .catch((e) => setToast(e.message));
    }, 2500);
    return () => clearInterval(timer);
  }, [jobs, refresh]);
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
  useEffect(() => {
    if (!me) return;
    let alive = true;
    const sync = () => {
      const before = getLanguage();
      return api<Me>("/api/v1/me")
        .then((value) => {
          if (alive && before === getLanguage()) {
            setMe(value);
            setLanguage(value.account.language ?? "en");
          }
        })
        .catch(() => {});
    };
    const timer = setInterval(sync, 15000);
    window.addEventListener("focus", sync);
    return () => {
      alive = false;
      clearInterval(timer);
      window.removeEventListener("focus", sync);
    };
  }, [me?.account.id]);
  async function send(type: string, values: Data = {}) {
    const result = await command(type, values);
    if (result.job_id) {
      setJobs((j) =>
        [
          { id: result.job_id, kind: type, state: "queued" },
          ...j.filter((v) => v.id !== result.job_id),
        ].slice(0, 10),
      );
      setToast(t("Request accepted. Follow its progress below."));
    } else setToast(t("Saved."));
    if (type === "language" || type === "privacy") {
      const value = await api<Me>("/api/v1/me");
      setMe(value);
      setLanguage(value.account.language ?? "en");
    }
    refresh();
    return result;
  }
  function act(type: string, values: Data = {}) {
    void send(type, values).catch((e) => setToast(e.message));
  }
  function go(page: string) {
    location.hash = page;
  }
  if (loading)
    return (
      <div className="full-state">
        <div className="wordmark">
          lkjmc<span>●</span>
        </div>
        <p>{t("Checking connection…")}</p>
      </div>
    );
  if (!me) return <Landing error={fatal} />;
  const current = pages.find((p) => p[0] === page) ?? pages[0];
  const inSmp = smpPages.includes(page);
  const section = inSmp ? "play" : page;
  const available = pages.filter(
    (p) =>
      !smpPages.includes(p[0]) &&
      (p[0] !== "admin" || me.account.administrator),
  );
  const context: Context = { me, send, act, open: setDialog, refresh, go };
  const components: Record<string, ReactNode> = {
    home: <Home data={data ?? {}} />,
    play: <Play data={data ?? {}} />,
    smp: <Smp data={data ?? {}} />,
    social: <Social data={data ?? {}} />,
    life: <Life data={data ?? {}} />,
    market: <Market data={data ?? {}} />,
    adventure: <Adventure data={data ?? {}} />,
    servers: <Servers data={data ?? {}} />,
    settings: <Settings data={data ?? {}} />,
    admin: <Admin data={data ?? {}} />,
  };
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
          <a className="wordmark" href="#home">
            lkjmc<span>●</span>
          </a>
          <p className="side-caption">{t("Minecraft community")}</p>
          <nav aria-label={t("Main menu")}>
            {available.map(([id, name]) => (
              <a
                key={id}
                href={`#${id}`}
                className={section === id ? "active" : ""}
                aria-current={section === id ? "page" : undefined}
              >
                <Icon name={id} />
                <span>{name}</span>
                {section === id && <span className="nav-dot" />}
              </a>
            ))}
          </nav>
          <LanguagePicker
            save={(value) => send("language", { language: value })}
          />
          <div className="sidebar-foot">
            <span className="avatar">{me.account.name?.slice(0, 1)}</span>
            <div>
              <strong>{me.account.name}</strong>
              <small>{me.account.rank.name}</small>
            </div>
          </div>
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
              <span>lkjmc / {inSmp ? "SMP" : current[1]}</span>
            </div>
            <button
              className="connection"
              onClick={() =>
                navigator.clipboard
                  .writeText(me.game_address)
                  .then(() => setToast(t("Server address copied.")))
                  .catch(() => setToast(me.game_address))
              }
            >
              <span className="connection-dot" />
              {me.game_address}
              <span className="copy-label">{t("Copy")}</span>
            </button>
          </header>
          <main id="main" tabIndex={-1}>
            {inSmp && (
              <nav className="breadcrumbs" aria-label={t("Breadcrumbs")}>
                <a href="#play">{t("Servers")}</a>
                <span aria-hidden="true">/</span>
                {page === "smp" ? (
                  <span aria-current="page">SMP</span>
                ) : (
                  <>
                    <a href="#smp">SMP</a>
                    <span aria-hidden="true">/</span>
                    <span aria-current="page">{current[1]}</span>
                  </>
                )}
              </nav>
            )}
            <div className="page-heading">
              <div>
                <p className="eyebrow">{current[2]}</p>
                <h1>{current[1]}</h1>
              </div>
              <button className="quiet" onClick={refresh}>
                {t("Refresh")}
              </button>
            </div>
            {inSmp && (
              <nav className="section-nav" aria-label={t("SMP menu")}>
                {smpPages.map((id) => (
                  <a
                    key={id}
                    href={`#${id}`}
                    aria-current={page === id ? "page" : undefined}
                  >
                    {id === "smp"
                      ? t("Overview")
                      : pages.find((p) => p[0] === id)![1]}
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
            {error && (
              <div className="error" role="alert">
                {error}
                <button onClick={refresh}>{t("Reload")}</button>
              </div>
            )}
            {!data && !error ? (
              <div className="loading" role="status">
                {t("Loading…")}
              </div>
            ) : data ? (
              (components[page] ?? <p>{t("This page could not be found.")}</p>)
            ) : null}
            {jobs.length > 0 && (
              <section className="job-tray">
                <details
                  open={jobs.some(
                    (j) =>
                      !["succeeded", "failed", "cancelled"].includes(j.state),
                  )}
                >
                  <summary>
                    {t("Activity")}{" "}
                    <span>
                      {
                        jobs.filter(
                          (j) =>
                            !["succeeded", "failed", "cancelled"].includes(
                              j.state,
                            ),
                        ).length
                      }{" "}
                      {t(" in progress")}
                    </span>
                  </summary>
                  {jobs.map((j) => (
                    <div className="job-item" key={j.id}>
                      <div>
                        <strong>{j.progress?.message ?? j.kind}</strong>
                        <small>{date(j.updated_at)}</small>
                        {j.error && (
                          <p className="error">{translateError(j.error)}</p>
                        )}
                        {j.result?.lines && (
                          <pre>{j.result.lines.join("\n")}</pre>
                        )}
                        {j.result?.preview_hash && (
                          <div>
                            <p>
                              {j.result.clear
                                ? t("Ready to place.")
                                : t(
                                    "Something is in the way. Clear the area and try again.",
                                  )}
                            </p>
                            <pre>
                              {JSON.stringify(j.result.summary ?? {}, null, 2)}
                            </pre>
                            <code>{j.result.preview_hash}</code>
                          </div>
                        )}
                      </div>
                      <Status value={j.state} />
                    </div>
                  ))}
                </details>
              </section>
            )}
          </main>
          <footer>
            lkjmc <span>{t("Minecraft community")}</span>
          </footer>
        </div>
      </div>
      {toast && (
        <div role="status" className="toast">
          {toast}
          <button
            aria-label={t("Dismiss notification")}
            onClick={() => setToast("")}
          >
            ×
          </button>
        </div>
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
        <a className="wordmark" href="/">
          lkjmc<span>●</span>
        </a>
        <LanguagePicker />
      </header>
      <main>
        <p className="eyebrow">lkjmc</p>
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
        <div className="landing-grid">
          <section>
            <span>01</span>
            <h2>{t("Survival server")}</h2>
            <p>
              {t(
                "Start at least 10,000 blocks from other players’ starting points and protected land. Protect your own land, manage assets, and trade buildings.",
              )}
            </p>
          </section>
          <section>
            <span>02</span>
            <h2>{t("Friends and groups")}</h2>
            <p>
              {t(
                "Manage friends, private and group chats, teams, and parties. Find invitations and results on Home.",
              )}
            </p>
          </section>
          <section>
            <span>03</span>
            <h2>{t("Your own servers")}</h2>
            <p>
              {t(
                "Create servers within your approved limits. Start and stop them, upload files, invite co-managers, and manage backups.",
              )}
            </p>
          </section>
        </div>
      </main>
      <footer>
        {t(
          "Minecraft is a trademark of Mojang / Microsoft. lkjmc is an unofficial community.",
        )}
      </footer>
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
