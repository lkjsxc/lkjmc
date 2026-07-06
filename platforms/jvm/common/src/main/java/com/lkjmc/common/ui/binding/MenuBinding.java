package com.lkjmc.common.ui.binding;

import com.google.gson.JsonObject;
import com.lkjmc.common.ui.kernel.DaemonRequestPlan;

public interface MenuBinding {
    String id();
    DaemonRequestPlan plan(BindingContext ctx);
    BindingResult decode(JsonObject body, BindingContext ctx);
}
