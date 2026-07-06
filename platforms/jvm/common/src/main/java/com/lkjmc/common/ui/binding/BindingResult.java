package com.lkjmc.common.ui.binding;

import com.lkjmc.common.ui.kernel.RouteView;

public sealed interface BindingResult permits BindingResult.Data, BindingResult.Empty,
    BindingResult.Denied {
    static BindingResult data(RouteView view) {
        return new Data(view);
    }

    static BindingResult empty() {
        return new Empty();
    }

    static BindingResult denied() {
        return new Denied();
    }

    record Data(RouteView view) implements BindingResult {}
    record Empty() implements BindingResult {}
    record Denied() implements BindingResult {}
}
