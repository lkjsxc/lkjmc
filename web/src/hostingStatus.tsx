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

/** Legacy responses can describe Minecraft, but cannot establish the VM state. */
export function hostingStatus(server: Data): HostingStatus {
  const status = server.status;
  const observedAt = Date.parse(server.last_observed_at ?? "");
  const age = Date.now() - observedAt;
  const fresh = status
    ? status.observation_fresh === true
    : Number.isFinite(age) && age >= -5000 && age <= 45000;
  const gameStates = [
    "unprovisioned",
    "starting",
    "running",
    "stopping",
    "stopped",
    "error",
    "unknown",
  ];
  const game =
    status?.game_state ??
    (server.observed === "unprovisioned" || server.observed === "provisioning"
      ? "unprovisioned"
      : fresh
        ? server.observed
        : "unknown");
  const gameState = gameStates.includes(game) ? game : "unknown";
  const inspecting =
    server.inspection?.state === "ready" && server.can_administer;
  const startReason = !server.can_manage
    ? "permission_required"
    : !fresh
      ? "observation_stale"
      : ["unprovisioned", "provisioning"].includes(server.observed)
        ? "provisioning"
        : server.maintenance && !inspecting
          ? "maintenance"
          : server.desired === "running"
            ? "already_running"
            : server.observed !== "stopped"
              ? "game_not_ready"
              : null;
  const stopReason = !server.can_manage
    ? "permission_required"
    : server.kind === "lobby"
      ? "lobby_always_running"
      : !fresh
        ? "observation_stale"
        : server.maintenance
          ? "maintenance"
          : server.observed === "stopped" && server.desired === "stopped"
            ? "already_stopped"
            : server.observed !== "running" || server.desired !== "running"
              ? "game_not_ready"
              : null;
  const action = (
    name: "start" | "stop",
    reason: string | null,
  ): PowerAction =>
    status
      ? {
          allowed: status.actions?.[name]?.allowed === true,
          reason:
            status.actions?.[name]?.reason ??
            (status.actions?.[name]?.allowed === true
              ? null
              : "game_not_ready"),
        }
      : { allowed: reason === null, reason };
  return {
    machine_state: ["running", "stopped"].includes(status?.machine_state)
      ? status.machine_state
      : "unknown",
    game_state: gameState,
    observation_fresh: fresh,
    joinable: status
      ? status.joinable === true
      : fresh &&
        gameState === "running" &&
        server.desired === "running" &&
        !server.maintenance,
    actions: {
      start: action("start", startReason),
      stop: action("stop", stopReason),
    },
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
