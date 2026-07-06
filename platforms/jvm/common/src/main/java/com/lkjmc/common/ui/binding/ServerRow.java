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
        var presence = Jsons.object(row, "presence", binding);
        return new ServerRow(
            Jsons.string(row, "id", binding),
            Jsons.string(row, "kind", binding),
            Jsons.string(row, "desiredState", binding),
            Jsons.string(row, "observedState", binding),
            Jsons.bool(row, "healthy", binding),
            Jsons.string(row, "connectHost", binding),
            Jsons.nullableInt(row, "connectPort", binding),
            Jsons.bool(row, "proxyRegistrationDesired", binding),
            Jsons.bool(row, "proxyRegistered", binding),
            Jsons.bool(row, "joinable", binding),
            Jsons.string(row, "joinDisabledReason", binding),
            Math.toIntExact(Jsons.integer(presence, "playerCount", binding)));
    }

    String disabledReason() {
        return joinDisabledReason.isBlank() ? "menu.disabled.server-actions" : joinDisabledReason;
    }

    String summary() {
        var address = connectPort == null ? connectHost : connectHost + ":" + connectPort;
        return kind + " " + desiredState + " " + observedState + " " + playerCount + " " + address;
    }
}
