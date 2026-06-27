package com.lkjmc.common.menu;

import static com.lkjmc.common.menu.MenuSpecAssertions.actionAt;
import static com.lkjmc.common.menu.MenuSpecAssertions.slotAt;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;

import java.util.List;
import org.junit.jupiter.api.Test;

final class DynamicMenuBackActionSpecTest {
    @Test
    void everyVisibleSlot49MenuBackUsesBackAction() {
        for (var spec : backSpecs()) {
            var slot = slotAt(spec, 49);
            assertEquals("menu.back", slot.item().nameKey(), spec.id().value());
            assertEquals(new MenuAction.Back(), slot.action(), spec.id().value());
            assertFalse(slot.action() instanceof MenuAction.OpenRoute, spec.id().value());
        }
    }

    @Test
    void backClicksProduceOpenPreviousEffects() {
        for (var spec : backSpecs()) {
            var decision = MenuReducer.click(spec, new MenuState(spec.id(), 0), new MenuClick(49, "back", true));
            assertEquals(new MenuEffect.OpenPrevious(), decision.effects().get(0), spec.id().value());
        }
    }

    @Test
    void homesBackIsTheSameForLoadingLoadedAndUnavailableStates() {
        var loading = LoadingDynamicMenus.loading(new MenuId("homes"), "menu.homes.title", MenuTheme.TRAVEL, "travel");
        var loaded = TravelDynamicMenus.homes(List.of(new TravelMenuEntry("base", "hub")));
        var unavailable = UnavailableDynamicMenus.unavailable(new MenuId("homes"), "menu.homes.title", MenuTheme.TRAVEL, "travel");
        for (var spec : List.of(loading, loaded, unavailable)) {
            assertEquals(new MenuAction.Back(), actionAt(spec, 49));
        }
    }

    @Test
    void confirmationCancelUsesHistoryBack() {
        var spec = StandardMenus.confirmation(new ConfirmationSpec(new MenuId("claim-confirm"),
            "menu.claims.confirm.delete", new MenuAction.RunPlayerCommand("claim delete base")));
        assertEquals(new MenuAction.Back(), actionAt(spec, 15));
    }

    private static List<MenuSpec> backSpecs() {
        return List.of(
            StandardMenus.network(), StandardMenus.travel(), StandardMenus.claims(), StandardMenus.economy(),
            StandardMenus.shopList(), StandardMenus.shopDetail(), StandardMenus.kits(), StandardMenus.daily(),
            StandardMenus.votes(), StandardMenus.social(), StandardMenus.party(), StandardMenus.mail(),
            StandardMenus.reports(), StandardMenus.profile(), StandardMenus.achievements(), StandardMenus.settings(),
            StandardMenus.language(), StandardMenus.serverList(), DynamicMenus.serverList(List.of()),
            TravelDynamicMenus.homes(List.of(new TravelMenuEntry("base", "hub"))), TravelDynamicMenus.warps(List.of()),
            TeleportDynamicMenus.teleports(), PlayerPickerDynamicMenus.picker("teleport-picker",
                "menu.teleports.picker.title", MenuTheme.TRAVEL, "teleports", "tpa", List.of()),
            ClaimDynamicMenus.claims(List.of()), ClaimDynamicMenus.claimDetail("base", 2),
            ShopDynamicMenus.shop(List.of()), KitDynamicMenus.kits(List.of()), DailyDynamicMenus.daily(null),
            VoteDynamicMenus.votes(List.of()), MailDynamicMenus.mail(List.of()), PartyDynamicMenus.party(null),
            ReportDynamicMenus.reports(List.of()), ReportDynamicMenus.reportDetail(new ReportMenuEntry("r1", "hub", "spam", "open")),
            ProfileDynamicMenus.profile(null), AchievementDynamicMenus.achievements(List.of()),
            LoadingDynamicMenus.loading(new MenuId("homes"), "menu.homes.title", MenuTheme.TRAVEL, "travel"),
            UnavailableDynamicMenus.unavailable(new MenuId("homes"), "menu.homes.title", MenuTheme.TRAVEL, "travel")
        );
    }
}
