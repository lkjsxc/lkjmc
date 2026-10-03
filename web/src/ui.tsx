import { t } from "./i18n";
import {
  useEffect,
  useId,
  useRef,
  useState,
  type ReactNode,
  type FormEvent,
} from "react";
import { api, type Data, states } from "./api";

export function Icon({ name }: { name: string }) {
  const paths: Record<string, ReactNode> = {
    home: (
      <>
        <path d="m3 10 9-7 9 7v10H3Z" />
        <path d="M9 20v-7h6v7" />
      </>
    ),
    play: (
      <>
        <rect x="3" y="5" width="18" height="14" rx="4" />
        <path d="M6 12h6m-3-3v6m7-4h.01m2 3h.01" />
      </>
    ),
    social: (
      <>
        <circle cx="9" cy="8" r="3" />
        <path d="M3 21v-3a6 6 0 0 1 12 0v3m2-15a3 3 0 0 1 0 6m1 3a5 5 0 0 1 3 5" />
      </>
    ),
    chat: (
      <>
        <path d="M3 4h18v12H8l-5 5Z" />
        <path d="M7 8h10M7 12h6" />
      </>
    ),
    teams: (
      <>
        <path d="M5 22V3m0 1h14l-3 5 3 5H5" />
      </>
    ),
    parties: (
      <>
        <circle cx="12" cy="7" r="3" />
        <circle cx="4" cy="10" r="2" />
        <circle cx="20" cy="10" r="2" />
        <path d="M7 21v-4a5 5 0 0 1 10 0v4M1 21v-4a3 3 0 0 1 4-3m18 7v-4a3 3 0 0 0-4-3" />
      </>
    ),
    life: (
      <>
        <path d="m12 2 9 5v10l-9 5-9-5V7Zm0 10 9-5M12 12 3 7m9 5v10" />
      </>
    ),
    market: (
      <>
        <path d="M3 8h18l-2 13H5ZM8 8V6a4 4 0 0 1 8 0v2" />
      </>
    ),
    adventure: (
      <>
        <circle cx="12" cy="12" r="9" />
        <path d="m15 9-2 4-4 2 2-4Z" />
      </>
    ),
    servers: (
      <>
        <rect x="3" y="3" width="18" height="7" rx="2" />
        <rect x="3" y="14" width="18" height="7" rx="2" />
        <path d="M7 6.5h.01M7 17.5h.01m5-11h5m-5 11h5" />
      </>
    ),
    settings: (
      <>
        <path d="M4 6h16M4 12h16M4 18h16" />
        <circle cx="8" cy="6" r="2" />
        <circle cx="16" cy="12" r="2" />
        <circle cx="10" cy="18" r="2" />
      </>
    ),
    admin: (
      <>
        <path d="m12 2 8 4v6c0 5-8 10-8 10S4 17 4 12V6Zm-4 10 3 3 5-6" />
      </>
    ),
    arrow: <path d="M4 12h16m-6-6 6 6-6 6" />,
    menu: <path d="M3 6h18M3 12h18M3 18h18" />,
    close: <path d="m6 6 12 12M6 18 18 6" />,
    voice: (
      <>
        <rect x="9" y="2" width="6" height="13" rx="3" />
        <path d="M5 10v2a7 7 0 0 0 14 0v-2M12 19v3m-4 0h8" />
      </>
    ),
    plus: <path d="M12 4v16M4 12h16" />,
  };
  return (
    <svg
      viewBox="0 0 24 24"
      width="20"
      height="20"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.7"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      {paths[name] ?? paths.play}
    </svg>
  );
}
export function Status({ value }: { value: string }) {
  value = value ?? "unknown";
  return (
    <span className={`status status-${value}`}>
      {t(states[value] ?? value)}
    </span>
  );
}
export function Empty({ children }: { children: ReactNode }) {
  return <div className="empty">{children}</div>;
}
export function Card({
  title,
  children,
  action,
}: {
  title: string;
  children: ReactNode;
  action?: ReactNode;
}) {
  return (
    <section className="card">
      <div className="card-head">
        <h2>{title}</h2>
        {action}
      </div>
      {children}
    </section>
  );
}
export function Modal({
  title,
  children,
  onClose,
}: {
  title: string;
  children: ReactNode;
  onClose: () => void;
}) {
  const ref = useRef<HTMLDialogElement>(null);
  const id = useId();
  useEffect(() => {
    const dialog = ref.current;
    dialog?.showModal();
    return () => dialog?.close();
  }, []);
  return (
    <dialog
      ref={ref}
      aria-labelledby={id}
      onCancel={onClose}
      onClick={(e) => {
        if (e.target === e.currentTarget) onClose();
      }}
    >
      <div className="modal-head">
        <h2 id={id}>{title}</h2>
        <button
          className="icon-button"
          onClick={onClose}
          aria-label={t("Close")}
        >
          <Icon name="close" />
        </button>
      </div>
      {children}
    </dialog>
  );
}
export type Field = {
  name: string;
  label: string;
  type?: "text" | "number" | "checkbox" | "textarea" | "select" | "player";
  options?: { value: string; label: string }[];
  value?: string | number | boolean;
  min?: number;
  step?: number;
  max?: number;
  required?: boolean;
  hint?: string;
};
export function PlayerPicker({
  name,
  label,
  required = true,
}: {
  name: string;
  label: string;
  required?: boolean;
}) {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<Data[]>([]);
  const [chosen, setChosen] = useState<Data | null>(null);
  const [error, setError] = useState("");
  useEffect(() => {
    let alive = true;
    const timer = setTimeout(() => {
      if (query.trim() && !chosen)
        api(`/api/v1/players?q=${encodeURIComponent(query)}`)
          .then((v) => {
            if (alive) setResults(v.players);
          })
          .catch((e) => {
            if (alive) setError(e.message);
          });
      else setResults([]);
    }, 250);
    return () => {
      alive = false;
      clearTimeout(timer);
    };
  }, [query, chosen]);
  return (
    <label className="field">
      {label}
      <input type="hidden" name={name} value={chosen?.id ?? ""} />
      <input
        aria-label={label}
        value={query}
        required={required}
        placeholder={t("Search player names")}
        autoComplete="off"
        onChange={(e) => {
          e.currentTarget.setCustomValidity("");
          setQuery(e.target.value);
          setChosen(null);
        }}
        onInvalid={(e) =>
          e.currentTarget.setCustomValidity(t("Choose a player."))
        }
      />
      {chosen ? (
        <small>
          {t("Selected: ")}
          {chosen.name}
        </small>
      ) : (
        <div className="search-results">
          {results.map((p) => (
            <button
              type="button"
              key={p.id}
              onClick={() => {
                setChosen(p);
                setQuery(p.name);
              }}
            >
              {p.name}
              <small>{p.rank}</small>
            </button>
          ))}
        </div>
      )}
      {error && <small role="alert">{error}</small>}
    </label>
  );
}
export function ActionForm({
  fields,
  onSubmit,
  submit = t("Save"),
  children,
}: {
  fields: Field[];
  onSubmit: (data: Data) => Promise<unknown>;
  submit?: string;
  children?: ReactNode;
}) {
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  async function send(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const values = new FormData(event.currentTarget);
    const data: Data = {};
    for (const field of fields) {
      const raw = values.get(field.name);
      if (field.type === "checkbox") data[field.name] = raw === "on";
      else if (field.type === "number") data[field.name] = Number(raw);
      else data[field.name] = String(raw ?? "");
      if (
        field.required !== false &&
        field.type === "player" &&
        !data[field.name]
      ) {
        setError(t("Choose a player from the search results."));
        return;
      }
    }
    setBusy(true);
    setError("");
    try {
      await onSubmit(data);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }
  return (
    <form onSubmit={send} className="action-form">
      <fieldset disabled={busy}>
        {fields.map((field) =>
          field.type === "player" ? (
            <PlayerPicker
              key={field.name}
              name={field.name}
              label={field.label}
              required={field.required}
            />
          ) : (
            <label
              className={`field ${field.type === "checkbox" ? "check-field" : ""}`}
              key={field.name}
            >
              {field.type !== "checkbox" && field.label}
              {field.type === "select" ? (
                <select
                  aria-label={field.label}
                  name={field.name}
                  defaultValue={String(
                    field.value ?? field.options?.[0]?.value ?? "",
                  )}
                  required={field.required !== false}
                >
                  {field.options?.map((o) => (
                    <option key={o.value} value={o.value}>
                      {o.label}
                    </option>
                  ))}
                </select>
              ) : field.type === "textarea" ? (
                <textarea
                  aria-label={field.label}
                  name={field.name}
                  defaultValue={String(field.value ?? "")}
                  required={field.required !== false}
                  maxLength={field.max ?? 4000}
                />
              ) : (
                <input
                  aria-label={field.label}
                  name={field.name}
                  type={field.type ?? "text"}
                  defaultValue={
                    field.type === "checkbox"
                      ? undefined
                      : String(field.value ?? "")
                  }
                  defaultChecked={
                    field.type === "checkbox" ? Boolean(field.value) : undefined
                  }
                  min={field.min}
                  step={field.step}
                  max={field.max}
                  maxLength={
                    field.type === "number" ? undefined : (field.max ?? 128)
                  }
                  required={
                    field.type !== "checkbox" && field.required !== false
                  }
                />
              )}{" "}
              {field.type === "checkbox" && field.label}
              {field.hint && <small>{field.hint}</small>}
            </label>
          ),
        )}
        {children}
        {error && (
          <p role="alert" className="error">
            {error}
          </p>
        )}
        <button type="submit" className="primary">
          {busy ? t("Working…") : submit}
        </button>
      </fieldset>
    </form>
  );
}
