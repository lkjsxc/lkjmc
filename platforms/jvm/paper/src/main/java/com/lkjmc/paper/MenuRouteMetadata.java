package com.lkjmc.paper;

import com.lkjmc.common.menu.MenuId;
import com.lkjmc.common.menu.MenuSpec;
import com.lkjmc.common.menu.MenuTheme;
import com.lkjmc.common.menu.UnavailableDynamicMenus;

final class MenuRouteMetadata {
    private MenuRouteMetadata() {}

    static MenuSpec unavailable(MenuId id, String code) {
        return switch (id.value()) {
            case "server-list" -> unavailable(id, "menu.server-list.title", MenuTheme.NETWORK, "network", code);
            case "homes" -> unavailable(id, "menu.homes.title", MenuTheme.TRAVEL, "travel", code);
            case "home-detail" -> unavailable(id, "menu.homes.detail.title", MenuTheme.TRAVEL, "homes", code);
            case "home-create-name", "home-create-confirm" -> unavailable(id, "menu.homes.set", MenuTheme.TRAVEL, "homes", code);
            case "home-update-confirm" -> unavailable(id, "menu.homes.update.confirm", MenuTheme.TRAVEL, "home-detail", code);
            case "home-delete-confirm" -> unavailable(id, "menu.homes.delete.confirm", MenuTheme.TRAVEL, "home-detail", code);
            case "warps" -> unavailable(id, "menu.warps.title", MenuTheme.TRAVEL, "travel", code);
            case "claims" -> unavailable(id, "menu.claims.title", MenuTheme.CLAIMS, "root", code);
            case "claim-detail" -> unavailable(id, "menu.claims.detail.title", MenuTheme.CLAIMS, "claims", code);
            case "claim-confirm" -> unavailable(id, "menu.claims.confirm.title", MenuTheme.CLAIMS, "claim-detail", code);
            case "claim-create-confirm" -> unavailable(id, "menu.claims.confirm.title", MenuTheme.CLAIMS, "claims", code);
            case "claim-trust-picker" -> unavailable(id, "menu.claims.trust.title", MenuTheme.CLAIMS, "claim-detail", code);
            case "shop" -> unavailable(id, "menu.shop.title", MenuTheme.ECONOMY, "economy", code);
            case "kits" -> unavailable(id, "menu.kits.title", MenuTheme.ECONOMY, "economy", code);
            case "votes" -> unavailable(id, "menu.votes.title", MenuTheme.ECONOMY, "economy", code);
            case "daily" -> unavailable(id, "menu.daily.title", MenuTheme.ECONOMY, "economy", code);
            case "adventures" -> unavailable(id, "menu.adventures.title", MenuTheme.ROOT, "root", code);
            case "mail" -> unavailable(id, "menu.mail.title", MenuTheme.SOCIAL, "social", code);
            case "reports" -> unavailable(id, "menu.reports.title", MenuTheme.SOCIAL, "social", code);
            case "report-detail" -> unavailable(id, "menu.reports.detail.title", MenuTheme.SOCIAL, "reports", code);
            case "report-confirm" -> unavailable(id, "menu.reports.confirm.title", MenuTheme.SOCIAL, "report-detail", code);
            case "party" -> unavailable(id, "menu.party.title", MenuTheme.SOCIAL, "social", code);
            case "party-confirm" -> unavailable(id, "menu.party.confirm.title", MenuTheme.SOCIAL, "party", code);
            case "party-invite-picker" -> unavailable(id, "menu.party.invite.title", MenuTheme.SOCIAL, "party", code);
            case "random-teleport-overworld", "random-teleport-nether-confirm", "random-teleport-end-confirm" ->
                unavailable(id, "menu.random-teleport.title", MenuTheme.TRAVEL, "teleports", code);
            case "teleport-picker" -> unavailable(id, "menu.teleports.picker.title", MenuTheme.TRAVEL, "teleports", code);
            case "profile" -> unavailable(id, "menu.profile.title", MenuTheme.PROFILE, "root", code);
            case "achievements" -> unavailable(id, "menu.achievements.title", MenuTheme.PROFILE, "profile", code);
            case "achievement-directory" -> unavailable(id, "menu.achievements.directory.title", MenuTheme.PROFILE, "achievements", code);
            case "achievement-detail" -> unavailable(id, "menu.achievements.detail.title", MenuTheme.PROFILE, "achievement-directory", code);
            default -> unavailable(id, "menu.root.title", MenuTheme.ROOT, "root", code);
        };
    }

    static String rtpProfile(MenuId id) {
        return switch (id.value()) {
            case "random-teleport-nether-confirm" -> "nether";
            case "random-teleport-end-confirm" -> "end";
            default -> "overworld";
        };
    }

    private static MenuSpec unavailable(MenuId id, String title, MenuTheme theme, String back, String code) {
        return UnavailableDynamicMenus.unavailable(id, title, theme, back, code);
    }
}
