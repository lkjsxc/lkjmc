package com.lkjmc.common.command;

import java.util.LinkedHashMap;
import java.util.List;
import java.util.Locale;
import java.util.Map;

final class LkjmcCommandParser {
    private LkjmcCommandParser() {}

    static CommandParseResult parse(CommandPlatform platform, List<String> original, List<CommandSpec> specs) {
        var args = original == null ? List.<String>of() : original;
        var lower = lower(args);
        for (var spec : specs) {
            if (spec.supports(platform) && exactStructure(spec, lower)) {
                var malformed = malformedArgument(spec, args);
                if (malformed) {
                    return failure(CommandParseFailureKind.MALFORMED_ARGUMENT, platform, args);
                }
                return CommandParseResult.success(new CommandInvocation(spec, arguments(spec, args)));
            }
        }
        for (var spec : specs) {
            if (!spec.supports(platform) && exactStructure(spec, lower)) {
                return failure(CommandParseFailureKind.UNSUPPORTED_PLATFORM, platform, args);
            }
        }
        return failure(classify(platform, args, lower, specs), platform, args);
    }

    private static CommandParseFailureKind classify(
        CommandPlatform platform, List<String> original, List<String> lower, List<CommandSpec> specs
    ) {
        if (lower.isEmpty()) {
            return CommandParseFailureKind.EMPTY_ROOT;
        }
        var compatible = specs.stream().filter(spec -> prefixCompatible(spec, lower)).toList();
        if (compatible.isEmpty()) {
            return CommandParseFailureKind.UNKNOWN_LITERAL;
        }
        if (compatible.stream().noneMatch(spec -> spec.supports(platform))) {
            return CommandParseFailureKind.UNSUPPORTED_PLATFORM;
        }
        if (compatible.stream().anyMatch(spec -> malformedArgument(spec, original))) {
            return CommandParseFailureKind.MALFORMED_ARGUMENT;
        }
        var supported = compatible.stream().filter(spec -> spec.supports(platform)).toList();
        var index = lower.size();
        if (supported.stream().anyMatch(spec -> index < spec.path().size()
            && CommandSpec.isArgument(spec.path().get(index)))) {
            return CommandParseFailureKind.MISSING_ARGUMENT;
        }
        return CommandParseFailureKind.INCOMPLETE_BRANCH;
    }

    private static CommandParseResult failure(CommandParseFailureKind kind, CommandPlatform platform, List<String> args) {
        return CommandParseResult.failure(kind, LkjmcCommandTree.usage(platform, args));
    }

    private static boolean exactStructure(CommandSpec spec, List<String> args) {
        return args.size() == spec.path().size() && prefixCompatible(spec, args);
    }

    private static boolean prefixCompatible(CommandSpec spec, List<String> args) {
        if (args.size() > spec.path().size()) {
            return false;
        }
        for (var index = 0; index < args.size(); index++) {
            var expected = spec.path().get(index);
            var actual = args.get(index);
            if (CommandSpec.isArgument(expected)) {
                if (actual.isBlank()) {
                    return false;
                }
            } else if (!expected.equals(actual)) {
                return false;
            }
        }
        return true;
    }

    private static boolean malformedArgument(CommandSpec spec, List<String> args) {
        for (var index = 0; index < Math.min(args.size(), spec.path().size()); index++) {
            var token = spec.path().get(index);
            if (("<seconds>".equals(token) || "<lines>".equals(token)) && !validSeconds(args.get(index))) {
                return true;
            }
        }
        return false;
    }

    private static boolean validSeconds(String value) {
        try {
            return Integer.parseInt(value) >= 0;
        } catch (NumberFormatException error) {
            return false;
        }
    }

    private static Map<String, String> arguments(CommandSpec spec, List<String> args) {
        var map = new LinkedHashMap<String, String>();
        for (var index = 0; index < spec.path().size(); index++) {
            var token = spec.path().get(index);
            if (CommandSpec.isArgument(token)) {
                map.put(token.substring(1, token.length() - 1), args.get(index));
            }
        }
        return map;
    }

    private static List<String> lower(List<String> args) {
        return args.stream().map(value -> value.toLowerCase(Locale.ROOT)).toList();
    }
}
