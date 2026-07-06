package com.lkjmc.common.ui.kernel;

public sealed interface RoutePhase permits RoutePhase.Loading, RoutePhase.Loaded,
    RoutePhase.Empty, RoutePhase.Denied, RoutePhase.Stale, RoutePhase.Diagnostic,
    RoutePhase.Static {
    record Loading() implements RoutePhase {}
    record Loaded(RouteView view) implements RoutePhase {}
    record Empty() implements RoutePhase {}
    record Denied() implements RoutePhase {}
    record Stale(RouteView view, String diagnosticCode) implements RoutePhase {}
    record Diagnostic(String code) implements RoutePhase {}
    record Static() implements RoutePhase {}
}
