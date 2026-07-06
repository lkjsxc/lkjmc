package com.lkjmc.common.ui.binding;

public record PermissionsView(
    boolean status,
    boolean reload,
    boolean admin,
    boolean economy,
    boolean announce,
    boolean reports,
    boolean warn,
    boolean ban,
    boolean mute,
    boolean claim,
    boolean listServers,
    boolean createServer,
    boolean startServer,
    boolean stopServer,
    boolean restartServer,
    boolean deleteServer
) {
    public static PermissionsView none() {
        return new PermissionsView(false, false, false, false, false, false, false, false,
            false, false, false, false, false, false, false, false);
    }

    public static PermissionsView all() {
        return new PermissionsView(true, true, true, true, true, true, true, true,
            true, true, true, true, true, true, true, true);
    }
}
