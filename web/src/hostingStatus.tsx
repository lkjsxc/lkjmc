import { type Data } from "./api";

type PowerAction = { allowed: boolean; reason: string | null };
export type HostingStatus = {
  machine_state: "running" | "stopped" | "unknown";
  game_state:
    | "unprovisioned"
    | "starting"
    | "running"
    | "stopping"
    | "stopped"
    | "error"
    | "unknown";
  observation_fresh: boolean;
  joinable: boolean;
  actions: { start: PowerAction; stop: PowerAction };
};

/** Every actionable state comes from the authorized runtime projection. */
export function hostingStatus(server: Data): HostingStatus {
  const status = server.status;
  const fresh = status?.observation_fresh === true;
  const gameStates = [
    "unprovisioned",
    "starting",
    "running",
    "stopping",
    "stopped",
    "error",
    "unknown",
  ];
  const action = (name: "start" | "stop"): PowerAction => ({
    allowed: fresh && status?.actions?.[name]?.allowed === true,
    reason: !fresh
      ? "observation_stale"
      : (status?.actions?.[name]?.reason ??
        (status?.actions?.[name]?.allowed === true ? null : "game_not_ready")),
  });
  return {
    machine_state: ["running", "stopped"].includes(status?.machine_state)
      ? status.machine_state
      : "unknown",
    game_state:
      fresh && gameStates.includes(status?.game_state)
        ? status.game_state
        : "unknown",
    observation_fresh: fresh,
    joinable: fresh && status?.joinable === true,
    actions: { start: action("start"), stop: action("stop") },
  };
}

export const hostingActionReasons: Record<string, string> = {
  permission_required: "text.you_need_hosting_permission_for_this_action",
  maintenance: "text.wait_for_the_current_operation_before_changing_power",
  observation_stale: "text.wait_for_a_current_observation_before_changing_power",
  provisioning: "text.server_creation_must_finish_before_minecraft_can_start",
  server_sleeping: "text.start_minecraft_before_connecting",
  already_running: "text.minecraft_is_already_running_or_starting",
  already_stopped: "text.minecraft_is_already_stopped",
  lobby_always_running:
    "text.the_lobby_stays_running_for_players_arriving_and_returning",
  files_closed: "text.open_files_to_access_the_stopped_server",
  restore_in_progress: "text.wait_for_the_restore_to_finish",
  game_not_ready: "text.wait_for_minecraft_to_reach_a_confirmed_power_state",
};
