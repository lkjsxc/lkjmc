package com.lkjmc.common.ui.binding;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertInstanceOf;
import static org.junit.jupiter.api.Assertions.assertThrows;

import java.util.Map;
import java.util.stream.Stream;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.Arguments;
import org.junit.jupiter.params.provider.MethodSource;
import org.junit.jupiter.api.Test;

final class BindingDecodeFixturesTest {
    private final BindingRegistry registry = BindingRegistry.standard();

    @ParameterizedTest
    @MethodSource("happyCases")
    void decodesHappyFixtures(String binding, String fixture, BindingContext ctx) {
        var result = registry.require(binding).decode(BindingTestSupport.json(fixture), ctx);
        assertInstanceOf(BindingResult.Data.class, result, binding);
    }

    @ParameterizedTest
    @MethodSource("emptyCases")
    void emptyArraysReturnEmpty(String binding, String fixture) {
        var result = registry.require(binding).decode(BindingTestSupport.json(fixture), BindingTestSupport.ctx());
        assertInstanceOf(BindingResult.Empty.class, result, binding);
    }

    @Test
    void missingRequiredFieldThrowsTypedDecodeCode() {
        var error = assertThrows(BindingDecodeException.class,
            () -> registry.require("server-list").decode(BindingTestSupport.json("empty.json"), BindingTestSupport.ctx()));
        assertEquals("menu.decode.server-list", error.code());

        var settings = assertThrows(BindingDecodeException.class,
            () -> registry.require("settings").decode(BindingTestSupport.json("empty.json"), BindingTestSupport.ctx()));
        assertEquals("menu.decode.settings", settings.code());
    }

    static Stream<Arguments> happyCases() {
        return Stream.of(
            c("server-list", "instance-list.json"), c("admin-servers", "instance-list.json"),
            c("admin-server-detail", "instance-list.json", Map.of("serverId", "survival")),
            c("admin-server-create-kind", "empty.json"),
            c("admin-server-create-template", "instance-create-plan.json", Map.of("kind", "paper")),
            c("admin-config", "status.json"), c("admin-web", "status.json"),
            c("admin-economy", "shop-combined.json"), c("admin-moderation", "player-report-list.json"),
            c("admin-security", "admin-security.json"), c("admin-audit", "admin-audit-tail.json"),
            c("homes", "player-home-list.json"), c("home-detail", "player-home-get.json", Map.of("home", "base")),
            c("warps", "player-warp-list.json"), c("teleports", "empty.json"),
            c("random-teleport", "player-random-teleport-quote.json", Map.of("profileId", "overworld")),
            c("claims", "claim-list.json"), c("claim-detail", "claim-snapshot.json", Map.of("claimId", "claim-1")),
            local("claim-trust-picker", "empty.json", Map.of("claimId", "claim-1")),
            c("shop", "shop-combined.json"), c("kits", "player-kit-list.json"),
            c("votes", "player-vote-list.json"), c("daily", "player-daily-status.json"),
            c("mail", "player-mail-inbox.json"), c("party", "player-party-info.json"),
            local("party-invite-picker", "empty.json", Map.of()),
            c("reports", "player-report-list.json"), c("report-detail", "player-report-list.json", Map.of("reportId", "report-1")),
            c("profile", "profile-combined.json"), c("achievements", "player-achievements-list.json"),
            c("achievement-directory", "player-achievements-list.json", Map.of("path", "claimable")),
            c("achievement-detail", "player-achievements-list.json", Map.of("id", "first-home")),
            c("adventures", "adventure-catalog.json"), c("settings", "player-settings-get.json"),
            local("teleport-picker", "empty.json", Map.of()),
            local("docs-directory", "empty.json", Map.of("path", "guide")),
            local("docs-file", "empty.json", Map.of("path", "guide/start.md", "page", "0")),
            local("docs-search", "empty.json", Map.of("query", "start")));
    }

    static Stream<Arguments> emptyCases() {
        return Stream.of(Arguments.of("server-list", "instance-list-empty.json"),
            Arguments.of("homes", "player-home-list-empty.json"), Arguments.of("shop", "shop-empty.json"));
    }

    private static Arguments c(String binding, String fixture) {
        return c(binding, fixture, Map.of());
    }

    private static Arguments c(String binding, String fixture, Map<String, String> params) {
        return Arguments.of(binding, fixture, BindingTestSupport.ctx(params));
    }

    private static Arguments local(String binding, String fixture, Map<String, String> params) {
        return Arguments.of(binding, fixture,
            BindingTestSupport.ctx(params, PermissionsView.all(), BindingTestSupport.local()));
    }
}
