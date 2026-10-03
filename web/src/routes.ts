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
  ["home", "Home", "Invitations and activity", "home"],
  ["servers", "Servers", "Status and connection details", "play"],
  ["friends", "Friends", "Friends and requests", "social"],
  ["chat", "Chat", "Private and group conversations", "chat"],
  ["teams", "Teams", "Shared land, coins and permissions", "teams"],
  ["parties", "Parties", "Invitations and adventure readiness", "parties"],
  ["manage", "Manage servers", "Settings, files and backups", "servers"],
  ["account", "Account", "Linked accounts and privacy", "settings"],
  ["admin", "Administration", "Access, reports and service status", "admin"],
];
export const topPages = () =>
  roots.map(([id, name, description, icon]) => ({
    id,
    name: t(name),
    description: t(description),
    icon,
    path: id === "manage" ? "/manage/servers" : "/" + id,
  }));
const aliases: Record<string, string> = {
  home: "/home",
  play: "/servers",
  servers: "/manage/servers",
  social: "/friends",
  settings: "/account",
  admin: "/admin",
  smp: "/servers/official",
  life: "/servers/official/land",
  market: "/servers/official/market",
  adventure: "/servers/official/end",
};
export const normalize = (path: string) =>
  aliases[path] ?? (path.startsWith("/") ? path : "/home");

export function resolveRoute(raw: string): Route {
  const url = new URL(normalize(raw), location.origin);
  const parts = url.pathname.split("/").filter(Boolean);
  const [area, second, third, fourth] = parts;
  const root = roots.find((r) => r[0] === area);
  const result: Route = {
    path: url.pathname + url.search,
    area: area ?? "home",
    section: "overview",
    component: "missing",
    title: root ? root[1] : "Page not found",
    description: root ? root[2] : "Choose a page from the menu.",
  };
  const set = (
    section: string,
    component: string,
    title: string,
    api?: string,
  ) => Object.assign(result, { section, component, title, api });
  if (
    area === "home" &&
    parts.length <= 2 &&
    (!second || ["notifications", "invitations", "activity"].includes(second))
  ) {
    set(
      second ?? "overview",
      second ? "feed" : "home",
      second
        ? {
            notifications: "Notifications",
            invitations: "Invitations",
            activity: "Recent actions",
          }[second]!
        : "Home",
      second ? "/api/v1/history/" + second + url.search : "/api/v1/home",
    );
  } else if (area === "servers" && parts.length === 1)
    set("list", "play", "Servers", "/api/v1/view/play");
  else if (
    area === "servers" &&
    second &&
    parts.length <= 3 &&
    (second === "official" || /^[0-9a-f-]{36}$/.test(second))
  ) {
    result.id = second;
    const section = third ?? "overview";
    const choices: Record<string, [string, string]> = {
      overview: ["server", "Server details"],
      land: ["life", "Protected land"],
      homes: ["life", "Homes"],
      coins: ["life", "Land & assets"],
      "coin-history": ["life", "Coin history"],
      achievements: ["life", "Achievements"],
      meetup: ["life", "Meet up"],
      market: ["market", "Market"],
      "stored-assets": ["market", "Stored assets"],
      materials: ["market", "Sell materials"],
      end: ["adventure", "Private End"],
    };
    if (choices[section])
      set(
        section,
        choices[section][0],
        choices[section][1],
        "/api/v1/servers/" + second + "?section=" + section,
      );
  } else if (area === "manage" && second === "servers" && parts.length <= 4) {
    if (!third)
      set("list", "managed-list", "Manage servers", "/api/v1/view/servers");
    else if (third === "new" && !fourth)
      set("new", "create-server", "Create a server", "/api/v1/server-presets");
    else if (/^[0-9a-f-]{36}$/.test(third)) {
      result.id = third;
      const section = fourth ?? "overview";
      const names: Record<string, string> = {
        overview: "Overview",
        console: "Console and logs",
        files: "Files",
        backups: "Backups",
        members: "Members and permissions",
        settings: "Server settings",
        activity: "Recent actions",
      };
      if (names[section])
        set(
          "manage-" + section,
          "managed-server",
          names[section],
          "/api/v1/servers/" + third + "?section=manage-" + section,
        );
    }
  } else if (
    area === "manage" &&
    second === "communities" &&
    parts.length === 2
  )
    set(
      "communities",
      "social",
      "Server communities",
      "/api/v1/view/social?section=communities",
    );
  else if (
    ["friends", "chat", "teams", "parties"].includes(area) &&
    parts.length <= 2
  ) {
    const options: Record<string, Record<string, string>> = {
      friends: {
        overview: "Friends",
        incoming: "Incoming friend requests",
        outgoing: "Sent friend requests",
      },
      teams: {
        overview: "Teams",
        members: "Members and permissions",
        settings: "Team settings",
      },
      parties: {
        overview: "Parties",
        members: "Members",
        ready: "Ready for adventure",
      },
    };
    if (area === "chat" && (!second || /^[0-9a-f-]{36}$/.test(second))) {
      result.id = second;
      set("chat", "social", "Chat", "/api/v1/view/social?section=chat");
    } else if (options[area]?.[second ?? "overview"])
      set(
        area === "friends"
          ? (second ?? "friends")
          : area === "teams"
            ? "team" + (second ? "-" + second : "")
            : "party" + (second ? "-" + second : ""),
        "social",
        options[area][second ?? "overview"],
        "/api/v1/view/social?section=" + area,
      );
  } else if (area === "account" && parts.length <= 2) {
    const names: Record<string, string> = {
      overview: "Account",
      profile: "Profile",
      privacy: "Privacy",
      linking: "Link game accounts",
      blocks: "Blocked players",
      reports: "Your reports",
    };
    if (names[second ?? "overview"])
      set(
        second ?? "profile",
        "settings",
        names[second ?? "overview"],
        "/api/v1/view/settings?section=" + (second ?? "profile"),
      );
  } else if (area === "admin" && parts.length <= 2) {
    const names: Record<string, string> = {
      overview: "Administration",
      reports: "Reports",
      ranks: "Hosting access tiers",
      backups: "Official backups",
      jobs: "Actions needing attention",
      audit: "Audit log",
    };
    if (names[second ?? "overview"])
      set(
        second ?? "overview",
        second ? "admin" : "admin-home",
        names[second ?? "overview"],
        "/api/v1/view/admin?section=" + (second ?? "overview"),
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
  if (route.area === "home")
    pairs = [
      ["", "Overview"],
      ["invitations", "Invitations"],
      ["notifications", "Notifications"],
      ["activity", "Recent actions"],
    ];
  if (route.area === "friends")
    pairs = [
      ["", "Friends"],
      ["incoming", "Incoming friend requests"],
      ["outgoing", "Sent friend requests"],
    ];
  if (route.area === "teams")
    pairs = [
      ["", "Overview"],
      ["members", "Members and permissions"],
      ["settings", "Team settings"],
    ];
  if (route.area === "parties")
    pairs = [
      ["", "Overview"],
      ["members", "Members"],
      ["ready", "Ready for adventure"],
    ];
  if (route.area === "account")
    pairs = [
      ["", "Profile"],
      ["privacy", "Privacy"],
      ["linking", "Link game accounts"],
      ["blocks", "Blocked players"],
      ["reports", "Your reports"],
    ];
  if (route.area === "admin")
    pairs = [
      ["", "Overview"],
      ["reports", "Reports"],
      ["ranks", "Hosting access tiers"],
      ["backups", "Official backups"],
      ["jobs", "Actions needing attention"],
      ["audit", "Audit log"],
    ];
  if (route.area === "manage") {
    base = "/manage";
    pairs = [
      ["servers", "Manage servers"],
      ["servers/new", "Create a server"],
      ["communities", "Server communities"],
    ];
    if (route.id) {
      base = "/manage/servers/" + route.id;
      pairs = [
        ["", "Overview"],
        ["console", "Console and logs"],
        ["activity", "Recent actions"],
        ...(server?.can_administer
          ? [
              ["files", "Files"],
              ["backups", "Backups"],
              ["members", "Members and permissions"],
              ["settings", "Server settings"],
            ]
          : []),
      ];
    }
  }
  if (route.area === "servers" && route.id) {
    base = "/servers/" + route.id;
    pairs = [
      ["", "Overview"],
      ...(server?.kind === "official"
        ? [
            ["coins", "Land & assets"],
            ["land", "Protected land"],
            ["homes", "Homes"],
            ["meetup", "Meet up"],
            ["achievements", "Achievements"],
            ["coin-history", "Coin history"],
            ["market", "Market"],
            ["stored-assets", "Stored assets"],
            ["materials", "Sell materials"],
            ["end", "Private End"],
          ]
        : []),
    ];
  }
  return pairs.map(([path, name]) => ({
    path: base + (path ? "/" + path : ""),
    name: t(name),
  }));
}
