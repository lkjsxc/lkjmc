package com.lkjmc.common.command;

import com.lkjmc.common.permission.PermissionNodes;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import java.util.Locale;
import java.util.Set;

public final class LkjmcCommandTree {
    private static final Set<CommandPlatform> BOTH = Set.of(CommandPlatform.PAPER, CommandPlatform.VELOCITY);
    private static final Set<CommandPlatform> PAPER = Set.of(CommandPlatform.PAPER);
    private static final Set<CommandPlatform> PROXY = Set.of(CommandPlatform.VELOCITY);
    private static final List<CommandSpec> SPECS = List.of(
        spec("status", PermissionNodes.ADMIN_STATUS, "status", BOTH),
        spec("doctor", PermissionNodes.ADMIN_STATUS, "doctor", BOTH),
        spec("server list", PermissionNodes.ADMIN_INSTANCE_LIST, "instance.list", BOTH),
        spec("server start <server>", PermissionNodes.ADMIN_INSTANCE_START, "instance.start", BOTH),
        spec("server stop <server>", PermissionNodes.ADMIN_INSTANCE_STOP, "instance.stop", BOTH),
        spec("server restart <server>", PermissionNodes.ADMIN_INSTANCE_RESTART, "instance.restart", BOTH),
        spec("server create <server> <template>", PermissionNodes.ADMIN_INSTANCE_CREATE, "instance.create", BOTH),
        spec("server delete <server> confirm", PermissionNodes.ADMIN_INSTANCE_DELETE, "instance.delete", BOTH),
        spec("admin role list", PermissionNodes.ADMIN_ADMIN, "admin.role.list", BOTH),
        spec("admin grant <principal> <role> --reason <reason>", PermissionNodes.ADMIN_ADMIN, "admin.grant.create", BOTH),
        spec("admin revoke <principal> <role> --reason <reason>", PermissionNodes.ADMIN_ADMIN, "admin.grant.revoke", BOTH),
        spec("admin inspect <principal>", PermissionNodes.ADMIN_ADMIN, "admin.principal.inspect", BOTH),
        spec("admin audit <lines>", PermissionNodes.ADMIN_ADMIN, "admin.audit.tail", BOTH),
        spec("config check", PermissionNodes.ADMIN_STATUS, "config.check", BOTH),
        spec("config reload", PermissionNodes.ADMIN_RELOAD, "config.reload", BOTH),
        spec("security daemon-token status", PermissionNodes.ADMIN_ADMIN, "security.daemon-token.status", BOTH),
        spec("security daemon-token rotate", PermissionNodes.ADMIN_ADMIN, "security.daemon-token.rotate", BOTH),
        spec("economy seed-defaults", PermissionNodes.ADMIN_ECONOMY, "economy.catalog.seed-defaults", BOTH),
        spec("adventure catalog", PermissionNodes.USER_ADVENTURE, "adventure.catalog.list", BOTH),
        spec("adventure start <adventure>", PermissionNodes.USER_ADVENTURE, "adventure.purchase", PAPER),
        spec("adventure return", PermissionNodes.USER_ADVENTURE, "adventure.return", PAPER),
        spec("adventure sessions", PermissionNodes.ADMIN_INSTANCE_LIST, "adventure.session.list", BOTH),
        spec("adventure cancel <session> <reason>", PermissionNodes.ADMIN_INSTANCE_DELETE, "adventure.session.cancel", BOTH),
        spec("send <player> <server>", PermissionNodes.ADMIN_SEND, "proxy.send", PROXY),
        spec("temporary send <player> <instance>", PermissionNodes.ADMIN_SEND, "temporary.send", PROXY),
        spec("wake send <player> <server>", PermissionNodes.ADMIN_SEND, "wake.send", PROXY),
        spec("reload", PermissionNodes.ADMIN_RELOAD, "config.reload", BOTH),
        spec("restart warn <seconds>", PermissionNodes.ADMIN_RELOAD, "restart.warn", BOTH)
    );

    private LkjmcCommandTree() {}
    public static List<CommandSpec> specs() { return SPECS; }
    public static CommandParseResult parse(CommandPlatform platform, List<String> args) {
        return LkjmcCommandParser.parse(platform, original(args), SPECS);
    }

    public static List<String> suggest(CommandPlatform platform, List<String> args,
                                       CommandPermissionChecker permissions,
                                       CommandCompletionContext context) {
        var clean = args == null ? List.<String>of() : args;
        var position = clean.isEmpty() ? 0 : clean.size() - 1;
        var prefix = clean.isEmpty() ? "" : clean.get(position).toLowerCase();
        var values = new ArrayList<String>();
        for (var spec : SPECS) {
            if (spec.supports(platform) && permissions.has(spec.permission())
                && position < spec.path().size() && previousMatches(spec, clean, position)) {
                values.addAll(candidates(spec.path().get(position), context));
            }
        }
        return values.stream().filter(value -> value.toLowerCase().startsWith(prefix))
            .distinct().sorted(Comparator.naturalOrder()).toList();
    }

    public static String usage(CommandPlatform platform, List<String> args) {
        var clean = lower(original(args));
        if (clean.isEmpty()) return rootUsage();
        return switch (clean.get(0)) {
            case "admin" -> "/lkjmc admin role|grant|revoke|inspect|audit";
            case "config" -> "/lkjmc config check|reload";
            case "security" -> "/lkjmc security daemon-token status|rotate";
            case "economy" -> "/lkjmc economy seed-defaults";
            case "adventure" -> "/lkjmc adventure catalog|start|return|sessions|cancel";
            case "server" -> serverUsage(clean);
            case "restart" -> "/lkjmc restart warn <seconds>";
            case "send", "temporary", "wake" -> platform == CommandPlatform.VELOCITY
                ? "/lkjmc send <player> <server> | temporary send <player> <instance> | wake send <player> <server>"
                : rootUsage();
            default -> rootUsage();
        };
    }

    private static String rootUsage() {
        return "/lkjmc status|doctor|server|admin|config|security|economy|adventure|reload|restart";
    }

    private static String serverUsage(List<String> clean) {
        if (clean.size() > 1 && "delete".equals(clean.get(1))) return "/lkjmc server delete <server> confirm";
        if (clean.size() > 1 && "create".equals(clean.get(1))) return "/lkjmc server create <server> <template>";
        return "/lkjmc server list|start|stop|restart|create|delete";
    }

    private static CommandSpec spec(String path, String permission, String target, Set<CommandPlatform> platforms) {
        return new CommandSpec(List.of(path.split(" ")), permission, CommandSenderKind.ANY,
            "/lkjmc " + path, "command.lkjmc." + target.replace('.', '-'), target, platforms);
    }

    private static boolean previousMatches(CommandSpec spec, List<String> args, int endExclusive) {
        for (var index = 0; index < endExclusive; index++) {
            if (index >= spec.path().size()) return false;
            var expected = spec.path().get(index);
            var actual = args.get(index).toLowerCase();
            if (!CommandSpec.isArgument(expected) && !expected.equals(actual)) return false;
            if (CommandSpec.isArgument(expected) && actual.isBlank()) return false;
        }
        return true;
    }

    private static List<String> candidates(String token, CommandCompletionContext context) {
        if (!CommandSpec.isArgument(token)) return List.of(token);
        return switch (token) {
            case "<server>", "<instance>", "<session>" -> context.serverIds();
            case "<player>" -> context.playerNames();
            case "<template>" -> context.templates();
            case "<role>" -> context.roleIds();
            case "<adventure>" -> context.adventureIds();
            case "<item>" -> context.shopItemIds();
            case "<principal>" -> context.principalHints();
            case "<reason>" -> List.of("maintenance", "security", "cleanup");
            case "<lines>" -> List.of("25", "50", "100");
            case "<seconds>" -> List.of("30", "60", "300");
            default -> List.of();
        };
    }

    private static List<String> original(List<String> args) {
        return args == null ? List.of() : args.stream().filter(value -> value != null).toList();
    }

    private static List<String> lower(List<String> args) {
        return args.stream().map(value -> value.toLowerCase(Locale.ROOT)).toList();
    }
}
