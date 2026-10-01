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
const pages = [
  ["home", "ホーム", "あなたの拠点"],
  ["play", "遊ぶ", "次の行き先を選ぶ"],
  ["social", "つながり", "離れていても、一緒に"],
  ["life", "暮らし", "少しずつ、自分の場所に"],
  ["market", "マーケット", "誰かの工夫が、次の暮らしへ"],
  ["adventure", "冒険", "仲間と、まだ見ぬ場所へ"],
  ["servers", "マイサーバー", "自分たちの遊び場をつくる"],
  ["settings", "設定", "アカウントと公開範囲"],
  ["admin", "運営", "状態と記録を確かめる"],
];

export function App() {
  const [me, setMe] = useState<Me | null>(null);
  const [loading, setLoading] = useState(true);
  const [fatal, setFatal] = useState("");
  const [page, setPage] = useState(location.hash.slice(1) || "home");
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
      api(`/api/v1/view/${page}`)
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
  async function send(type: string, values: Data = {}) {
    const result = await command(type, values);
    if (result.job_id) {
      setJobs((j) =>
        [
          { id: result.job_id, kind: type, state: "queued" },
          ...j.filter((v) => v.id !== result.job_id),
        ].slice(0, 10),
      );
      setToast("受け付けました。処理の結果は進行状況に表示します。");
    } else setToast("保存しました。");
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
        <p>接続を確認しています…</p>
      </div>
    );
  if (!me) return <Landing error={fatal} />;
  const current = pages.find((p) => p[0] === page) ?? pages[0];
  const available = pages.filter(
    (p) => p[0] !== "admin" || me.account.administrator,
  );
  const context: Context = { me, send, act, open: setDialog, refresh, go };
  const components: Record<string, ReactNode> = {
    home: <Home data={data ?? {}} />,
    play: <Play data={data ?? {}} />,
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
      <div className="app-shell">
        <aside className={menu ? "sidebar open" : "sidebar"}>
          <a className="wordmark" href="#home">
            lkjmc<span>●</span>
          </a>
          <p className="side-caption">暮らす。集まる。つくる。</p>
          <nav aria-label="メインメニュー">
            {available.map(([id, name]) => (
              <a
                key={id}
                href={`#${id}`}
                className={page === id ? "active" : ""}
                aria-current={page === id ? "page" : undefined}
              >
                <Icon name={id} />
                <span>{name}</span>
                {page === id && <span className="nav-dot" />}
              </a>
            ))}
          </nav>
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
                onClick={() => setMenu(!menu)}
                aria-label="メニューを開く"
                aria-expanded={menu}
              >
                <Icon name="menu" />
              </button>
              <span>あなたの遊び場 / {current[1]}</span>
            </div>
            <button
              className="connection"
              onClick={() =>
                navigator.clipboard
                  .writeText(me.game_address)
                  .then(() => setToast("接続先をコピーしました。"))
                  .catch(() => setToast(me.game_address))
              }
            >
              <span className="connection-dot" />
              {me.game_address}
              <span className="copy-label">コピー</span>
            </button>
          </header>
          <main id="main">
            <div className="page-heading">
              <div>
                <p className="eyebrow">{current[2]}</p>
                <h1>{current[1]}</h1>
              </div>
              <button className="quiet" onClick={refresh}>
                更新
              </button>
            </div>
            {me.development && (
              <div className="dev-banner">
                開発環境 — 公開サーバーへの参加確認は別途必要です。
              </div>
            )}
            {error && (
              <div className="error" role="alert">
                {error}
                <button onClick={refresh}>再読み込み</button>
              </div>
            )}
            {!data && !error ? (
              <div className="loading" role="status">
                読み込んでいます…
              </div>
            ) : data ? (
              (components[page] ?? <p>このページは見つかりません。</p>)
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
                    進行状況{" "}
                    <span>
                      {
                        jobs.filter(
                          (j) =>
                            !["succeeded", "failed", "cancelled"].includes(
                              j.state,
                            ),
                        ).length
                      }{" "}
                      件処理中
                    </span>
                  </summary>
                  {jobs.map((j) => (
                    <div className="job-item" key={j.id}>
                      <div>
                        <strong>{j.progress?.message ?? j.kind}</strong>
                        <small>{date(j.updated_at)}</small>
                        {j.error && <p className="error">{j.error}</p>}
                        {j.result?.lines && (
                          <pre>{j.result.lines.join("\n")}</pre>
                        )}
                        {j.result?.preview_hash && (
                          <div>
                            <p>
                              {j.result.clear
                                ? "設置可能です。"
                                : "障害物があります。整地後に再確認してください。"}
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
            lkjmc <span>それぞれの場所から、つながる。</span>
          </footer>
        </div>
      </div>
      {toast && (
        <div role="status" className="toast">
          {toast}
          <button aria-label="通知を閉じる" onClick={() => setToast("")}>
            ×
          </button>
        </div>
      )}
      {dialog && (
        <Modal title={dialog.title} onClose={() => setDialog(null)}>
          {dialog.note && <div className="modal-note">{dialog.note}</div>}
          <ActionForm
            fields={dialog.fields ?? []}
            submit={dialog.submit ?? "確定する"}
            onSubmit={async (values) => {
              await (dialog.action
                ? dialog.action(values)
                : send(dialog.type!, { ...dialog.values, ...values }));
              setDialog((current) => current === dialog ? null : current);
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
        <span>暮らす。集まる。つくる。</span>
      </header>
      <main>
        <p className="eyebrow">YOUR NEXT PLACE TO PLAY</p>
        <h1>
          遠くから、
          <br />
          はじまる。
        </h1>
        <p className="lead">
          まだ誰もいない場所で暮らしをつくる。
          <br />
          会いたくなったら、仲間を呼ぶ。
          <br />
          次の遊び場も、自分たちの手で。
        </p>
        <div className="landing-actions">
          {ready ? (
            <a className="button primary" href="/auth/login">
              登録・ログイン <Icon name="arrow" />
            </a>
          ) : (
            <p className="notice">
              {ready === null
                ? "接続を確認しています…"
                : "Webログインは接続準備中です。"}
            </p>
          )}
          <div>
            <small>ゲームの接続先</small>
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
            <span>01 / LIVE</span>
            <h2>自分だけのはじまり</h2>
            <p>
              遠く離れた場所から始まるSMP。建てた家も、育てた暮らしも、ここに残ります。
            </p>
          </section>
          <section>
            <span>02 / CONNECT</span>
            <h2>好きなときに、集まる</h2>
            <p>
              フレンド、チーム、チャット、音声。遊ぶ前も、遊んだあとも、同じ場所で。
            </p>
          </section>
          <section>
            <span>03 / CREATE</span>
            <h2>遊び場をひらく</h2>
            <p>
              承認されたランクで、自分たちのサーバーを。設定やファイル、バックアップも一か所に。
            </p>
          </section>
        </div>
      </main>
      <footer>
        Minecraft は Mojang / Microsoft の商標です。lkjmc
        は非公式のコミュニティです。
      </footer>
    </div>
  );
}
