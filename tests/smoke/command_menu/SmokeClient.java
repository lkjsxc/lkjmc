package com.lkjmc.smoke;

import java.nio.charset.StandardCharsets;
import java.time.Duration;
import java.util.*;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.TimeUnit;
import net.kyori.adventure.text.Component;
import net.kyori.adventure.text.serializer.plain.PlainTextComponentSerializer;
import org.geysermc.mcprotocollib.auth.GameProfile;
import org.geysermc.mcprotocollib.network.*;
import org.geysermc.mcprotocollib.network.event.session.*;
import org.geysermc.mcprotocollib.network.factory.ClientNetworkSessionFactory;
import org.geysermc.mcprotocollib.network.packet.Packet;
import org.geysermc.mcprotocollib.protocol.MinecraftProtocol;
import org.geysermc.mcprotocollib.protocol.data.game.inventory.*;
import org.geysermc.mcprotocollib.protocol.data.game.item.*;
import org.geysermc.mcprotocollib.protocol.data.game.item.component.DataComponentTypes;
import org.geysermc.mcprotocollib.protocol.packet.ingame.clientbound.*;
import org.geysermc.mcprotocollib.protocol.packet.ingame.clientbound.entity.player.ClientboundPlayerPositionPacket;
import org.geysermc.mcprotocollib.protocol.packet.ingame.clientbound.inventory.*;
import org.geysermc.mcprotocollib.protocol.packet.ingame.serverbound.*;
import org.geysermc.mcprotocollib.protocol.packet.ingame.serverbound.inventory.*;
import org.geysermc.mcprotocollib.protocol.packet.ingame.serverbound.level.*;
import org.geysermc.mcprotocollib.protocol.packet.ingame.serverbound.player.*;

final class SmokeClient implements AutoCloseable {
    private static final String PLAYER = "LkjmcSmoke";
    private final String host;
    private final int port;
    private final CountDownLatch joined = new CountDownLatch(1);
    private final List<String> messages = new ArrayList<>();
    private final Map<Integer, List<String>> slotTexts = new HashMap<>();
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
        session.addListener(new Listener()); session.connect(false);
        if (!joined.await(60, TimeUnit.SECONDS)) { throw new IllegalStateException("join timed out"); }
    }

    void command(String command) { session.send(new ServerboundChatCommandPacket(command)); }

    void assertSuggestions(String input, String... expected) throws InterruptedException {
        var deadline = System.nanoTime() + Duration.ofSeconds(10).toNanos();
        var last = List.<String>of();
        while (System.nanoTime() < deadline) {
            var id = transaction++; session.send(new ServerboundCommandSuggestionPacket(id, input));
            try { last = awaitSuggestions(id, Duration.ofSeconds(2)); } catch (IllegalStateException error) { last = List.of(error.getMessage()); }
            if (last.containsAll(List.of(expected))) { return; }
            Thread.sleep(200L);
        }
        throw new IllegalStateException(input + " missing " + List.of(expected) + " in " + last);
    }

    void click(int slot) {
        var empty = new HashedStack(0, 0, Map.of(), Set.of());
        session.send(new ServerboundContainerClickPacket(containerId, stateId, slot, ContainerActionType.CLICK_ITEM, ClickItemAction.LEFT_CLICK, empty, Map.of()));
    }

    void clickItem(String needle, Duration timeout) throws InterruptedException {
        waitUntil(timeout, () -> topSlot(needle).isPresent(), "item slot not seen: " + needle + " in " + itemSnapshot());
        click(topSlot(needle).orElseThrow());
    }

    void awaitTitle(String expected, Duration timeout) throws InterruptedException {
        waitUntil(timeout, () -> title.contains(expected), "title not seen: " + expected + " current=" + title);
    }

    void awaitTitleExact(String expected, Duration timeout) throws InterruptedException {
        waitUntil(timeout, () -> title.equals(expected), "title not exact: " + expected + " current=" + title);
    }

    void awaitItem(String needle, Duration timeout) throws InterruptedException {
        waitUntil(timeout, () -> anySlotHas(needle), "item not seen: " + needle + " in " + itemSnapshot());
    }

    void awaitSlot(int slot, String needle, Duration timeout) throws InterruptedException {
        waitUntil(timeout, () -> slotHas(slot, needle), "slot " + slot + " missing " + needle + " in " + itemSnapshot());
    }

    String awaitSlotAny(int slot, Duration timeout, String... needles) throws InterruptedException {
        var deadline = System.nanoTime() + timeout.toNanos();
        while (System.nanoTime() < deadline) {
            for (var needle : needles) { if (slotHas(slot, needle)) { return needle; } }
            Thread.sleep(100L);
        }
        throw new IllegalStateException("slot " + slot + " missing " + List.of(needles) + " in " + itemSnapshot());
    }

    void assertStillOpen(String expected, Duration timeout) throws InterruptedException {
        var closes = closeCount; Thread.sleep(timeout.toMillis());
        if (!title.contains(expected) || closeCount != closes) { throw new IllegalStateException("menu closed or changed: " + title); }
    }

    void awaitMessage(String needle, Duration timeout) throws InterruptedException {
        waitUntil(timeout, () -> { synchronized (messages) { return messages.stream().anyMatch(m -> m.contains(needle)); } }, "message not seen: " + needle + " in " + messages);
    }

    int closeCount() { return closeCount; }
    void assertNoCloseSince(int closes) { if (closeCount != closes) { throw new IllegalStateException("unexpected inventory close"); } }
    void awaitCloseAfter(int closes, Duration timeout) throws InterruptedException { waitUntil(timeout, () -> closeCount > closes, "inventory close not seen"); }

    void assertNoParserLeak() {
        synchronized (messages) { for (var message : messages) {
            var lower = message.toLowerCase(Locale.ROOT);
            if (lower.contains("incorrect") || lower.contains("at position")) { throw new IllegalStateException("parser leak: " + message); }
        } }
    }

    @Override public void close() { if (session != null && session.isConnected()) { session.disconnect("done"); } }

    private List<String> awaitSuggestions(int id, Duration timeout) throws InterruptedException {
        waitUntil(timeout, () -> { synchronized (suggestions) { return suggestions.containsKey(id); } }, "suggestions timed out for " + id);
        synchronized (suggestions) { return suggestions.get(id); }
    }

    private void waitUntil(Duration timeout, BooleanSupplier condition, String failure) throws InterruptedException {
        var deadline = System.nanoTime() + timeout.toNanos();
        while (System.nanoTime() < deadline) { if (condition.getAsBoolean()) { return; } Thread.sleep(100L); }
        throw new IllegalStateException(failure);
    }

    private final class Listener extends SessionAdapter {
        @Override public void packetReceived(Session ignored, Packet packet) {
            if (packet instanceof ClientboundPlayerPositionPacket position) {
                session.send(new ServerboundAcceptTeleportationPacket(position.getId()));
                session.send(new ServerboundMovePlayerPosRotPacket(true, false, position.getPosition().getX(), position.getPosition().getY(), position.getPosition().getZ(), position.getYRot(), position.getXRot()));
                joined.countDown();
            } else if (packet instanceof ClientboundCommandSuggestionsPacket result) {
                synchronized (suggestions) { suggestions.put(result.getTransactionId(), List.of(result.getMatches())); }
            } else if (packet instanceof ClientboundSystemChatPacket chat) {
                remember(PlainTextComponentSerializer.plainText().serialize(chat.getContent()));
            } else if (packet instanceof ClientboundOpenScreenPacket open) {
                containerId = open.getContainerId(); title = PlainTextComponentSerializer.plainText().serialize(open.getTitle());
                synchronized (slotTexts) { slotTexts.clear(); }
            } else if (packet instanceof ClientboundContainerSetContentPacket content && content.getContainerId() == containerId) {
                stateId = content.getStateId(); rememberItems(content.getItems());
            } else if (packet instanceof ClientboundContainerSetSlotPacket slot && slot.getContainerId() == containerId) {
                stateId = slot.getStateId(); rememberItem(slot.getSlot(), slot.getItem());
            } else if (packet instanceof ClientboundContainerClosePacket close && close.getContainerId() == containerId) {
                closeCount++;
            }
        }
        @Override public void disconnected(DisconnectedEvent event) { remember("disconnect: " + event.getReason()); }
        private void remember(String message) { synchronized (messages) { messages.add(message); } }
    }

    private String itemSnapshot() { synchronized (slotTexts) { return slotTexts.toString(); } }

    private void rememberItems(ItemStack[] items) {
        synchronized (slotTexts) { slotTexts.clear(); for (int slot = 0; slot < items.length; slot++) { rememberItemLocked(slot, items[slot]); } }
    }

    private void rememberItem(int slot, ItemStack item) { synchronized (slotTexts) { slotTexts.remove(slot); rememberItemLocked(slot, item); } }

    private void rememberItemLocked(int slot, ItemStack item) {
        if (item == null || item.getDataComponentsPatch() == null) { return; }
        addText(slot, item.getDataComponentsPatch().get(DataComponentTypes.CUSTOM_NAME));
        addText(slot, item.getDataComponentsPatch().get(DataComponentTypes.ITEM_NAME));
        var lore = item.getDataComponentsPatch().get(DataComponentTypes.LORE);
        if (lore != null) { lore.forEach(component -> addText(slot, component)); }
    }

    private Optional<Integer> topSlot(String needle) {
        synchronized (slotTexts) {
            return slotTexts.entrySet().stream().filter(e -> e.getKey() < 54 && e.getValue().stream().anyMatch(t -> t.contains(needle))).map(Map.Entry::getKey).findFirst();
        }
    }

    private boolean anySlotHas(String needle) { synchronized (slotTexts) { return slotTexts.values().stream().flatMap(List::stream).anyMatch(t -> t.contains(needle)); } }
    private boolean slotHas(int slot, String needle) { synchronized (slotTexts) { return slotTexts.getOrDefault(slot, List.of()).stream().anyMatch(t -> t.contains(needle)); } }
    private void addText(int slot, Component component) { if (component != null) { slotTexts.computeIfAbsent(slot, ignored -> new ArrayList<>()).add(PlainTextComponentSerializer.plainText().serialize(component)); } }
    private static UUID offlineUuid() { return UUID.nameUUIDFromBytes(("OfflinePlayer:" + PLAYER).getBytes(StandardCharsets.UTF_8)); }
    private interface BooleanSupplier { boolean getAsBoolean(); }
}
