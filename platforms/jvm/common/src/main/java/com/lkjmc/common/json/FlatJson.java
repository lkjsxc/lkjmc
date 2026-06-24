package com.lkjmc.common.json;

import java.util.Map;

import com.lkjmc.common.i18n.MessageCatalog;

public final class FlatJson {
    private FlatJson() {}

    public static Map<String, String> parseStringMap(String json) {
        return MessageCatalog.parseJson(json);
    }
}
