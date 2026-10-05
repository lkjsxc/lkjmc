import { t } from "./i18n";

export type Route = {
  path: string;
  area: string;
  section: string;
  component: string;
  title: string;
  description: string;
  api?: string;
  id?: string;
  cursor?: string;
  unread?: boolean;
};
const roots = [
  [
    "play",
    "text.play",
    "text.your_next_adventure_starts_here",
    "play",
    "/play",
  ],
  [
    "worlds",
    "text.worlds",
    "text.find_a_world_to_call_home",
    "life",
    "/worlds",
  ],
  [
    "people",
    "text.people_7db20897",
    "text.friends_teams_and_parties",
    "social",
    "/people",
  ],
  [
    "timeline",
    "text.timeline",
    "text.conversations_and_updates",
    "chat",
    "/timeline",
  ],
  [
    "hosting",
    "text.hosting",
    "text.your_servers_and_tools",
    "servers",
    "/hosting/servers",
  ],
  [
    "account",
    "text.account",
    "text.your_profile_and_preferences",
    "settings",
    "/account",
  ],
  [
    "admin",
    "text.administration",
    "text.community_and_service_tools",
    "admin",
    "/admin",
  ],
];
export const topPages = () =>
  roots
    .filter(([id]) => id !== "account")
    .map(([id, name, description, icon, path]) => ({
      id,
      name: t(name),
      description: t(description),
      icon,
      path,
    }));
// A cutover has one route vocabulary. Unknown and retired paths remain missing.
export const normalize = (path: string) =>
  !path || path === "/" ? "/play" : path.startsWith("/") ? path : "/" + path;
const uuid = (value?: string) => !!value && /^[0-9a-f-]{36}$/.test(value);
export function resolveRoute(raw: string): Route {
  const url = new URL(normalize(raw), location.origin);
  const [area, second, third, fourth] = url.pathname.split("/").filter(Boolean);
  const length = url.pathname.split("/").filter(Boolean).length;
  const root = roots.find((r) => r[0] === area);
  const result: Route = {
    path: url.pathname + url.search,
    area: area ?? "play",
    section: "overview",
    component: "missing",
    title: "text.page_not_found",
    description: "text.choose_a_page_from_the_menu",
  };
  const set = (
    section: string,
    component: string,
    title: string,
    api?: string,
  ) =>
    Object.assign(result, {
      section,
      component,
      title,
      api,
      description: root?.[2] ?? "text.temporary_worlds_lasting_adventures",
    });
  if (area === "timeline" && length === 1) {
    result.id = url.searchParams.get("room") ?? undefined;
    set("timeline", "timeline", "text.timeline");
  } else if (
    area === "play" &&
    length <= 2 &&
    (!second || ["invitations", "notifications"].includes(second))
  )
    set(
      second ?? "overview",
      second ? "feed" : "play-hub",
      second === "invitations"
        ? "text.invitations"
        : second === "notifications"
          ? "text.notifications"
          : "text.play",
      second ? "/api/v1/history/" + second + url.search : "/api/v1/view/play",
    );
  else if (area === "worlds" && length === 1)
    set("list", "worlds", "text.worlds", "/api/v1/view/play");
  else if (area === "worlds" && uuid(second) && length <= 3) {
    result.id = second;
    if (!third)
      set(
        "overview",
        "world",
        "text.world_overview",
        "/api/v1/servers/" + second,
      );
    if (third === "world") {
      const tabs: Record<string, [string, string]> = {
        land: ["land", "text.protected_land"],
        homes: ["homes", "text.homes"],
        meetup: ["meetup", "text.meet_up"],
        achievements: ["achievements", "text.achievements"],
      };
      const tab = url.searchParams.get("tab") ?? "land";
      if (tabs[tab])
        set(
          tabs[tab][0],
          "life",
          tabs[tab][1],
          "/api/v1/servers/" + second + "?section=" + tabs[tab][0],
        );
    }
    if (third === "economy") {
      const tabs: Record<string, [string, string, string]> = {
        wallet: ["coins", "life", "text.wallet"],
        market: ["market", "market", "text.market"],
        storage: ["stored-assets", "market", "text.storage"],
        materials: ["materials", "market", "text.sell_materials"],
        history: ["coin-history", "life", "text.coin_history"],
      };
      const tab = url.searchParams.get("tab") ?? "wallet";
      if (tabs[tab])
        set(
          tabs[tab][0],
          tabs[tab][1],
          tabs[tab][2],
          "/api/v1/servers/" + second + "?section=" + tabs[tab][0],
        );
    }
  } else if (area === "expeditions" && length <= 2) {
    if (!second)
      set(
        "expeditions",
        "expeditions",
        "text.expeditions",
        "/api/v1/view/expedition",
      );
    else if (second === "journal")
      set(
        "journal",
        "expeditions",
        "text.expedition_journal",
        "/api/v1/expeditions" + url.search,
      );
    else if (uuid(second)) {
      result.id = second;
      set(
        "detail",
        "expeditions",
        "text.end_expedition",
        "/api/v1/expeditions/" + second,
      );
    }
  } else if (area === "people" && second === "teams" && length <= 4) {
    if (!third)
      set("teams", "teams", "text.teams", "/api/v1/view/social?section=teams");
    else if (uuid(third)) {
      const tabs: Record<string, [string, string]> = {
        overview: ["team", "text.overview"],
        members: ["team-members", "text.members"],
        settings: ["team-settings", "text.settings"],
      };
      const tab = tabs[fourth ?? "overview"];
      if (tab) {
        result.id = third;
        set(tab[0], "teams", tab[1], "/api/v1/teams/" + third);
      }
    }
  } else if (area === "people" && length <= 3) {
    const group = second ?? "friends";
    const maps: Record<string, Record<string, [string, string]>> = {
      friends: {
        overview: ["friends", "text.friends"],
        incoming: ["incoming", "text.incoming_friend_requests"],
        outgoing: ["outgoing", "text.sent_friend_requests"],
      },
      parties: {
        overview: ["party", "text.parties"],
        members: ["party-members", "text.members"],
        ready: ["party-ready", "text.ready_for_adventure"],
      },
    };
    const found = maps[group]?.[third ?? "overview"];
    if (found)
      set(found[0], "social", found[1], "/api/v1/view/social?section=" + group);
  } else if (area === "hosting" && second === "servers" && length <= 4) {
    if (!third)
      set("list", "managed-list", "text.your_servers", "/api/v1/view/servers");
    else if (third === "new" && !fourth)
      set(
        "new",
        "create-server",
        "text.create_a_server",
        "/api/v1/server-presets",
      );
    else if (uuid(third)) {
      result.id = third;
      const names: Record<string, string> = {
        overview: "text.server_overview",
        console: "text.console",
        logs: "text.logs",
        files: "text.files",
        backups: "text.backups",
        members: "text.members",
        settings: "text.settings",
      };
      const section = fourth ?? "overview";
      if (names[section])
        set(
          "manage-" + section,
          "managed-server",
          names[section],
          "/api/v1/servers/" +
            third +
            "?section=manage-" +
            (section === "logs" ? "console" : section),
        );
    }
  } else if (area === "hosting" && second === "communities" && length === 2)
    set(
      "communities",
      "social",
      "text.server_communities",
      "/api/v1/view/social?section=communities",
    );
  else if (area === "account" && length <= 2) {
    const names: Record<string, string> = {
      overview: "text.account",
      profile: "text.profile",
      privacy: "text.privacy",
      linking: "text.link_game_accounts",
      blocks: "text.blocked_players",
      reports: "text.your_reports",
    };
    if (names[second ?? "overview"])
      set(
        second ?? "profile",
        "settings",
        names[second ?? "overview"],
        "/api/v1/view/settings?section=" + (second ?? "profile"),
      );
  } else if (area === "admin" && length <= 2) {
    const names: Record<string, string> = {
      overview: "text.administration",
      reports: "text.reports",
      ranks: "text.hosting_access_tiers",
      backups: "text.official_backups",
      operations: "text.operations",
      audit: "text.audit_log",
    };
    if (names[second ?? "overview"])
      set(
        second ?? "overview",
        second ? "admin" : "admin-home",
        names[second ?? "overview"],
        second === "operations"
          ? "/api/v1/admin/operations" + url.search
          : "/api/v1/view/admin?section=" + (second ?? "overview"),
      );
  }
  result.cursor = url.searchParams.get("cursor") ?? undefined;
  result.unread = url.searchParams.get("unread") === "true";
  return result;
}
export function childPages(
  route: Route,
  server?: { kind?: string; can_administer?: boolean },
) {
  let pairs: string[][] = [];
  let base = "/" + route.area;
  if (route.area === "play")
    pairs = [
      ["", "text.play"],
      ["invitations", "text.invitations"],
      ["notifications", "text.notifications"],
    ];
  if (route.area === "people") {
    const group = route.path.split("/")[2] ?? "friends";
    base = "/people/" + group;
    if (group === "friends")
      pairs = [
        ["", "text.friends"],
        ["incoming", "text.incoming_friend_requests"],
        ["outgoing", "text.sent_friend_requests"],
      ];
    if (group === "teams" && route.id) {
      base += "/" + route.id;
      pairs = [
        ["", "text.overview"],
        ["members", "text.members"],
        ["settings", "text.settings"],
      ];
    }
    if (group === "parties")
      pairs = [
        ["", "text.party"],
        ["members", "text.members"],
        ["ready", "text.ready_for_adventure"],
      ];
  }
  if (route.area === "account")
    pairs = [
      ["", "text.profile"],
      ["privacy", "text.privacy"],
      ["linking", "text.link_game_accounts"],
      ["blocks", "text.blocked_players"],
      ["reports", "text.your_reports"],
    ];
  if (route.area === "admin")
    pairs = [
      ["", "text.overview"],
      ["reports", "text.reports"],
      ["ranks", "text.hosting_access_tiers"],
      ["backups", "text.official_backups"],
      ["operations", "text.operations"],
      ["audit", "text.audit_log"],
    ];
  if (route.area === "hosting") {
    pairs = [
      ["servers", "text.your_servers"],
      ["servers/new", "text.create_a_server"],
      ["communities", "text.server_communities"],
    ];
    if (route.id) {
      base = "/hosting/servers/" + route.id;
      pairs = [
        ["", "text.overview"],
        ["console", "text.console"],
        ["logs", "text.logs"],
        ...(server?.can_administer
          ? [
              ["files", "text.files"],
              ["backups", "text.backups"],
              ["members", "text.members"],
              ["settings", "text.settings"],
            ]
          : []),
      ];
    }
  }
  if (route.area === "worlds" && route.id) {
    base = "/worlds/" + route.id;
    pairs = [
      ["", "text.overview"],
      ...(server?.kind === "official"
        ? [
            ["world", "text.world"],
            ["economy", "text.economy"],
          ]
        : []),
    ];
  }
  if ((route.area === "worlds" && !route.id) || route.area === "expeditions") {
    base = "";
    pairs = [
      ["worlds", "text.worlds"],
      ["expeditions", "text.expeditions"],
    ];
  }
  return pairs.map(([path, name]) => ({
    path: base + (path ? "/" + path : ""),
    name: t(name),
  }));
}
