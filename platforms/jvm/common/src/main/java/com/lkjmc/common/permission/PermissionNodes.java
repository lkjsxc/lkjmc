package com.lkjmc.common.permission;

import java.util.Set;

public final class PermissionNodes {
    public static final String USER_MENU = "lkjmc.user.menu";
    public static final String USER_LANGUAGE = "lkjmc.user.language";
    public static final String USER_HOME = "lkjmc.user.home";
    public static final String USER_WARP = "lkjmc.user.warp";
    public static final String USER_TELEPORT_REQUEST = "lkjmc.user.teleport.request";
    public static final String USER_POINTS = "lkjmc.user.points";
    public static final String USER_PARTY = "lkjmc.user.party";
    public static final String USER_ACHIEVEMENTS = "lkjmc.user.achievements";
    public static final String USER_HUD = "lkjmc.user.hud";
    public static final String USER_SHOP = "lkjmc.user.shop";
    public static final String USER_KIT = "lkjmc.user.kit";
    public static final String USER_MAIL = "lkjmc.user.mail";
    public static final String USER_REPORT = "lkjmc.user.report";
    public static final String ADMIN_REPORTS = "lkjmc.admin.reports";
    public static final String ADMIN_WARN = "lkjmc.admin.warn";
    public static final String ADMIN_BAN = "lkjmc.admin.ban";
    public static final String USER_DAILY = "lkjmc.user.daily";
    public static final String ADMIN_ANNOUNCE = "lkjmc.admin.announce";
    public static final String ADMIN_STATUS = "lkjmc.admin.status";
    public static final String ADMIN_RELOAD = "lkjmc.admin.reload";
    public static final String ADMIN_WARP = "lkjmc.admin.warp";
    public static final String ADMIN_SEND = "lkjmc.admin.send";
    public static final String ADMIN_INSTANCE_LIST = "lkjmc.admin.instance.list";
    public static final String ADMIN_INSTANCE_CREATE = "lkjmc.admin.instance.create";
    public static final String ADMIN_INSTANCE_START = "lkjmc.admin.instance.start";
    public static final String ADMIN_INSTANCE_STOP = "lkjmc.admin.instance.stop";
    public static final String ADMIN_INSTANCE_RESTART = "lkjmc.admin.instance.restart";
    public static final String ADMIN_INSTANCE_DELETE = "lkjmc.admin.instance.delete";

    private PermissionNodes() {}

    public static Set<String> all() {
        return Set.of(
            USER_MENU,
            USER_LANGUAGE,
            USER_HOME,
            USER_WARP,
            USER_TELEPORT_REQUEST,
            USER_POINTS,
            USER_PARTY,
            USER_ACHIEVEMENTS,
            USER_HUD,
            USER_SHOP,
            USER_KIT,
            USER_MAIL,
            USER_REPORT,
            ADMIN_REPORTS,
            ADMIN_WARN,
            ADMIN_BAN,
            USER_DAILY,
            ADMIN_ANNOUNCE,
            ADMIN_STATUS,
            ADMIN_RELOAD,
            ADMIN_WARP,
            ADMIN_SEND,
            ADMIN_INSTANCE_LIST,
            ADMIN_INSTANCE_CREATE,
            ADMIN_INSTANCE_START,
            ADMIN_INSTANCE_STOP,
            ADMIN_INSTANCE_RESTART,
            ADMIN_INSTANCE_DELETE
        );
    }
}
