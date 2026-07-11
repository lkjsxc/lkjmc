package com.lkjmc.paper.ui;

import com.lkjmc.common.i18n.LocaleResolver;
import com.lkjmc.common.i18n.MessageCatalog;
import com.lkjmc.common.i18n.MiniMessageText;
import com.lkjmc.common.ui.document.ChromeSpec;
import com.lkjmc.common.ui.document.MenuDocument;
import com.lkjmc.common.ui.document.MenuDocumentSet;
import com.lkjmc.common.ui.document.StaticSlot;
import java.lang.reflect.Proxy;
import java.util.ArrayDeque;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.UUID;
import org.bukkit.Location;
import org.bukkit.World;
import org.bukkit.entity.Player;

final class UiTestFixtures {
    static final UUID PLAYER_ID = UUID.fromString("00000000-0000-0000-0000-000000000123");

    private UiTestFixtures() {}

    static UiText text() {
        return new UiText(new MiniMessageText(catalog(), new LocaleResolver("en")));
    }

    static MessageCatalog catalog() {
        return MessageCatalog.of(Map.of("en", Map.ofEntries(
            Map.entry("title.root", "<gold>Root</gold>"),
            Map.entry("title.local", "<green>Local</green>"),
            Map.entry("item.name", "<green>Open</green>"),
            Map.entry("item.lore", "<gray>Lore</gray>"),
            Map.entry("ok", "<green>OK</green>"),
            Map.entry("fail", "<red>Fail</red>"),
            Map.entry("menu.transfer.sending", "<green>Transferring</green>"),
            Map.entry("menu.input.cancel.lore", "<gray>cancel</gray>"),
            Map.entry("menu.input.expired", "<red>expired</red>"),
            Map.entry("menu.input.cancelled", "<yellow>cancelled</yellow>"),
            Map.entry("menu.input.invalid", "<red>invalid</red>"),
            Map.entry("diagnostic.daemon.command_failed.title", "<red>Typed failed</red>"),
            Map.entry("diagnostic.daemon.command_failed.hint", "<gray>Typed hint</gray>")
        )), "en");
    }

    static MenuDocumentSet docs(MenuDocument... extra) {
        var values = new ArrayList<MenuDocument>();
        values.add(document("root", "title.root", null, null));
        values.addAll(List.of(extra));
        return new MenuDocumentSet(values);
    }

    static MenuDocument document(String id, String title, MenuDocument.Data data,
                                 List<StaticSlot> slots) {
        return new MenuDocument(id, MenuDocument.Kind.STATIC, title, "root", 27, List.of(),
            id.equals("root") ? null : "root", data, ChromeSpec.empty(), null,
            slots == null ? List.of() : slots, null);
    }

    static Player player() {
        return proxy(Player.class, new PlayerHandler());
    }

    static PlayerState state(Player player) {
        return ((PlayerHandler) Proxy.getInvocationHandler(player)).state;
    }

    static Object fallback(Class<?> type) {
        if (type.equals(boolean.class)) return false;
        if (type.equals(int.class)) return 0;
        if (type.equals(long.class)) return 0L;
        if (type.equals(double.class)) return 0.0d;
        if (type.equals(float.class)) return 0.0f;
        return null;
    }

    @SuppressWarnings("unchecked")
    static <T> T proxy(Class<T> type, java.lang.reflect.InvocationHandler handler) {
        return (T) Proxy.newProxyInstance(type.getClassLoader(), new Class<?>[] {type}, handler);
    }

    static final class PlayerState {
        final List<Object> messages = new ArrayList<>();
        final List<String> commands = new ArrayList<>();
        boolean closed;
    }

    static final class PlayerHandler implements java.lang.reflect.InvocationHandler {
        final PlayerState state = new PlayerState();
        @Override public Object invoke(Object proxy, java.lang.reflect.Method method, Object[] args) {
            return switch (method.getName()) {
                case "getUniqueId" -> PLAYER_ID;
                case "getName" -> "Alex";
                case "hasPermission", "isOp" -> false;
                case "sendMessage" -> { state.messages.add(args[0]); yield null; }
                case "performCommand" -> { state.commands.add((String) args[0]); yield true; }
                case "closeInventory" -> { state.closed = true; yield null; }
                case "getLocation" -> new Location(null, 1.0, 2.0, 3.0, 4.0f, 5.0f);
                case "toString" -> "Player(Alex)";
                default -> fallback(method.getReturnType());
            };
        }
    }

    static final class Scheduler implements com.lkjmc.paper.SchedulerBridge {
        final ArrayDeque<Runnable> playerTasks = new ArrayDeque<>();
        @Override public void runPlayer(Player player, Runnable task) { playerTasks.add(task); }
        @Override public void runAsync(Runnable task) { task.run(); }
        @Override public void runGlobal(Runnable task) { task.run(); }
        @Override public void runAsyncRepeating(Runnable task, java.time.Duration initialDelay,
                                                java.time.Duration period) {}
        @Override public void cancelAll() {}
        @Override public void runRegion(World world, int chunkX, int chunkZ, Runnable task) { task.run(); }
        void drain() { while (!playerTasks.isEmpty()) { playerTasks.removeFirst().run(); } }
    }
}
