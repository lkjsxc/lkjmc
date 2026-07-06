package com.lkjmc.common.ui.document;

import java.util.List;

public record MenuDocument(
    String id,
    Kind kind,
    String title,
    String theme,
    int size,
    List<Param> params,
    String parent,
    Data data,
    ChromeSpec chrome,
    ListGrammar list,
    List<StaticSlot> staticSlots,
    String confirmation
) {
    public MenuDocument {
        if (id == null || id.isBlank()) {
            throw new IllegalArgumentException("document id is required");
        }
        if (kind == null) {
            throw new IllegalArgumentException("document kind is required");
        }
        if (title == null || title.isBlank()) {
            throw new IllegalArgumentException("document title is required");
        }
        if (theme == null || theme.isBlank()) {
            throw new IllegalArgumentException("document theme is required");
        }
        params = List.copyOf(params == null ? List.of() : params);
        chrome = chrome == null ? ChromeSpec.empty() : chrome;
        staticSlots = List.copyOf(staticSlots == null ? List.of() : staticSlots);
    }

    public boolean bound() {
        return data != null;
    }

    public enum Kind { STATIC, LIST, DETAIL, CONFIRM, CUSTOM }

    public record Param(String name, boolean required) {
        public Param {
            if (name == null || name.isBlank()) {
                throw new IllegalArgumentException("param name is required");
            }
        }
    }

    public record Data(String binding, Source source, List<String> commands) {
        public Data {
            if (binding == null || binding.isBlank()) {
                throw new IllegalArgumentException("binding is required");
            }
            source = source == null ? Source.LOCAL : source;
            commands = List.copyOf(commands == null ? List.of() : commands);
        }
    }

    public enum Source { DAEMON, LOCAL }
}
