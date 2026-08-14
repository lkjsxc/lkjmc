package com.lkjmc.common.menu;

public final class MenuTypes {
    private MenuTypes() {}

    public enum RouteKind { STATIC, LIST, CUSTOM }
    public enum Theme { ROOT, DOCS }
    public enum Domain { LOCAL_DOCS }
    public enum Scope { LOCAL }
    public enum Role { INFO, NAVIGATION }
    public enum ActionType { NAVIGATE, BACK, CLOSE, NONE }
    public enum Failure { UNKNOWN_ROUTE, MISSING_PARAMETER, STALE_RENDER, UNKNOWN_ACTION }
    public enum Binding { DOCS_DIRECTORY, DOCS_FILE, DOCS_LINKS, DOCS_SEARCH }
}
