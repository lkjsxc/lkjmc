import { Fragment } from "react";
import type { Data } from "./api";
import { t } from "./i18n";
import { topPages, type Route } from "./routes";

/** The hierarchy is navigation, not a second page heading. Native links keep
 * keyboard, context-menu and open-in-new-tab behavior on every ancestor. */
export function Breadcrumbs({ route, data }: { route: Route; data: Data }) {
  const root = topPages().find((page) => page.id === route.area);
  const items = [
    {
      label: root?.name ?? t("text.account"),
      path: root?.path ?? "/account",
    },
  ];
  if (route.area === "people" && route.component === "teams") {
    items.push({ label: t("text.teams"), path: "/people/teams" });
    if (route.id) {
      items.push({
        label: data.team?.name ?? t("text.team"),
        path: "/people/teams/" + route.id,
      });
      if (route.section !== "team")
        items.push({ label: t(route.title), path: route.path });
    }
  } else if (route.id && ["worlds", "hosting"].includes(route.area)) {
    const base = route.area === "hosting" ? "/hosting/servers/" : "/worlds/";
    items.push({
      label: data.server?.name ?? t("text.server"),
      path: base + route.id,
    });
    if (route.component === "expeditions" && route.section !== "expeditions")
      items.push({
        label: t("text.expeditions"),
        path: base + route.id + "/expeditions",
      });
    if (route.component !== "world" || route.section !== "overview")
      items.push({ label: t(route.title), path: route.path });
  } else if (route.path.split("?")[0] !== items[0].path) {
    items.push({ label: t(route.title), path: route.path });
  }
  return (
    <nav className="breadcrumbs" aria-label={t("text.breadcrumbs")}>
      {items.map((item, index) => (
        <Fragment key={index}>
          {index > 0 && <span aria-hidden="true">/</span>}
          {index === items.length - 1 ? (
            <span aria-current="page">{item.label}</span>
          ) : (
            <a href={"#" + item.path}>{item.label}</a>
          )}
        </Fragment>
      ))}
    </nav>
  );
}
