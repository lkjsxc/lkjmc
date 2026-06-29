package com.lkjmc.velocity;

import com.lkjmc.common.command.CommandPlatform;
import com.lkjmc.common.command.CommandSpec;
import com.lkjmc.common.command.LkjmcCommandTree;
import com.mojang.brigadier.arguments.IntegerArgumentType;
import com.mojang.brigadier.arguments.StringArgumentType;
import com.mojang.brigadier.builder.ArgumentBuilder;
import com.mojang.brigadier.context.CommandContext;
import com.mojang.brigadier.suggestion.SuggestionProvider;
import com.velocitypowered.api.command.BrigadierCommand;
import com.velocitypowered.api.command.CommandSource;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Locale;
import java.util.Map;

final class VelocityLkjmcBrigadier {
    private VelocityLkjmcBrigadier() {}

    static BrigadierCommand create(VelocityLkjmcCommand executor) {
        var tree = commandTree();
        var root = BrigadierCommand.literalArgumentBuilder("lkjmc")
            .requires(executor::hasAnyPermission)
            .executes(context -> executor.usage(context.getSource(), List.of()));
        for (var child : tree.children.values()) {
            root.then(builder(child, List.of(child.token), executor));
        }
        return new BrigadierCommand(root);
    }

    private static Node commandTree() {
        var root = new Node("");
        for (var spec : LkjmcCommandTree.specs()) {
            if (!spec.supports(CommandPlatform.VELOCITY)) {
                continue;
            }
            var current = root;
            for (var token : spec.path()) {
                current = current.children.computeIfAbsent(token, Node::new);
            }
            current.spec = spec;
        }
        return root;
    }

    private static ArgumentBuilder<CommandSource, ?> builder(
        Node node, List<String> prefix, VelocityLkjmcCommand executor
    ) {
        var builder = CommandSpec.isArgument(node.token)
            ? argumentBuilder(node.token, executor)
            : BrigadierCommand.literalArgumentBuilder(node.token);
        builder.requires(source -> executor.canUsePrefix(source, prefix));
        builder.executes(context -> node.spec == null
            ? executor.usage(context.getSource(), actual(context, prefix))
            : executor.execute(context.getSource(), actual(context, node.spec.path())));
        for (var child : node.children.values()) {
            var childPrefix = new ArrayList<>(prefix);
            childPrefix.add(child.token);
            builder.then(builder(child, List.copyOf(childPrefix), executor));
        }
        return builder;
    }

    private static ArgumentBuilder<CommandSource, ?> argumentBuilder(String token, VelocityLkjmcCommand executor) {
        var name = argumentName(token);
        if ("seconds".equals(name)) {
            return BrigadierCommand.requiredArgumentBuilder(name, IntegerArgumentType.integer(0))
                .suggests(suggestions(executor, token));
        }
        return BrigadierCommand.requiredArgumentBuilder(name, StringArgumentType.word())
            .suggests(suggestions(executor, token));
    }

    private static SuggestionProvider<CommandSource> suggestions(VelocityLkjmcCommand executor, String token) {
        return (context, builder) -> {
            var remaining = builder.getRemaining().toLowerCase(Locale.ROOT);
            for (var value : candidates(executor, token)) {
                if (value.toLowerCase(Locale.ROOT).startsWith(remaining)) {
                    builder.suggest(value);
                }
            }
            return builder.buildFuture();
        };
    }

    private static List<String> candidates(VelocityLkjmcCommand executor, String token) {
        var context = executor.context();
        return switch (token) {
            case "<server>", "<instance>" -> context.serverIds();
            case "<player>" -> context.playerNames();
            case "<template>" -> context.templates();
            case "<seconds>" -> List.of("30", "60", "300");
            default -> List.of();
        };
    }

    private static List<String> actual(CommandContext<CommandSource> context, List<String> pattern) {
        var args = new ArrayList<String>();
        for (var token : pattern) {
            if (!CommandSpec.isArgument(token)) {
                args.add(token);
                continue;
            }
            var name = argumentName(token);
            var type = "seconds".equals(name) ? Integer.class : String.class;
            args.add(String.valueOf(context.getArgument(name, type)));
        }
        return List.copyOf(args);
    }

    private static String argumentName(String token) {
        return token.substring(1, token.length() - 1);
    }

    private static final class Node {
        private final String token;
        private final Map<String, Node> children = new LinkedHashMap<>();
        private CommandSpec spec;

        private Node(String token) {
            this.token = token;
        }
    }
}
