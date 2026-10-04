import {
  acceptIdentity,
  assertIdentity,
  identityEpoch,
  identitySignal,
  onIdentityReset,
  resetIdentity,
} from "./identity";
import { t, getLocale, message, renderSystemMessage, type SystemMessage } from "./i18n";
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
onIdentityReset(() => {
  csrf = "";
  jobRequests.clear();
});
export class ApiError extends Error {
  constructor(
    public status: number,
    public systemMessage: SystemMessage,
  ) {
    super();
    Object.defineProperty(this, "message", { get: () => renderSystemMessage(this.systemMessage) });
  }
}
export async function api<T = Data>(
  path: string,
  options: RequestInit = {},
): Promise<T> {
  const epoch = identityEpoch();
  const sessionSignal = identitySignal();
  const headers = new Headers(options.headers);
  if (options.method && options.method !== "GET") {
    headers.set("x-csrf-token", csrf);
    if (!(options.body instanceof FormData))
      headers.set("content-type", "application/json");
  }
  let response: Response;
  try {
    response = await fetch(path, {
      ...options,
      signal: AbortSignal.any([
        sessionSignal,
        AbortSignal.timeout(20000),
        ...(options.signal ? [options.signal] : []),
      ]),
      headers,
      credentials: "same-origin",
    });
  } catch (e) {
    assertIdentity(epoch);
    if (options.signal?.aborted) throw e;
    throw new ApiError(
      0,
      message(
        "Could not reach the service. Check your connection. If you submitted an action, check its details before trying again.",
      ),
    );
  }
  assertIdentity(epoch);
  if (response.status === 401) {
    resetIdentity();
    throw new ApiError(401, message("error.login_required"));
  }
  if (path === "/auth/logout" && response.ok) {
    resetIdentity();
    return {} as T;
  }
  const text = await response.text();
  assertIdentity(epoch);
  let body: Data;
  try {
    body = JSON.parse(text);
  } catch {
    throw new ApiError(
      response.status,
      message("Could not read the response. Check your connection and reload."),
    );
  }
  if (!response.ok)
    throw new ApiError(
      response.status,
      (body.error?.message && typeof body.error.message === "object"
        ? body.error.message : message("Request failed ({0})", response.status)),
    );
  if (path === "/api/v1/me") {
    acceptIdentity(body.account.id, body.csrf);
    setCsrf(body.csrf);
  }
  return body as T;
}
const jobRequests = new Map<string, Promise<Data>>();
export function readJob(id: string, signal?: AbortSignal) {
  // A new view must reauthorize; do not join a request started by a closed view.
  if (signal) return api(`/api/v1/jobs/${encodeURIComponent(id)}`, { signal });
  let request = jobRequests.get(id);
  if (!request) {
    request = api(`/api/v1/jobs/${encodeURIComponent(id)}`).finally(() => {
      if (jobRequests.get(id) === request) jobRequests.delete(id);
    });
    jobRequests.set(id, request);
  }
  return request;
}
export async function command(
  type: string,
  fields: Data = {},
  requestId: string = crypto.randomUUID(),
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

export function jobTitle(job: Data) {
  const kind = String(job.kind).replaceAll("_", ".");
  if (kind === "server.inspection") {
    const opening = job.open ?? job.result?.open;
    return t(opening === true ? "Open files" : opening === false ? "Close files" : "Files");
  }
  const names: Record<string, string> = {
    "server.create": "Create a server",
    "server.start": "Start server",
    "server.stop": "Stop server",
    "server.logs": "Read logs",
    "server.files": "Browse files",
    "server.file.read": "Read file",
    "server.file.write": "Save file",
    "server.file.delete": "Delete file",
    "server.directory.create": "Create folder",
    "server.operator": "Minecraft operator",
    "player.join": "Join server",
    "server.join": "Join server",
    "server.configure": "Server settings",
    "server.member": "Member permissions",
    "server.console": "Console command",
    "server.backup": "Create backup",
    "server.restore": "Restore a backup",
    "server.install": "Apply file",
    "asset.capture": "Deposit an asset",
    "asset.place": "Place building",
    "asset.receive": "Receive items",
    "npc.sell": "Sell materials",
    "adventure.create": "Private End",
    "asset.preview": "Preview building placement",
    "asset.consent": "Animal owner consent",
    "adventure.prepare": "Private End",
    "adventure.cancel": "Cancel adventure",
    "adventure.join": "Join adventure",
    "home.set": "Set home",
    "home.travel": "Go home",
    "claim.create": "Protect land",
    "claim.sync": "Protect land",
    "claim.release": "Release land",
    "listing.create": "Create listing",
    "listing.cancel": "Cancel listing",
    "listing.buy": "Buy listing",
    "coins.transfer": "Transfer coins",
    "coin.transfer": "Transfer coins",
    "friend.request": "Friend request",
    "friend.respond": "Respond to friend request",
    "friend.remove": "Remove friend or request",
    "direct.room": "Start private conversation",
    "room.create": "Create group chat",
    "room.leave": "Leave group",
    "room.read": "Mark conversation read",
    "message.send": "Send message",
    "message.delete": "Delete message",
    "team.create": "Create team",
    "team.permissions": "Team permissions",
    "team.transfer": "Transfer leadership",
    "team.leave": "Leave team",
    "team.disband": "Disband team",
    "party.create": "Create party",
    "party.ready": "Ready for adventure",
    "party.transfer": "Make leader",
    "party.leave": "Leave party",
    invite: "Send an invitation",
    "invite.respond": "Respond to invitation",
    "community.create": "Create community",
    privacy: "Privacy",
    language: "Language",
    block: "Block player",
    report: "Report",
    "report.resolve": "Resolve report",
    "notifications.read": "Mark all as read",
    "link.begin": "Create a link code",
    "link.present": "Link to this account",
    "link.confirm": "Confirm game data",
    "identity.migrate": "Link game accounts",
    "rank.configure": "Hosting access tiers",
    "account.rank": "Hosting access tiers",
    "account.ban": "Suspend account",
    "backup.pin": "Keep backup",
    "backup.policy": "Backup policy",
    "backup.create": "Create backup",
    "backup.restore": "Restore a backup",
  };
  return t(names[kind] ?? "Action");
}
