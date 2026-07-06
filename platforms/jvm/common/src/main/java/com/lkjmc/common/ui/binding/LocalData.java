package com.lkjmc.common.ui.binding;

import com.lkjmc.common.docs.DocBundle;
import java.util.List;

public record LocalData(DocBundle docs, List<OnlinePlayer> onlinePlayers) {
    public LocalData {
        onlinePlayers = List.copyOf(onlinePlayers == null ? List.of() : onlinePlayers);
    }

    public static LocalData empty() {
        return new LocalData(null, List.of());
    }

    public record OnlinePlayer(String uuid, String name, String serverId) {
        public OnlinePlayer {
            uuid = uuid == null ? "" : uuid;
            name = name == null ? "" : name;
            serverId = serverId == null ? "" : serverId;
        }
    }
}
