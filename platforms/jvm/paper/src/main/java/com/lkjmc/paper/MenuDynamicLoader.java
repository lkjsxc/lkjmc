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
import com.lkjmc.common.permission.PrincipalIdentity;
import java.util.Optional;
import java.util.concurrent.CompletionException;
import org.bukkit.entity.Player;

final class MenuDynamicLoader {
    private final LkjmcPaperPlugin plugin;
    private final LocaleResolver resolver;
    private final MenuSessionStore sessions;
    private final MenuInventoryRenderer renderer;
    private final MenuDataGateway data;
    private final ProfileMenuDataGateway profileData;
    private final PartyMenuDataGateway partyData;
    private final AdminMenuLoader adminData;
    private final AdminServerMenuLoader adminServers;
    private final AdventureMenuDataGateway adventureData;

    MenuDynamicLoader(LkjmcPaperPlugin plugin, LocaleResolver resolver,
                      MenuSessionStore sessions, MenuInventoryRenderer renderer) {
        this.plugin = plugin;
        this.resolver = resolver;
        this.sessions = sessions;
        this.renderer = renderer;
        this.data = new MenuDataGateway(plugin.daemon());
        this.profileData = new ProfileMenuDataGateway(plugin.daemon());
        this.partyData = new PartyMenuDataGateway(plugin.daemon());
        this.adminData = new AdminMenuLoader(plugin);
        this.adminServers = new AdminServerMenuLoader(data, adminData);
        this.adventureData = new AdventureMenuDataGateway(plugin.daemon());
    }

    void load(Player player, MenuState state, MenuId id) {
        switch (id.value()) {
            case "server-list" -> loadServers(player, state);
            case "homes" -> data.homes(player).whenComplete((v, e) -> reopen(player, state, e, TravelDynamicMenus.homes(v)));
            case "home-create-name" -> data.homes(player).whenComplete((v, e) -> reopen(player, state, e, TravelDynamicMenus.homeCreateName(v)));
            case "home-create-confirm" -> reopen(player, state, null, TravelDynamicMenus.homeCreateConfirm(param(state, "home")));
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
            case "adventures" -> adventureData.catalog(player)
                .whenComplete((v, e) -> reopen(player, state, e, com.lkjmc.common.menu.AdventureDynamicMenus.catalog(v)));
            case "profile" -> profileData.profile(player).whenComplete((v, e) -> reopen(player, state, e, ProfileDynamicMenus.profile(v)));
            case "achievements" -> profileData.achievements(player).whenComplete((v, e) -> reopen(player, state, e, AchievementDynamicMenus.achievements(v)));
            case "party" -> partyData.party(player).whenComplete((v, e) -> reopen(player, state, e, PartyDynamicMenus.party(v)));
            case "party-confirm" -> reopen(player, state, null, PartyDynamicMenus.partyConfirm());
            case "party-invite-picker" -> reopen(player, state, null, picker(player, "party-invite-picker",
                "menu.party.invite.title", MenuTheme.SOCIAL, "party", "party invite"));
            case "teleport-picker" -> reopen(player, state, null, picker(player, "teleport-picker",
                "menu.teleports.picker.title", MenuTheme.TRAVEL, "teleports", "tpa"));
            case "admin-servers", "admin-server-detail", "admin-server-stop-confirm",
                "admin-server-restart-confirm", "admin-server-delete-confirm" ->
                adminServers.load(player, state).whenComplete((v, e) -> reopen(player, state, e, v));
            default -> adminData.load(player, id).ifPresent(spec -> reopen(player, state, null, spec));
        }
    }

    private void loadServers(Player player, MenuState state) {
        var permissions = new ServerMenuPermissions(allowed(player, PermissionNodes.ADMIN_INSTANCE_START),
            allowed(player, PermissionNodes.ADMIN_INSTANCE_STOP));
        data.servers(player).whenComplete((v, e) -> reopen(player, state, e, DynamicMenus.serverList(v, permissions)));
    }

    private void loadReports(Player player, MenuState state) {
        if (!allowed(player, PermissionNodes.ADMIN_REPORTS)) {
            reopen(player, state, null, ReportDynamicMenus.reports(java.util.List.of(), false));
            return;
        }
        data.reports(player).whenComplete((v, e) -> reopen(player, state, e, ReportDynamicMenus.reports(v)));
    }

    private void reopen(Player player, MenuState state, Throwable error, MenuSpec spec) {
        var next = error == null ? spec : unavailable(state.current(), diagnostic(error));
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

    private String diagnostic(Throwable error) {
        var cause = error instanceof CompletionException && error.getCause() != null ? error.getCause() : error;
        return cause instanceof MenuDataException typed ? typed.code() : "daemon.http_failed";
    }

    private MenuSpec unavailable(MenuId id, String code) {
        return switch (id.value()) {
            case "server-list" -> unavailable(id, "menu.server-list.title", MenuTheme.NETWORK, "network", code);
            case "homes" -> unavailable(id, "menu.homes.title", MenuTheme.TRAVEL, "travel", code);
            case "home-create-name", "home-create-confirm" -> unavailable(id, "menu.homes.set", MenuTheme.TRAVEL, "homes", code);
            case "warps" -> unavailable(id, "menu.warps.title", MenuTheme.TRAVEL, "travel", code);
            case "claims" -> unavailable(id, "menu.claims.title", MenuTheme.CLAIMS, "root", code);
            case "claim-detail" -> unavailable(id, "menu.claims.detail.title", MenuTheme.CLAIMS, "claims", code);
            case "claim-confirm" -> unavailable(id, "menu.claims.confirm.title", MenuTheme.CLAIMS, "claim-detail", code);
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
            case "teleport-picker" -> unavailable(id, "menu.teleports.picker.title", MenuTheme.TRAVEL, "teleports", code);
            case "profile" -> unavailable(id, "menu.profile.title", MenuTheme.PROFILE, "root", code);
            case "achievements" -> unavailable(id, "menu.achievements.title", MenuTheme.PROFILE, "profile", code);
            default -> unavailable(id, "menu.root.title", MenuTheme.ROOT, "root", code);
        };
    }

    private MenuSpec unavailable(MenuId id, String title, MenuTheme theme, String back, String code) {
        return UnavailableDynamicMenus.unavailable(id, title, theme, back, code);
    }

    private boolean allowed(Player player, String permission) {
        var platform = player.hasPermission(permission) || player.isOp();
        return plugin.adminGrants().decide(identity(player), permission, platform, player.isOp()).allowed();
    }

    private PrincipalIdentity identity(Player player) {
        return new PrincipalIdentity("minecraft-player", player.getUniqueId().toString(), player.getName());
    }

    private String locale(Player player) {
        return resolver.resolve(Optional.of(player.locale().toLanguageTag()));
    }
}
