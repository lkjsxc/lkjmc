package com.lkjmc.common.menu;

public record ReportMenuEntry(String id, String serverId, String reason, String status) {
    public ReportMenuEntry {
        if (id == null || id.isBlank()) {
            throw new IllegalArgumentException("report id is required");
        }
        serverId = serverId == null || serverId.isBlank() ? "unknown" : serverId;
        reason = reason == null ? "" : reason;
        status = status == null || status.isBlank() ? "open" : status;
    }

    public String shortId() {
        return id.length() <= 8 ? id : id.substring(0, 8);
    }
}
