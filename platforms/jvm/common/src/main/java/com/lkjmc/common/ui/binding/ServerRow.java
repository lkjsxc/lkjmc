package com.lkjmc.common.ui.binding;

import com.google.gson.JsonObject;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;

record ServerRow(
    String id,
    String kind,
    String desiredState,
    String observedState,
    boolean healthy,
    String connectHost,
    Integer connectPort,
    boolean proxyRegistrationDesired,
    boolean proxyRegistered,
    boolean joinable,
    String joinDisabledReason,
    int playerCount
) {
    static List<ServerRow> list(JsonObject body, String binding) {
        var rows = new ArrayList<ServerRow>();
        for (var value : Jsons.array(body, "instances", binding)) {
            rows.add(parse(Jsons.elementObject(value, binding), binding));
        }
        return rows.stream().sorted(Comparator.comparing(ServerRow::id)).toList();
    }

    static ServerRow parse(JsonObject row, String binding) {
        var presence = nullableObject(row, "presence");
        return new ServerRow(
            Jsons.string(row, "id", binding),
            Jsons.string(row, "kind", binding),
            Jsons.string(row, "desiredState", binding),
            Jsons.string(row, "observedState", binding),
            nullableBool(row, "healthy", false, binding),
            Jsons.string(row, "connectHost", binding),
            Jsons.nullableInt(row, "connectPort", binding),
            Jsons.bool(row, "proxyRegistrationDesired", binding),
            Jsons.bool(row, "proxyRegistered", binding),
            Jsons.bool(row, "joinable", binding),
            Jsons.string(row, "joinDisabledReason", binding),
            presence == null ? 0 : Math.toIntExact(Jsons.integer(presence, "playerCount", binding)));
    }

    private static JsonObject nullableObject(JsonObject object, String field) {
        var element = object.get(field);
        return element == null || element.isJsonNull() ? null : element.getAsJsonObject();
    }

    private static boolean nullableBool(JsonObject object, String field, boolean fallback, String binding) {
        var element = object.get(field);
        if (element == null || element.isJsonNull()) {
            return fallback;
        }
        try {
            return element.getAsBoolean();
        } catch (RuntimeException error) {
            throw Jsons.fail(binding);
        }
    }

    String disabledReason() {
        return joinDisabledReason.isBlank() ? "menu.disabled.server-actions" : joinDisabledReason;
    }

    String summary() {
        var address = connectPort == null ? connectHost : connectHost + ":" + connectPort;
        return kind + " " + desiredState + " " + observedState + " " + playerCount + " " + address;
    }
}
