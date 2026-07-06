package com.lkjmc.paper.ui;

import com.lkjmc.common.i18n.MiniMessageText;
import com.lkjmc.common.ui.document.ItemRole;
import com.lkjmc.common.ui.kernel.TextRef;
import java.util.List;
import net.kyori.adventure.text.Component;
import net.kyori.adventure.text.format.NamedTextColor;
import net.kyori.adventure.text.format.TextDecoration;

final class UiText {
    private static final String NOT_ITALIC = "<!italic>";
    private static final String NOT_ITALIC_CLOSE = "</!italic>";
    private final MiniMessageText mini;

    UiText(MiniMessageText mini) {
        this.mini = mini;
    }

    Component title(String locale, TextRef ref) {
        return render(locale, ref);
    }

    Component itemName(String locale, TextRef ref, ItemRole role) {
        var component = switch (ref) {
            case TextRef.Key key -> mini.renderItemName(locale, key.key(), key.args());
            case TextRef.Literal literal -> Component.text(literal.value());
        };
        return component.decoration(TextDecoration.ITALIC, false).colorIfAbsent(color(role));
    }

    List<Component> lore(String locale, List<TextRef> lore, ItemRole role) {
        return lore.stream().map(line -> loreLine(locale, line, role)).toList();
    }

    Component chat(String locale, TextRef ref) {
        return render(locale, ref);
    }

    private Component loreLine(String locale, TextRef ref, ItemRole role) {
        var component = switch (ref) {
            case TextRef.Key key -> MiniMessageText.parseStrict(
                NOT_ITALIC + mini.renderMarkup(locale, key.key(), key.args()) + NOT_ITALIC_CLOSE);
            case TextRef.Literal literal -> Component.text(literal.value());
        };
        return component.decoration(TextDecoration.ITALIC, false).colorIfAbsent(color(role));
    }

    private Component render(String locale, TextRef ref) {
        return switch (ref) {
            case TextRef.Key key -> mini.render(locale, key.key(), key.args());
            case TextRef.Literal literal -> Component.text(literal.value());
        };
    }

    private static NamedTextColor color(ItemRole role) {
        return switch (role == null ? ItemRole.ACTION : role) {
            case INFO -> NamedTextColor.GOLD;
            case NAVIGATION -> NamedTextColor.AQUA;
            case ACTION -> NamedTextColor.GREEN;
            case SUCCESS -> NamedTextColor.GREEN;
            case DANGER -> NamedTextColor.RED;
            case DISABLED -> NamedTextColor.DARK_GRAY;
            case WARNING, LOADING -> NamedTextColor.YELLOW;
            case DECORATION -> NamedTextColor.GRAY;
        };
    }
}
