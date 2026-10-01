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
      "応答を読み取れませんでした。接続を確認して再読み込みしてください。",
    );
  }
  if (!response.ok)
    throw new ApiError(
      response.status,
      body.error?.message ?? `処理に失敗しました (${response.status})`,
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
  new Intl.NumberFormat("ja-JP").format(value ?? 0);
export const date = (value: string) =>
  value ? new Date(value).toLocaleString("ja-JP") : "";
export const states: Record<string, string> = {
  queued: "順番待ち",
  leased: "処理中",
  waiting: "再開待ち",
  succeeded: "完了",
  failed: "失敗",
  cancelled: "取り消し",
  unprovisioned: "作成待ち",
  provisioning: "作成中",
  stopped: "休止中",
  starting: "起動中",
  running: "稼働中",
  stopping: "停止中",
  unknown: "状態を確認中",
  error: "要確認",
  pending: "準備中",
  active: "有効",
  transferring: "移転中",
  releasing: "解除中",
  released: "解除済み",
  capturing: "預託中",
  escrowed: "保管中",
  listed: "出品中",
  placing: "受け渡し中",
  placed: "設置済み",
  delivered: "受取済み",
  quarantined: "保全中",
  preparing: "準備中",
  activating: "生成中",
  closing: "終了処理中",
  closed: "終了",
  refunding: "返却中",
  refunded: "返却済み",
};
