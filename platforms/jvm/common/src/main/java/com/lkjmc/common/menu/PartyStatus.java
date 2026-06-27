package com.lkjmc.common.menu;

public record PartyStatus(boolean found, String name, String role, boolean loaded) {
    public PartyStatus {
        name = name == null || name.isBlank() ? "party" : name;
        role = role == null || role.isBlank() ? "member" : role;
    }

    public static PartyStatus loading() {
        return new PartyStatus(false, "party", "member", false);
    }
}
