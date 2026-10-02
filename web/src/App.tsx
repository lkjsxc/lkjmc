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
const pages = [
  ["home", "ホーム", "通知と処理状況"],
  ["play", "サーバー一覧", "稼働状況と接続先"],
  ["smp", "SMP", "稼働状況、参加方法、ゲーム内の管理"],
  ["social", "フレンド・チャット", "メンバーとグループの管理"],
  ["life", "土地・資産", "土地、ホーム、残高の管理"],
  ["market", "マーケット", "商品の出品と購入"],
  ["adventure", "プライベートエンド", "専用ワールドの作成と参加"],
  ["servers", "サーバー管理", "設定、ファイル、バックアップ"],
  ["settings", "アカウント設定", "連携アカウントと公開範囲"],
  ["admin", "運営管理", "利用権限、通報、稼働状況"],
];
const smpPages = ["smp", "life", "market", "adventure"];

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
      <div className="app-shell">
        <aside className={menu ? "sidebar open" : "sidebar"}>
          <a className="wordmark" href="#home">
            lkjmc<span>●</span>
          </a>
          <p className="side-caption">Minecraft サーバー管理</p>
          <nav aria-label="メインメニュー">
            {available.map(([id, name]) => (
              <a
                key={id}
                href={`#${id}`}
                className={section === id ? "active" : ""}
                aria-current={page === id ? "page" : undefined}
              >
                <Icon name={id} />
                <span>{name}</span>
                {section === id && <span className="nav-dot" />}
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
              <span>lkjmc / {inSmp ? "SMP" : current[1]}</span>
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
            {inSmp && (
              <nav className="breadcrumbs" aria-label="現在の位置">
                <a href="#play">サーバー一覧</a>
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
                更新
              </button>
            </div>
            {inSmp && (
              <nav className="section-nav" aria-label="SMPメニュー">
                {smpPages.map((id) => (
                  <a
                    key={id}
                    href={`#${id}`}
                    aria-current={page === id ? "page" : undefined}
                  >
                    {id === "smp" ? "概要" : pages.find((p) => p[0] === id)![1]}
                  </a>
                ))}
              </nav>
            )}
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
            lkjmc <span>Minecraft サーバー管理</span>
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
        <span>Minecraft サーバー管理</span>
      </header>
      <main>
        <p className="eyebrow">lkjmc</p>
        <h1>Minecraft サーバー管理</h1>
        <p className="lead">
          サーバーへの参加、フレンドとの連絡、土地や建築物の取引を管理します。
          <br />
          アカウントを登録するか、既存のアカウントでログインしてください。
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
            <span>01</span>
            <h2>サバイバルサーバー</h2>
            <p>
              初回参加時は、ほかのプレイヤーの開始地点や保護地から10,000ブロック以上離れた場所に移動します。土地の保護、資産の管理、建築物の売買に対応しています。
            </p>
          </section>
          <section>
            <span>02</span>
            <h2>フレンド・グループ管理</h2>
            <p>
              フレンド申請、個別・グループチャット、チーム、パーティーを管理します。招待や処理結果はホームで確認できます。
            </p>
          </section>
          <section>
            <span>03</span>
            <h2>ユーザーサーバー</h2>
            <p>
              運営が承認したランクの上限内でサーバーを作成します。起動・停止、ファイルのアップロード、共同管理者の設定、バックアップを管理できます。
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
