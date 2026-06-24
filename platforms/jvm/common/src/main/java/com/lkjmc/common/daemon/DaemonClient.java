package com.lkjmc.common.daemon;

import java.util.concurrent.CompletableFuture;

public interface DaemonClient {
    CompletableFuture<DaemonResponse> send(DaemonRequest request);
}
