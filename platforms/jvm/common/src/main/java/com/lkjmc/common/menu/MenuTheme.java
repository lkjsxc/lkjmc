package com.lkjmc.common.menu;

public record MenuTheme(String borderMaterial) {
    public static final MenuTheme ROOT = new MenuTheme("LIGHT_BLUE_STAINED_GLASS_PANE");
    public static final MenuTheme NETWORK = new MenuTheme("CYAN_STAINED_GLASS_PANE");
    public static final MenuTheme TRAVEL = new MenuTheme("GREEN_STAINED_GLASS_PANE");
    public static final MenuTheme ECONOMY = new MenuTheme("YELLOW_STAINED_GLASS_PANE");
    public static final MenuTheme CLAIMS = new MenuTheme("LIME_STAINED_GLASS_PANE");
    public static final MenuTheme SOCIAL = new MenuTheme("PURPLE_STAINED_GLASS_PANE");
    public static final MenuTheme SETTINGS = new MenuTheme("LIGHT_GRAY_STAINED_GLASS_PANE");
    public static final MenuTheme DANGER = new MenuTheme("RED_STAINED_GLASS_PANE");
}
