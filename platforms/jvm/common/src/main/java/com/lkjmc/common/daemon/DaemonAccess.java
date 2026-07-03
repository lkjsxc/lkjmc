package com.lkjmc.common.daemon;

import com.lkjmc.common.config.RuntimeConfigValidator;
import java.util.Optional;

public record DaemonAccess(Optional<DaemonClient> client, String code) {
    public static DaemonAccess fromEnv() {
        var runtime = RuntimeConfigValidator.fromEnv();
        if (!runtime.valid()) {
            return new DaemonAccess(Optional.empty(), runtime.code());
        }
        var status = DaemonHttpConfigStatus.fromEnv();
        if (!status.configured()) {
            return new DaemonAccess(Optional.empty(), status.code());
        }
        return new DaemonAccess(HttpDaemonClient.fromEnv().map(value -> (DaemonClient) value), status.code());
    }

    public boolean available() {
        return client.isPresent();
    }
}
