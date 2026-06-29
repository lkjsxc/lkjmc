package com.lkjmc.paper;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertIterableEquals;

import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import com.lkjmc.common.daemon.DaemonClient;
import com.lkjmc.common.daemon.DaemonResponse;
import java.lang.reflect.Proxy;
import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import java.util.UUID;
import java.util.concurrent.CompletableFuture;
import org.bukkit.entity.Player;
import org.junit.jupiter.api.Test;

final class MenuDataGatewayTest {
    private static final UUID PLAYER_ID = UUID.fromString("00000000-0000-0000-0000-000000000123");

    @Test
    void parsesEveryMenuDataCommandShape() {
        var daemon = new FakeDaemon();
        var player = player();
        var gateway = new MenuDataGateway(Optional.of(daemon));
        var profile = new ProfileMenuDataGateway(Optional.of(daemon));
        var party = new PartyMenuDataGateway(Optional.of(daemon));

        assertEquals("hub", gateway.servers(player).join().get(0).id());
        assertEquals("base", gateway.homes(player).join().get(0).name());
        assertEquals("spawn", gateway.warps(player).join().get(0).name());
        assertEquals("claim", gateway.claims(player).join().get(0).name());
        assertEquals("apple", gateway.shop(player).join().get(0).id());
        assertEquals("daily", gateway.kits(player).join().get(0).id());
        assertEquals("site", gateway.votes(player).join().get(0).id());
        assertEquals("mail-1", gateway.mail(player).join().get(0).id());
        assertEquals("report-1", gateway.reports(player).join().get(0).id());
        assertEquals(100, gateway.daily(player).join().points());
        assertEquals(5, profile.profile(player).join().pointsBalance());
        assertEquals("first-home", profile.achievements(player).join().get(0).id());
        assertEquals("Raiders", party.party(player).join().name());
        assertIterableEquals(expectedCommands(), daemon.commands);
    }

    private static List<String> expectedCommands() {
        return List.of("instance.list", "player.home.list", "player.warp.list", "claim.list",
            "player.shop.list", "player.kit.list", "player.vote.list", "player.mail.inbox",
            "player.report.list", "player.daily.status", "player.points.balance", "player.achievements.list",
            "player.achievements.list", "player.party.info");
    }

    private static Player player() {
        return proxy(Player.class, (proxy, method, args) -> switch (method.getName()) {
            case "getUniqueId" -> PLAYER_ID;
            case "getName" -> "Alex";
            default -> fallback(method.getReturnType());
        });
    }

    private static JsonObject json(String value) {
        return JsonParser.parseString(value).getAsJsonObject();
    }

    private static Object fallback(Class<?> type) {
        if (type.equals(boolean.class)) return false;
        if (type.equals(int.class)) return 0;
        if (type.equals(void.class)) return null;
        return null;
    }

    @SuppressWarnings("unchecked")
    private static <T> T proxy(Class<T> type, java.lang.reflect.InvocationHandler handler) {
        return (T) Proxy.newProxyInstance(type.getClassLoader(), new Class<?>[] {type}, handler);
    }

    private static final class FakeDaemon implements DaemonClient {
        private final List<String> commands = new ArrayList<>();

        @Override
        public CompletableFuture<DaemonResponse> send(com.lkjmc.common.daemon.DaemonRequest request) {
            commands.add(request.command());
            return CompletableFuture.completedFuture(new DaemonResponse(
                request.requestId(), true, body(request.command()), Optional.empty()));
        }

        private JsonObject body(String command) {
            return switch (command) {
                case "instance.list" -> json("{\"instances\":[{\"id\":\"hub\",\"kind\":\"folia\",\"desiredState\":\"running\",\"observedState\":\"ready\",\"healthy\":true,\"presence\":{\"playerCount\":1}}]}");
                case "player.home.list" -> json("{\"homes\":[{\"home\":\"base\",\"serverId\":\"hub\"}]}");
                case "player.warp.list" -> json("{\"warps\":[{\"warp\":\"spawn\",\"serverId\":\"hub\"}]}");
                case "claim.list" -> json("{\"claims\":[{\"name\":\"claim\",\"chunkCount\":2}]}");
                case "player.shop.list" -> json("{\"items\":[{\"id\":\"apple\",\"titleKey\":\"shop.apple\",\"pricePoints\":5,\"deliveryAvailable\":true}]}");
                case "player.kit.list" -> json("{\"kits\":[{\"id\":\"daily\",\"titleKey\":\"kit.daily\",\"rewardPoints\":10,\"cooldownHours\":24}]}");
                case "player.vote.list" -> json("{\"links\":[{\"id\":\"site\",\"titleKey\":\"vote.site\",\"url\":\"https://example.test\"}]}");
                case "player.mail.inbox" -> json("{\"messages\":[{\"id\":\"mail-1\",\"senderName\":\"Sam\",\"body\":\"hi\",\"read\":false}]}");
                case "player.report.list" -> json("{\"reports\":[{\"id\":\"report-1\",\"serverId\":\"hub\",\"reason\":\"grief\",\"status\":\"open\"}]}");
                case "player.daily.status" -> json("{\"claimedToday\":false,\"points\":100}");
                case "player.points.balance" -> json("{\"balance\":5}");
                case "player.achievements.list" -> json("{\"achievements\":[{\"id\":\"first-home\",\"titleKey\":\"achievement.first-home\"}]}");
                case "player.party.info" -> json("{\"found\":true,\"name\":\"Raiders\",\"role\":\"owner\"}");
                default -> json("{}");
            };
        }
    }
}
