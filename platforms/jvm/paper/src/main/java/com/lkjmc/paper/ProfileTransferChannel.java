package com.lkjmc.paper;

import com.lkjmc.common.transfer.ProfileTransferMessages;
import org.bukkit.entity.Player;

final class ProfileTransferChannel {
    interface Sink {
        void send(Player player, byte[] message);
    }

    private final Sink sink;

    ProfileTransferChannel(LkjmcPaperPlugin plugin) {
        this((player, message) -> player.sendPluginMessage(plugin, ProfileTransferMessages.CHANNEL, message));
    }

    ProfileTransferChannel(Sink sink) {
        this.sink = sink;
    }

    void transfer(Player player, String target) {
        sink.send(player, ProfileTransferMessages.transferRequest(target));
    }
}
