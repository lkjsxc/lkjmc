import {
  acceptIdentity,
  assertIdentity,
  identityEpoch,
  identitySignal,
  onIdentityReset,
  resetIdentity,
} from "./identity";
import {
  t,
  getLocale,
  message,
  renderSystemMessage,
  type SystemMessage,
} from "./i18n";
export type Data = { [key: string]: any };
export type Me = {
  account: Data;
  csrf: string;
  game_address: string | null;
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
    Object.defineProperty(this, "message", {
      get: () => renderSystemMessage(this.systemMessage),
    });
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
        "text.could_not_reach_the_service_check_your_connection_if_yo_759ce0c0be",
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
      message(
        "text.could_not_read_the_response_check_your_connection_and_reload",
      ),
    );
  }
  if (!response.ok)
    throw new ApiError(
      response.status,
      body.error?.message && typeof body.error.message === "object"
        ? body.error.message
        : message("text.request_failed_0", response.status),
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
  queued: "text.queued",
  leased: "text.in_progress_c1f88e9d",
  waiting: "text.waiting_to_resume",
  succeeded: "text.completed",
  failed: "text.failed",
  cancelled: "text.cancelled",
  unprovisioned: "text.awaiting_creation",
  provisioning: "text.creating",
  stopped: "text.sleeping",
  starting: "text.starting",
  running: "text.running",
  stopping: "text.stopping",
  unknown: "text.checking_status",
  error: "text.needs_attention",
  pending: "text.pending",
  active: "text.active",
  transferring: "text.transferring",
  releasing: "text.releasing",
  released: "text.released",
  capturing: "text.depositing",
  escrowed: "text.stored",
  listed: "text.listed",
  placing: "text.delivering",
  placed: "text.placed",
  delivered: "text.delivered",
  quarantined: "text.quarantined",
  preparing: "text.pending",
  activating: "text.generating",
  closing: "text.closing",
  closed: "text.closed",
  refunding: "text.refunding",
  refunded: "text.refunded",
  ready: "text.saved_b5c120b3",
  freezing: "text.frozen",
  saving: "text.saving",
  verifying: "text.verifying",
  restoring: "text.restoring",
  pruning: "text.pruning",
  pruned: "text.pruned",
};

export function jobTitle(job: Data) {
  const kind = String(job.kind).replaceAll("_", ".");
  if (kind === "server.inspection") {
    const opening = job.open ?? job.result?.open;
    return t(
      opening === true
        ? "text.open_files"
        : opening === false
          ? "text.close_files"
          : "text.files",
    );
  }
  const names: Record<string, string> = {
    "server.create": "text.create_a_server",
    "server.start": "text.start_server",
    "server.stop": "text.stop_server",
    "server.logs": "text.read_logs",
    "server.files": "text.browse_files",
    "server.file.read": "text.read_file",
    "server.file.write": "text.save_file",
    "server.file.delete": "text.delete_file",
    "server.directory.create": "text.create_folder",
    "server.operator": "text.minecraft_operator",
    "player.join": "text.join_server",
    "server.join": "text.join_server",
    "server.configure": "text.server_settings",
    "server.member": "text.member_permissions",
    "server.console": "text.console_command",
    "server.backup": "text.create_backup",
    "server.restore": "text.restore_a_backup",
    "server.install": "text.apply_file",
    "asset.capture": "text.deposit_an_asset",
    "asset.place": "text.place_building",
    "asset.receive": "text.receive_items",
    "npc.sell": "text.sell_materials",
    "expedition.prepare": "text.prepare_expedition",
    "expedition.enter": "text.enter_expedition",
    "expedition.cancel": "text.cancel_preparation",
    "expedition.return": "text.return_to_survival",
    "adventure.return": "text.return_to_survival",
    "adventure.create": "text.prepare_expedition",
    "asset.preview": "text.preview_building_placement",
    "asset.consent": "text.animal_owner_consent",
    "adventure.prepare": "text.prepare_expedition",
    "adventure.cancel": "text.cancel_preparation",
    "adventure.join": "text.enter_expedition",
    "home.set": "text.set_home",
    "home.travel": "text.go_home",
    "claim.create": "text.protect_land",
    "claim.sync": "text.protect_land",
    "claim.release": "text.release_land",
    "listing.create": "text.create_listing",
    "listing.cancel": "text.cancel_listing",
    "listing.buy": "text.buy_listing",
    "coins.transfer": "text.transfer_coins",
    "coin.transfer": "text.transfer_coins",
    "friend.request": "text.friend_request",
    "friend.respond": "text.respond_to_friend_request",
    "friend.remove": "text.remove_friend_or_request",
    "direct.room": "text.start_private_conversation",
    "room.create": "text.create_group_chat",
    "room.leave": "text.leave_group",
    "room.read": "text.mark_conversation_read",
    "message.send": "text.send_message",
    "message.delete": "text.delete_message",
    "team.create": "text.create_team",
    "team.permissions": "text.team_permissions",
    "team.transfer": "text.transfer_leadership",
    "team.leave": "text.leave_team",
    "team.disband": "text.disband_team",
    "party.create": "text.create_party",
    "party.ready": "text.ready_for_adventure",
    "party.transfer": "text.make_leader",
    "party.leave": "text.leave_party",
    invite: "text.send_an_invitation",
    "invite.respond": "text.respond_to_invitation",
    "community.create": "text.create_community",
    privacy: "text.privacy",
    language: "text.language",
    block: "text.block_player",
    report: "text.report_b6ce788d",
    "report.resolve": "text.resolve_report",
    "notifications.read": "text.mark_all_as_read",
    "link.begin": "text.create_a_link_code",
    "link.present": "text.link_to_this_account",
    "link.confirm": "text.confirm_game_data",
    "identity.migrate": "text.link_game_accounts",
    "rank.configure": "text.hosting_access_tiers",
    "account.rank": "text.hosting_access_tiers",
    "account.ban": "text.suspend_account",
    "backup.pin": "text.keep_backup",
    "backup.policy": "text.backup_policy",
    "backup.create": "text.create_backup",
    "backup.restore": "text.restore_a_backup",
  };
  return t(names[kind] ?? "text.action");
}
