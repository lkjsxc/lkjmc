package com.lkjmc.common.ui.kernel;

public sealed interface UiMsg permits UiMsg.Open, UiMsg.Clicked, UiMsg.DataLoaded,
    UiMsg.DataEmpty, UiMsg.DataDenied, UiMsg.DataFailed, UiMsg.StaleAvailable,
    UiMsg.BackRequested, UiMsg.RefreshRequested, UiMsg.TextSubmitted,
    UiMsg.InventoryClosed {
    record Open(MenuRoute route) implements UiMsg {}
    record Clicked(int slot, MenuMetadata metadata, boolean malformed) implements UiMsg {}
    record DataLoaded(RouteView view) implements UiMsg {}
    record DataEmpty() implements UiMsg {}
    record DataDenied() implements UiMsg {}
    record DataFailed(String diagnosticCode) implements UiMsg {}
    record StaleAvailable(RouteView view, String code) implements UiMsg {}
    record BackRequested() implements UiMsg {}
    record RefreshRequested() implements UiMsg {}
    record TextSubmitted(String text, String commandPrefix) implements UiMsg {}
    record InventoryClosed() implements UiMsg {}
}
