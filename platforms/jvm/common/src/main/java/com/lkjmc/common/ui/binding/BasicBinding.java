package com.lkjmc.common.ui.binding;

import com.lkjmc.common.ui.kernel.DaemonRequestPlan;
import java.util.List;
import java.util.Map;

abstract class BasicBinding implements MenuBinding {
    private final String id;
    private final String source;
    private final List<String> commands;

    BasicBinding(String id, String source, List<String> commands) {
        this.id = id;
        this.source = source;
        this.commands = List.copyOf(commands == null ? List.of() : commands);
    }

    @Override
    public final String id() {
        return id;
    }

    @Override
    public DaemonRequestPlan plan(BindingContext ctx) {
        var command = commands.isEmpty() ? "" : commands.getFirst();
        var body = "daemon".equals(source) ? PlanBodies.forBinding(id, ctx) : Map.<String, String>of();
        return new DaemonRequestPlan(id, source, command, ctx.params(), body, commands);
    }

    String code() {
        return "menu.decode." + id;
    }
}
