package com.lkjmc.paper;

import com.lkjmc.common.i18n.LocaleResolver;
import com.lkjmc.common.menu.AchievementDynamicMenus;
import com.lkjmc.common.menu.ClaimDynamicMenus;
import com.lkjmc.common.menu.DailyDynamicMenus;
import com.lkjmc.common.menu.DynamicMenus;
import com.lkjmc.common.menu.KitDynamicMenus;
import com.lkjmc.common.menu.MailDynamicMenus;
import com.lkjmc.common.menu.MenuDynamicReplacement;
import com.lkjmc.common.menu.MenuId;
import com.lkjmc.common.menu.MenuSpec;
import com.lkjmc.common.menu.MenuState;
import com.lkjmc.common.menu.MenuTheme;
import com.lkjmc.common.menu.PartyDynamicMenus;
import com.lkjmc.common.menu.PlayerMenuEntry;
import com.lkjmc.common.menu.PlayerPickerDynamicMenus;
import com.lkjmc.common.menu.ProfileDynamicMenus;
import com.lkjmc.common.menu.ReportDynamicMenus;
import com.lkjmc.common.menu.ReportMenuEntry;
import com.lkjmc.common.menu.ServerMenuPermissions;
import com.lkjmc.common.menu.ShopDynamicMenus;
import com.lkjmc.common.menu.TravelDynamicMenus;
import com.lkjmc.common.menu.UnavailableDynamicMenus;
import com.lkjmc.common.menu.VoteDynamicMenus;
import com.lkjmc.common.permission.PermissionNodes;
import java.util.Optional;
import org.bukkit.entity.Player;

final class MenuDynamicLoader {
    private final LkjmcPaperPlugin plugin;
    private final LocaleResolver resolver;
    private final MenuSessionStore sessions;
    private final MenuInventoryRenderer renderer;
    private final MenuDataGateway data;
    private final ProfileMenuDataGateway profileData;
    private final PartyMenuDataGateway partyData;

    MenuDynamicLoader(LkjmcPaperPlugin plugin, LocaleResolver resolver,
                      MenuSessionStore sessions, MenuInventoryRenderer renderer) {
        this.plugin = plugin;
        this.resolver = resolver;
        this.sessions = sessions;
        this.renderer = renderer;
        this.data = new MenuDataGateway(plugin.daemon());
        this.profileData = new ProfileMenuDataGateway(plugin.daemon());
        this.partyData = new PartyMenuDataGateway(plugin.daemon());
    }

    void load(Player player, MenuState state, MenuId id) {
        switch (id.value()) {
            case "server-list" -> loadServers(player, state);
            case "homes" -> data.homes(player).whenComplete((v, e) -> reopen(player, state, e, TravelDynamicMenus.homes(v)));
            case "warps" -> data.warps(player).whenComplete((v, e) -> reopen(player, state, e, TravelDynamicMenus.warps(v)));
            case "claims" -> data.claims(player).whenComplete((v, e) -> reopen(player, state, e, ClaimDynamicMenus.claims(v)));
            case "claim-detail" -> reopen(player, state, null, ClaimDynamicMenus.claimDetail(param(state, "name"), longParam(state, "chunkCount")));
            case "claim-confirm" -> reopen(player, state, null, ClaimDynamicMenus.claimConfirm(param(state, "name")));
            case "claim-trust-picker" -> reopen(player, state, null, picker(player, "claim-trust-picker",
                "menu.claims.trust.title", MenuTheme.CLAIMS, "claim-detail", "claim trust " + param(state, "name")));
            case "shop" -> data.shop(player).whenComplete((v, e) -> reopen(player, state, e, ShopDynamicMenus.shop(v)));
            case "kits" -> data.kits(player).whenComplete((v, e) -> reopen(player, state, e, KitDynamicMenus.kits(v)));
            case "votes" -> data.votes(player).whenComplete((v, e) -> reopen(player, state, e, VoteDynamicMenus.votes(v)));
            case "mail" -> data.mail(player).whenComplete((v, e) -> reopen(player, state, e, MailDynamicMenus.mail(v)));
            case "reports" -> loadReports(player, state);
            case "report-detail" -> reopen(player, state, null, ReportDynamicMenus.reportDetail(report(state)));
            case "report-confirm" -> reopen(player, state, null, ReportDynamicMenus.reportConfirm(param(state, "action"), param(state, "reportId")));
            case "daily" -> data.daily(player).whenComplete((v, e) -> reopen(player, state, e, DailyDynamicMenus.daily(v)));
            case "profile" -> profileData.profile(player).whenComplete((v, e) -> reopen(player, state, e, ProfileDynamicMenus.profile(v)));
            case "achievements" -> profileData.achievements(player).whenComplete((v, e) -> reopen(player, state, e, AchievementDynamicMenus.achievements(v)));
            case "party" -> partyData.party(player).whenComplete((v, e) -> reopen(player, state, e, PartyDynamicMenus.party(v)));
            case "party-confirm" -> reopen(player, state, null, PartyDynamicMenus.partyConfirm());
            case "party-invite-picker" -> reopen(player, state, null, picker(player, "party-invite-picker",
                "menu.party.invite.title", MenuTheme.SOCIAL, "party", "party invite"));
            case "teleport-picker" -> reopen(player, state, null, picker(player, "teleport-picker",
                "menu.teleports.picker.title", MenuTheme.TRAVEL, "teleports", "tpa"));
            default -> { }
        }
    }

    private void loadServers(Player player, MenuState state) {
        var permissions = new ServerMenuPermissions(player.hasPermission(PermissionNodes.ADMIN_INSTANCE_START),
            player.hasPermission(PermissionNodes.ADMIN_INSTANCE_STOP));
        data.servers(player).whenComplete((v, e) -> reopen(player, state, e, DynamicMenus.serverList(v, permissions)));
    }

    private void loadReports(Player player, MenuState state) {
        if (!player.hasPermission(PermissionNodes.ADMIN_REPORTS)) {
            reopen(player, state, null, ReportDynamicMenus.reports(java.util.List.of(), false));
            return;
        }
        data.reports(player).whenComplete((v, e) -> reopen(player, state, e, ReportDynamicMenus.reports(v)));
    }

    private void reopen(Player player, MenuState state, Throwable error, MenuSpec spec) {
        var next = error == null ? spec : unavailable(state.current());
        plugin.scheduler().runPlayer(player, () -> sessions.state(player)
            .filter(current -> MenuDynamicReplacement.accepts(current, state))
            .ifPresent(current -> {
                var refreshed = sessions.replaceDynamic(player);
                player.openInventory(renderer.render(locale(player), next, refreshed));
            }));
    }

    private MenuSpec picker(Player player, String id, String title, MenuTheme theme, String back, String command) {
        var players = plugin.getServer().getOnlinePlayers().stream()
            .filter(candidate -> !candidate.getUniqueId().equals(player.getUniqueId()))
            .map(candidate -> new PlayerMenuEntry(candidate.getName()))
            .toList();
        return PlayerPickerDynamicMenus.picker(id, title, theme, back, command, players);
    }

    private ReportMenuEntry report(MenuState state) {
        return new ReportMenuEntry(param(state, "reportId"), param(state, "serverId"),
            param(state, "reason"), param(state, "status"));
    }

    private String param(MenuState state, String key) {
        return state.route().params().getOrDefault(key, "");
    }

    private long longParam(MenuState state, String key) {
        try {
            return Long.parseLong(param(state, key));
        } catch (NumberFormatException ignored) {
            return 0;
        }
    }

    private MenuSpec unavailable(MenuId id) {
        return switch (id.value()) {
            case "server-list" -> unavailable(id, "menu.server-list.title", MenuTheme.NETWORK, "network");
            case "homes" -> unavailable(id, "menu.homes.title", MenuTheme.TRAVEL, "travel");
            case "warps" -> unavailable(id, "menu.warps.title", MenuTheme.TRAVEL, "travel");
            case "claims" -> unavailable(id, "menu.claims.title", MenuTheme.CLAIMS, "root");
            case "claim-detail" -> unavailable(id, "menu.claims.detail.title", MenuTheme.CLAIMS, "claims");
            case "claim-confirm" -> unavailable(id, "menu.claims.confirm.title", MenuTheme.CLAIMS, "claim-detail");
            case "claim-trust-picker" -> unavailable(id, "menu.claims.trust.title", MenuTheme.CLAIMS, "claim-detail");
            case "shop" -> unavailable(id, "menu.shop.title", MenuTheme.ECONOMY, "economy");
            case "kits" -> unavailable(id, "menu.kits.title", MenuTheme.ECONOMY, "economy");
            case "votes" -> unavailable(id, "menu.votes.title", MenuTheme.ECONOMY, "economy");
            case "daily" -> unavailable(id, "menu.daily.title", MenuTheme.ECONOMY, "economy");
            case "mail" -> unavailable(id, "menu.mail.title", MenuTheme.SOCIAL, "social");
            case "reports" -> unavailable(id, "menu.reports.title", MenuTheme.SOCIAL, "social");
            case "report-detail" -> unavailable(id, "menu.reports.detail.title", MenuTheme.SOCIAL, "reports");
            case "report-confirm" -> unavailable(id, "menu.reports.confirm.title", MenuTheme.SOCIAL, "report-detail");
            case "party" -> unavailable(id, "menu.party.title", MenuTheme.SOCIAL, "social");
            case "party-confirm" -> unavailable(id, "menu.party.confirm.title", MenuTheme.SOCIAL, "party");
            case "party-invite-picker" -> unavailable(id, "menu.party.invite.title", MenuTheme.SOCIAL, "party");
            case "teleport-picker" -> unavailable(id, "menu.teleports.picker.title", MenuTheme.TRAVEL, "teleports");
            case "profile" -> unavailable(id, "menu.profile.title", MenuTheme.PROFILE, "root");
            case "achievements" -> unavailable(id, "menu.achievements.title", MenuTheme.PROFILE, "profile");
            default -> unavailable(id, "menu.root.title", MenuTheme.ROOT, "root");
        };
    }

    private MenuSpec unavailable(MenuId id, String title, MenuTheme theme, String back) {
        return UnavailableDynamicMenus.unavailable(id, title, theme, back);
    }

    private String locale(Player player) {
        return resolver.resolve(Optional.of(player.locale().toLanguageTag()));
    }
}
