package com.lkjmc.common.ui.kernel;

public sealed interface UiMsg permits UiMsg.Open, UiMsg.Clicked, UiMsg.DataLoaded,
    UiMsg.DataEmpty, UiMsg.DataDenied, UiMsg.DataFailed, UiMsg.StaleAvailable,
    UiMsg.BackRequested, UiMsg.RefreshRequested, UiMsg.TextSubmitted,
    UiMsg.InventoryClosed {
    record Open(MenuRoute route) implements UiMsg {}
    record Clicked(int slot, MenuMetadata metadata, boolean malformed) implements UiMsg {}
    record DataLoaded(RouteView view, UiRequest request) implements UiMsg {
        public DataLoaded(RouteView view) { this(view, UiRequest.none()); }
    }
    record DataEmpty(UiRequest request) implements UiMsg {
        public DataEmpty() { this(UiRequest.none()); }
    }
    record DataDenied(UiRequest request) implements UiMsg {
        public DataDenied() { this(UiRequest.none()); }
    }
    record DataFailed(String diagnosticCode, UiRequest request) implements UiMsg {
        public DataFailed(String code) { this(code, UiRequest.none()); }
    }
    record StaleAvailable(RouteView view, String code, UiRequest request) implements UiMsg {
        public StaleAvailable(RouteView view, String code) { this(view, code, UiRequest.none()); }
    }
    record BackRequested() implements UiMsg {}
    record RefreshRequested() implements UiMsg {}
    record TextSubmitted(String text, String commandPrefix) implements UiMsg {}
    record InventoryClosed() implements UiMsg {}
}
