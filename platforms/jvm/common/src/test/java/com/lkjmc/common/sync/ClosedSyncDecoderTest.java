package com.lkjmc.common.sync;

import static org.junit.jupiter.api.Assertions.*;

import com.google.gson.JsonParser;
import com.lkjmc.bindings.*;
import java.util.List;
import org.junit.jupiter.api.Test;

final class ClosedSyncDecoderTest {
    private static final String PLAYER = "4a1f2b5c-2a1e-4c7a-8b6d-111111111111";
    private final ClosedSyncDecoder decoder = new ClosedSyncDecoder();

    @Test
    void decodesEveryDomainAndEveryResultVariantToGeneratedRecords() {
        List<String[]> domains = List.of(
                row("permissions", "player:a", "{\"principalKind\":\"player\",\"principalId\":\"a\","
                        + "\"grants\":[],\"permissions\":[]}", PermissionSnapshot.class),
                row("claims", "survival", "{\"chunks\":[]}", ClaimSnapshot.class),
                row("menus", "global", "{\"shop\":[],\"kits\":[],\"votes\":[],\"plugins\":[]}",
                        MenuSnapshot.class),
                row("profiles", PLAYER + ":profile", "{\"playerUuid\":\"" + PLAYER
                        + "\",\"scope\":\"profile\",\"profile\":null}", ProfileSnapshot.class),
                row("presence", "hub", "{\"instanceId\":\"hub\",\"available\":false}",
                        PresenceSnapshot.class),
                row("routing", "network", "{\"instances\":[]}", RoutingSnapshot.class),
                row("settings", PLAYER, "{\"playerUuid\":\"" + PLAYER
                        + "\",\"language\":\"en\",\"menuEnabled\":true,\"hudEnabled\":true,"
                        + "\"tipsEnabled\":true,\"privacy\":{}}", SettingsSnapshot.class));
        for (String[] domain : domains) {
            assertInstanceOf(classFor(domain[3]), decoder.decode(snapshot(domain[0], domain[1], domain[2])));
        }
        assertInstanceOf(ProfileAvailable.class,
                ((ProfileSnapshot) decoder.decode(snapshot("profiles", PLAYER + ":profile",
                        availableProfile()))).payload());
        assertInstanceOf(PresenceAvailable.class,
                ((PresenceSnapshot) decoder.decode(snapshot("presence", "hub",
                        "{\"instanceId\":\"hub\",\"playerCount\":1,\"maxPlayers\":20,"
                        + "\"ready\":true,\"lastHeartbeatAt\":\"2026-07-14T00:00:00Z\","
                        + "\"suspendReason\":null}"))).payload());
        assertInstanceOf(FeedResponse.class, decode("{\"result\":\"changes\",\"cursor\":2,"
                + "\"activeFloor\":1,\"credentialRevision\":1,\"changes\":[{\"feedRevision\":2,"
                + "\"domain\":\"routing\",\"key\":\"network\",\"revision\":3}]}"));
        assertInstanceOf(ReloadRequired.class, decode("{\"result\":\"reload-required\","
                + "\"cursor\":2,\"activeFloor\":2,\"credentialRevision\":1}"));
        assertInstanceOf(SnapshotUnavailable.class, decode("{\"result\":\"unavailable\","
                + "\"domain\":\"routing\",\"key\":\"network\",\"credentialRevision\":1,"
                + "\"reason\":\"missing\"}"));
        assertInstanceOf(SyncUnavailable.class, decode("{\"result\":\"unavailable\","
                + "\"error\":{\"code\":\"sync.unavailable\"}}"));
    }

    @Test
    void unknownMissingAndWrongFieldsFailClosed() {
        String valid = "{\"playerUuid\":\"" + PLAYER + "\",\"language\":\"en\","
                + "\"menuEnabled\":true,\"hudEnabled\":true,\"tipsEnabled\":true,\"privacy\":{}}";
        assertThrows(IllegalArgumentException.class, () -> decoder.decode(snapshot("settings", PLAYER,
                valid.substring(0, valid.length() - 1) + ",\"unknown\":1}")));
        assertThrows(IllegalArgumentException.class, () -> decoder.decode(snapshot("settings", PLAYER,
                valid.replace("\"language\":\"en\",", ""))));
        assertThrows(IllegalArgumentException.class, () -> decoder.decode(snapshot("settings", PLAYER,
                valid.replace("\"menuEnabled\":true", "\"menuEnabled\":\"true\""))));
    }

    private static String availableProfile() {
        return "{\"playerUuid\":\"" + PLAYER + "\",\"scope\":\"profile\",\"profileRevision\":1,"
                + "\"schema\":\"lkjmc-profile-one\",\"sha256\":\"" + "0".repeat(64)
                + "\",\"envelope\":{\"schema\":\"lkjmc-profile-one\",\"inventory\":[],"
                + "\"armor\":[],\"offhand\":null,\"selectedHotbarSlot\":0,\"enderChest\":[],"
                + "\"experience\":{\"progress\":0,\"level\":0,\"total\":0},"
                + "\"vitals\":{\"health\":20,\"food\":20,\"saturation\":5,\"air\":300},"
                + "\"potionEffects\":[],\"gameMode\":null,\"pluginData\":[],\"homes\":[],"
                + "\"warps\":[],\"points\":0,\"achievements\":[],\"settings\":{"
                + "\"menuEnabled\":true,\"hudEnabled\":true,\"tipsEnabled\":true,"
                + "\"privacy\":\"default\"},\"language\":\"en\"}}";
    }

    private static String[] row(String domain, String key, String payload, Class<?> type) {
        return new String[] {domain, key, payload, type.getName()};
    }

    private static Class<?> classFor(String name) {
        try { return Class.forName(name); }
        catch (ClassNotFoundException impossible) { throw new AssertionError(impossible); }
    }

    private static com.google.gson.JsonObject snapshot(String domain, String key, String payload) {
        return JsonParser.parseString("{\"result\":\"snapshot\",\"domain\":\"" + domain
                + "\",\"key\":\"" + key + "\",\"revision\":1,"
                + "\"generatedAt\":\"2026-07-14T00:00:00Z\",\"credentialRevision\":1,"
                + "\"payload\":" + payload + "}").getAsJsonObject();
    }

    private SyncResponse decode(String value) {
        return decoder.decode(JsonParser.parseString(value).getAsJsonObject());
    }
}
