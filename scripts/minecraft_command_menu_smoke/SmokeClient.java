package com.lkjmc.smoke;

import java.nio.charset.StandardCharsets;
import java.time.Duration;
import java.util.*;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.TimeUnit;
import net.kyori.adventure.text.Component;
import net.kyori.adventure.text.serializer.plain.PlainTextComponentSerializer;
import org.geysermc.mcprotocollib.auth.GameProfile;
import org.geysermc.mcprotocollib.network.ClientSession;
import org.geysermc.mcprotocollib.network.Session;
import org.geysermc.mcprotocollib.network.event.session.DisconnectedEvent;
import org.geysermc.mcprotocollib.network.event.session.SessionAdapter;
import org.geysermc.mcprotocollib.network.factory.ClientNetworkSessionFactory;
import org.geysermc.mcprotocollib.network.packet.Packet;
import org.geysermc.mcprotocollib.protocol.MinecraftProtocol;
import org.geysermc.mcprotocollib.protocol.data.game.inventory.ClickItemAction;
import org.geysermc.mcprotocollib.protocol.data.game.inventory.ContainerActionType;
import org.geysermc.mcprotocollib.protocol.data.game.item.HashedStack;
import org.geysermc.mcprotocollib.protocol.data.game.item.ItemStack;
import org.geysermc.mcprotocollib.protocol.data.game.item.component.DataComponentTypes;
import org.geysermc.mcprotocollib.protocol.packet.ingame.clientbound.*;
import org.geysermc.mcprotocollib.protocol.packet.ingame.clientbound.entity.player.ClientboundPlayerPositionPacket;
import org.geysermc.mcprotocollib.protocol.packet.ingame.clientbound.inventory.*;
import org.geysermc.mcprotocollib.protocol.packet.ingame.serverbound.ServerboundChatCommandPacket;
import org.geysermc.mcprotocollib.protocol.packet.ingame.serverbound.ServerboundCommandSuggestionPacket;
import org.geysermc.mcprotocollib.protocol.packet.ingame.serverbound.inventory.ServerboundContainerClickPacket;
import org.geysermc.mcprotocollib.protocol.packet.ingame.serverbound.level.ServerboundAcceptTeleportationPacket;
import org.geysermc.mcprotocollib.protocol.packet.ingame.serverbound.player.ServerboundMovePlayerPosRotPacket;

final class SmokeClient implements AutoCloseable {
    private static final String PLAYER = "LkjmcSmoke";
    private final String host;
    private final int port;
    private final CountDownLatch joined = new CountDownLatch(1);
    private final List<String> messages = new ArrayList<>();
    private final List<String> itemTexts = new ArrayList<>();
    private final Map<Integer, List<String>> suggestions = new HashMap<>();
    private ClientSession session;
    private int transaction = 1;
    private volatile int containerId;
    private volatile int stateId;
    private volatile int closeCount;
    private volatile String title = "";

    SmokeClient(String host, int port) { this.host = host; this.port = port; }

    void connect() throws InterruptedException {
        var protocol = new MinecraftProtocol(new GameProfile(offlineUuid(), PLAYER), "");
        session = ClientNetworkSessionFactory.factory().setAddress(host, port).setProtocol(protocol).create();
        session.addListener(new Listener());
        session.connect(false);
        if (!joined.await(60, TimeUnit.SECONDS)) { throw new IllegalStateException("join timed out"); }
    }

    void command(String command) { session.send(new ServerboundChatCommandPacket(command)); }

    void assertSuggestions(String input, String... expected) throws InterruptedException {
        var deadline = System.nanoTime() + Duration.ofSeconds(10).toNanos();
        var last = List.<String>of();
        while (System.nanoTime() < deadline) {
            var id = transaction++;
            session.send(new ServerboundCommandSuggestionPacket(id, input));
            try {
                last = awaitSuggestions(id, Duration.ofSeconds(2));
            } catch (IllegalStateException error) {
                last = List.of(error.getMessage());
            }
            if (last.containsAll(List.of(expected))) { return; }
            Thread.sleep(200L);
        }
        throw new IllegalStateException(input + " missing " + List.of(expected) + " in " + last);
    }

    void click(int slot) {
        var empty = new HashedStack(0, 0, Map.of(), Set.of());
        session.send(new ServerboundContainerClickPacket(containerId, stateId, slot,
            ContainerActionType.CLICK_ITEM, ClickItemAction.LEFT_CLICK, empty, Map.of()));
    }

    void awaitTitle(String expected, Duration timeout) throws InterruptedException {
        waitUntil(timeout, () -> title.contains(expected), "title not seen: " + expected + " current=" + title);
    }

    void awaitItem(String needle, Duration timeout) throws InterruptedException {
        waitUntil(timeout, () -> {
            synchronized (itemTexts) { return itemTexts.stream().anyMatch(text -> text.contains(needle)); }
        }, "item not seen: " + needle + " in " + itemSnapshot());
    }

    void assertStillOpen(String expected, Duration timeout) throws InterruptedException {
        var closes = closeCount;
        Thread.sleep(timeout.toMillis());
        if (!title.contains(expected) || closeCount != closes) {
            throw new IllegalStateException("menu closed or changed: " + title);
        }
    }

    void awaitMessage(String needle, Duration timeout) throws InterruptedException {
        waitUntil(timeout, () -> {
            synchronized (messages) { return messages.stream().anyMatch(message -> message.contains(needle)); }
        }, "message not seen: " + needle + " in " + messages);
    }

    void assertNoParserLeak() {
        synchronized (messages) {
            for (var message : messages) {
                var lower = message.toLowerCase(Locale.ROOT);
                if (lower.contains("incorrect") || lower.contains("at position")) {
                    throw new IllegalStateException("parser leak: " + message);
                }
            }
        }
    }

    @Override public void close() { if (session != null && session.isConnected()) { session.disconnect("done"); } }

    private List<String> awaitSuggestions(int id, Duration timeout) throws InterruptedException {
        waitUntil(timeout, () -> { synchronized (suggestions) { return suggestions.containsKey(id); } },
            "suggestions timed out for " + id);
        synchronized (suggestions) { return suggestions.get(id); }
    }

    private void waitUntil(Duration timeout, BooleanSupplier condition, String failure) throws InterruptedException {
        var deadline = System.nanoTime() + timeout.toNanos();
        while (System.nanoTime() < deadline) {
            if (condition.getAsBoolean()) { return; }
            Thread.sleep(100L);
        }
        throw new IllegalStateException(failure);
    }

    private final class Listener extends SessionAdapter {
        @Override public void packetReceived(Session ignored, Packet packet) {
            if (packet instanceof ClientboundPlayerPositionPacket position) {
                session.send(new ServerboundAcceptTeleportationPacket(position.getId()));
                session.send(new ServerboundMovePlayerPosRotPacket(true, false, position.getPosition().getX(),
                    position.getPosition().getY(), position.getPosition().getZ(), position.getYRot(), position.getXRot()));
                joined.countDown();
            } else if (packet instanceof ClientboundCommandSuggestionsPacket result) {
                synchronized (suggestions) { suggestions.put(result.getTransactionId(), List.of(result.getMatches())); }
            } else if (packet instanceof ClientboundSystemChatPacket chat) {
                remember(PlainTextComponentSerializer.plainText().serialize(chat.getContent()));
            } else if (packet instanceof ClientboundOpenScreenPacket open) {
                containerId = open.getContainerId();
                title = PlainTextComponentSerializer.plainText().serialize(open.getTitle());
                synchronized (itemTexts) { itemTexts.clear(); }
            } else if (packet instanceof ClientboundContainerSetContentPacket content) {
                if (content.getContainerId() == containerId) { stateId = content.getStateId(); rememberItems(content.getItems()); }
            } else if (packet instanceof ClientboundContainerClosePacket close) {
                if (close.getContainerId() == containerId) { closeCount++; }
            }
        }
        @Override public void disconnected(DisconnectedEvent event) { remember("disconnect: " + event.getReason()); }
        private void remember(String message) { synchronized (messages) { messages.add(message); } }
    }

    private String itemSnapshot() {
        synchronized (itemTexts) { return itemTexts.toString(); }
    }

    private void rememberItems(ItemStack[] items) {
        synchronized (itemTexts) {
            itemTexts.clear();
            for (var item : items) {
                if (item == null || item.getDataComponentsPatch() == null) { continue; }
                addName(item.getDataComponentsPatch().get(DataComponentTypes.CUSTOM_NAME));
                addName(item.getDataComponentsPatch().get(DataComponentTypes.ITEM_NAME));
            }
        }
    }

    private void addName(Component component) {
        if (component != null) { itemTexts.add(PlainTextComponentSerializer.plainText().serialize(component)); }
    }

    private static UUID offlineUuid() {
        return UUID.nameUUIDFromBytes(("OfflinePlayer:" + PLAYER).getBytes(StandardCharsets.UTF_8));
    }

    private interface BooleanSupplier { boolean getAsBoolean(); }
}
