import { t, getLocale, translateError } from "./i18n";
export type Data = { [key: string]: any };
export type Me = {
  account: Data;
  csrf: string;
  game_address: string;
  voice_available: boolean;
  development: boolean;
};
let csrf = "";
export function setCsrf(value: string) {
  csrf = value;
}
export class ApiError extends Error {
  constructor(
    public status: number,
    message: string,
  ) {
    super(message);
  }
}
export async function api<T = Data>(
  path: string,
  options: RequestInit = {},
): Promise<T> {
  const headers = new Headers(options.headers);
  if (options.method && options.method !== "GET") {
    headers.set("x-csrf-token", csrf);
    if (!(options.body instanceof FormData))
      headers.set("content-type", "application/json");
  }
  const response = await fetch(path, {
    ...options,
    headers,
    credentials: "same-origin",
  });
  const text = await response.text();
  let body: Data;
  try {
    body = JSON.parse(text);
  } catch {
    throw new ApiError(
      response.status,
      t("Could not read the response. Check your connection and reload."),
    );
  }
  if (!response.ok)
    throw new ApiError(
      response.status,
      (body.error?.message ? translateError(body.error.message) : null) ??
        t("Request failed ({0})", response.status),
    );
  return body as T;
}
export async function command(
  type: string,
  fields: Data = {},
  requestId = crypto.randomUUID(),
) {
  const response = await api("/api/v1/commands", {
    method: "POST",
    body: JSON.stringify({
      request_id: requestId,
      command: { type, ...fields },
    }),
  });
  return response.result as Data;
}
export const money = (value: number) =>
  new Intl.NumberFormat(getLocale()).format(value ?? 0);
export const date = (value: string) =>
  value ? new Date(value).toLocaleString(getLocale()) : "";
export const states: Record<string, string> = {
  queued: "Queued",
  leased: "In progress",
  waiting: "Waiting to resume",
  succeeded: "Completed",
  failed: "Failed",
  cancelled: "Cancelled",
  unprovisioned: "Awaiting creation",
  provisioning: "Creating",
  stopped: "Sleeping",
  starting: "Starting",
  running: "Running",
  stopping: "Stopping",
  unknown: "Checking status",
  error: "Needs attention",
  pending: "Pending",
  active: "Active",
  transferring: "Transferring",
  releasing: "Releasing",
  released: "Released",
  capturing: "Depositing",
  escrowed: "Stored",
  listed: "Listed",
  placing: "Delivering",
  placed: "Placed",
  delivered: "Delivered",
  quarantined: "Quarantined",
  preparing: "Pending",
  activating: "Generating",
  closing: "Closing",
  closed: "Closed",
  refunding: "Refunding",
  refunded: "Refunded",
  ready: "Saved",
  freezing: "Frozen",
  saving: "Saving",
  verifying: "Verifying",
  restoring: "Restoring",
  pruning: "Pruning",
  pruned: "Pruned",
};
