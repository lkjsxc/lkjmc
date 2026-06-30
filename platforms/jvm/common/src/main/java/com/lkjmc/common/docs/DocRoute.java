package com.lkjmc.common.docs;

public final class DocRoute {
    private DocRoute() {}

    public static String parent(String route) {
        if (route == null || route.isBlank() || route.equals("dir:")) {
            return "dir:";
        }
        if (route.startsWith("dir:")) {
            return dirParent(route.substring(4));
        }
        if (route.startsWith("file:")) {
            return fileParent(route.substring(5));
        }
        if (route.startsWith("links:")) {
            return fileParent(route.substring(6));
        }
        return "dir:";
    }

    public static boolean hasParent(String route) {
        return route != null && !route.equals("dir:") && !route.isBlank();
    }

    public static String page(String route, int delta, int pageCount) {
        var idx = route == null ? -1 : route.lastIndexOf(':');
        if (idx < 0 || idx + 1 >= route.length()) {
            return route == null ? "dir:" : route;
        }
        var current = parsePage(route.substring(idx + 1));
        var max = Math.max(0, pageCount - 1);
        var next = Math.max(0, Math.min(max, current + delta));
        return route.substring(0, idx + 1) + next;
    }

    public static int parsePage(String value) {
        try {
            return Math.max(0, Integer.parseInt(value));
        } catch (NumberFormatException error) {
            return 0;
        }
    }

    private static String dirParent(String path) {
        if (path == null || path.isBlank()) {
            return "dir:";
        }
        var clean = strip(path);
        if (!clean.contains("/")) {
            return "dir:";
        }
        return "dir:" + clean.substring(0, clean.lastIndexOf('/'));
    }

    private static String fileParent(String payload) {
        if (payload == null || payload.isBlank()) {
            return "dir:";
        }
        var path = strip(payload.split(":", 2)[0]);
        if (!path.contains("/")) {
            return "dir:";
        }
        return "dir:" + path.substring(0, path.lastIndexOf('/'));
    }

    private static String strip(String path) {
        var value = path == null ? "" : path;
        while (value.startsWith("/")) {
            value = value.substring(1);
        }
        while (value.endsWith("/")) {
            value = value.substring(0, value.length() - 1);
        }
        return value;
    }
}
