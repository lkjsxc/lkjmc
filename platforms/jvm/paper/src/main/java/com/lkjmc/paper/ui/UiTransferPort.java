package com.lkjmc.paper.ui;

import com.lkjmc.paper.LkjmcPaperPlugin;
import java.lang.reflect.Constructor;
import java.lang.reflect.Method;
import org.bukkit.entity.Player;

@FunctionalInterface
interface UiTransferPort {
    void transfer(Player player, String target);

    static UiTransferPort profile(LkjmcPaperPlugin plugin) {
        try {
            var type = Class.forName("com.lkjmc.paper.ProfileTransferChannel");
            Constructor<?> constructor = type.getDeclaredConstructor(LkjmcPaperPlugin.class);
            constructor.setAccessible(true);
            var channel = constructor.newInstance(plugin);
            Method transfer = type.getDeclaredMethod("transfer", Player.class, String.class);
            transfer.setAccessible(true);
            return (player, target) -> invoke(transfer, channel, player, target);
        } catch (ReflectiveOperationException error) {
            throw new IllegalStateException("profile transfer channel unavailable", error);
        }
    }

    private static void invoke(Method method, Object target, Player player, String server) {
        try {
            method.invoke(target, player, server);
        } catch (ReflectiveOperationException error) {
            throw new IllegalStateException("profile transfer failed", error);
        }
    }
}
