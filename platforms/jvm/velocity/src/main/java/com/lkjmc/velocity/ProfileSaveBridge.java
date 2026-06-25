package com.lkjmc.velocity;

import com.velocitypowered.api.proxy.Player;
import java.util.concurrent.CompletableFuture;

public interface ProfileSaveBridge {
    CompletableFuture<Boolean> save(Player player);
}
