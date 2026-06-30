package com.lkjmc.common.menu;

public record AdminMenuPermissions(
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
) {}
