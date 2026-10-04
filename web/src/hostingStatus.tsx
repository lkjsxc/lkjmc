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
  permission_required: "You need hosting permission for this action.",
  maintenance: "Wait for the current operation before changing power.",
  observation_stale: "Wait for a current observation before changing power.",
  provisioning: "Server creation must finish before Minecraft can start.",
  server_sleeping: "Start Minecraft before connecting.",
  already_running: "Minecraft is already running or starting.",
  already_stopped: "Minecraft is already stopped.",
  lobby_always_running:
    "The lobby stays running for players arriving and returning.",
  files_closed: "Open files to access the stopped server.",
  restore_in_progress: "Wait for the restore to finish.",
  game_not_ready: "Wait for Minecraft to reach a confirmed power state.",
};
