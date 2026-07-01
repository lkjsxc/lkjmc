package com.lkjmc.paper;

import static org.junit.jupiter.api.Assertions.assertEquals;

import com.lkjmc.common.i18n.LocaleResolver;
import java.lang.reflect.Proxy;
import java.util.Locale;
import java.util.Optional;
import java.util.UUID;
import org.bukkit.entity.Player;
import org.junit.jupiter.api.Test;

final class PlayerLocaleServiceTest {
    @Test
    void cachedPersistedLanguageBeatsPlatformLocale() {
        var service = new PlayerLocaleService(null, new LocaleResolver("en"), Optional.empty());
        var player = player(Locale.US);
        service.update(player, "ja");
        assertEquals("ja", service.locale(player));
    }

    @Test
    void fallsBackToPlatformLocale() {
        var service = new PlayerLocaleService(null, new LocaleResolver("en"), Optional.empty());
        assertEquals("ja", service.locale(player(Locale.JAPAN)));
    }

    private static Player player(Locale locale) {
        return proxy(Player.class, (proxy, method, args) -> switch (method.getName()) {
            case "getUniqueId" -> UUID.fromString("00000000-0000-0000-0000-000000000123");
            case "locale" -> locale;
            default -> fallback(method.getReturnType());
        });
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
}
