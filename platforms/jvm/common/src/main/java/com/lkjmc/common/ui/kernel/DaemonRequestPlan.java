package com.lkjmc.common.ui.kernel;

import java.util.List;
import java.util.Map;

public record DaemonRequestPlan(
    String binding,
    String source,
    String command,
    Map<String, String> params,
    Map<String, String> body,
    List<String> commands
) {
    public DaemonRequestPlan {
        binding = binding == null ? "" : binding;
        source = source == null ? "" : source;
        command = command == null ? "" : command;
        params = Map.copyOf(params == null ? Map.of() : params);
        body = Map.copyOf(body == null ? Map.of() : body);
        commands = List.copyOf(commands == null ? List.of() : commands);
    }

    public static DaemonRequestPlan load(String binding, String source, MenuRoute route,
                                         List<String> commands) {
        return new DaemonRequestPlan(binding, source, "", route.params(), Map.of(), commands);
    }

    public static DaemonRequestPlan command(String command, Map<String, String> body) {
        return new DaemonRequestPlan("", "daemon", command, Map.of(), body, List.of(command));
    }
}
