package com.lkjmc.paper.ui;

import com.lkjmc.common.docs.DocBundle;
import com.lkjmc.common.permission.PermissionNodes;
import com.lkjmc.common.permission.PermissionResolver;
import com.lkjmc.common.permission.PermissionSnapshotCache;
import com.lkjmc.common.permission.PrincipalIdentity;
import com.lkjmc.common.ui.binding.BindingContext;
import com.lkjmc.common.ui.binding.LocalData;
import com.lkjmc.common.ui.binding.PermissionsView;
import com.lkjmc.common.ui.document.MenuDocumentSet;
import com.lkjmc.common.ui.kernel.MenuRoute;
import com.lkjmc.common.ui.kernel.UiEffect;
import com.lkjmc.common.ui.kernel.UiIds;
import com.lkjmc.common.ui.kernel.UiModel;
import com.lkjmc.common.ui.kernel.UiMsg;
import com.lkjmc.common.ui.kernel.UiRequest;
import com.lkjmc.common.ui.kernel.UiStep;
import com.lkjmc.common.ui.kernel.UiUpdate;
import java.time.Instant;
import java.util.Collection;
import java.util.HashMap;
import java.util.Map;
import java.util.Optional;
import java.util.UUID;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.ConcurrentMap;
import java.util.concurrent.atomic.AtomicReference;
import java.util.function.Function;
import java.util.function.Supplier;
import org.bukkit.entity.Player;

public final class UiSessionService {
    interface Renderer { void render(Player player, String locale, UiModel model); }
    interface Effects { void run(Player player, UiEffect effect, UiModel model, UiSessionService sessions); }

    private final MenuDocumentSet documents;
    private final Renderer renderer;
    private final Effects effects;
    private final Function<Player, String> locales;
    private final PermissionSnapshotCache grants;
    private final PermissionResolver resolver = new PermissionResolver();
    private final Supplier<DocBundle> docs;
    private final Supplier<Collection<? extends Player>> onlinePlayers;
    private final ConcurrentMap<UUID, UiModel> sessions = new ConcurrentHashMap<>();
    private final UiIds ids = () -> UUID.randomUUID().toString();

    public UiSessionService(MenuDocumentSet documents, Renderer renderer, Effects effects,
                            Function<Player, String> locales, PermissionSnapshotCache grants,
                            Supplier<DocBundle> docs,
                            Supplier<Collection<? extends Player>> onlinePlayers) {
        this.documents = documents;
        this.renderer = renderer;
        this.effects = effects;
        this.locales = locales;
        this.grants = grants == null ? PermissionSnapshotCache.disabled() : grants;
        this.docs = docs == null ? () -> null : docs;
        this.onlinePlayers = onlinePlayers == null ? java.util.List::of : onlinePlayers;
    }

    public void dispatch(Player player, UiMsg msg) {
        var request = responseRequest(msg);
        if (request != null && !accepts(player, request)) {
            return;
        }
        var step = new AtomicReference<UiStep>();
        sessions.compute(player.getUniqueId(), (id, current) -> {
            var model = current == null ? UiModel.root(player.getUniqueId().toString(), ids()) : current;
            var next = UiUpdate.update(documents, model, msg, ids);
            step.set(next);
            return next.model();
        });
        for (var effect : step.get().effects()) effects.run(player, effect, step.get().model(), this);
        render(player);
    }

    boolean accepts(Player player, UiRequest request) {
        var model = sessions.get(player.getUniqueId());
        return request == null || request.empty() || (model != null
            && player.getUniqueId().toString().equals(request.playerId()) && request.matches(model));
    }

    boolean completeMutation(Player player, UiRequest request) {
        if (request == null || request.empty()) return true;
        if (!accepts(player, request)) return false;
        sessions.computeIfPresent(player.getUniqueId(), (id, model) -> UiUpdate.complete(model, request).model());
        render(player);
        return true;
    }

    public void openRoot(Player player) {
        dispatch(player, new UiMsg.Open(com.lkjmc.common.ui.kernel.MenuRoute.root()));
    }

    public void openFromRoot(Player player, MenuRoute route) {
        sessions.put(player.getUniqueId(), UiModel.root(player.getUniqueId().toString(), ids()));
        dispatch(player, new UiMsg.Open(route));
    }

    public void close(Player player, String sessionId) {
        var current = sessions.get(player.getUniqueId());
        if (current != null && current.sessionId().equals(sessionId)) {
            sessions.remove(player.getUniqueId(), current);
        }
    }

    public void quit(Player player) {
        sessions.remove(player.getUniqueId());
    }

    public Optional<UiModel> model(Player player) {
        return Optional.ofNullable(sessions.get(player.getUniqueId()));
    }

    BindingContext context(Player player, UiModel model) {
        return new BindingContext(player.getUniqueId().toString(), player.getName(), locale(player),
            bindingParams(model.route()), permissions(player), localData(player));
    }

    String locale(Player player) {
        return locales == null ? "en" : locales.apply(player);
    }

    private PermissionsView permissions(Player player) {
        var identity = new PrincipalIdentity("minecraft-player", player.getUniqueId().toString(), player.getName());
        var snapshot = grants.snapshot(identity).orElse(null);
        return new PermissionsView(
            allowed(player, identity, PermissionNodes.ADMIN_STATUS, snapshot),
            allowed(player, identity, PermissionNodes.ADMIN_RELOAD, snapshot),
            allowed(player, identity, PermissionNodes.ADMIN_ADMIN, snapshot),
            allowed(player, identity, PermissionNodes.ADMIN_ECONOMY, snapshot),
            allowed(player, identity, PermissionNodes.ADMIN_ANNOUNCE, snapshot),
            allowed(player, identity, PermissionNodes.ADMIN_REPORTS, snapshot),
            allowed(player, identity, PermissionNodes.ADMIN_WARN, snapshot),
            allowed(player, identity, PermissionNodes.ADMIN_BAN, snapshot),
            allowed(player, identity, PermissionNodes.ADMIN_MUTE, snapshot),
            allowed(player, identity, PermissionNodes.ADMIN_CLAIM, snapshot),
            allowed(player, identity, PermissionNodes.ADMIN_INSTANCE_LIST, snapshot),
            allowed(player, identity, PermissionNodes.ADMIN_INSTANCE_CREATE, snapshot),
            allowed(player, identity, PermissionNodes.ADMIN_INSTANCE_START, snapshot),
            allowed(player, identity, PermissionNodes.ADMIN_INSTANCE_STOP, snapshot),
            allowed(player, identity, PermissionNodes.ADMIN_INSTANCE_RESTART, snapshot),
            allowed(player, identity, PermissionNodes.ADMIN_INSTANCE_DELETE, snapshot));
    }

    private boolean allowed(Player player, PrincipalIdentity identity, String node,
                            com.lkjmc.common.permission.PermissionSnapshot snapshot) {
        var platform = player.hasPermission(node) || player.isOp();
        return resolver.resolve(node, platform, player.isOp(), snapshot, Instant.now()).allowed();
    }

    private Map<String, String> bindingParams(MenuRoute route) {
        var values = new HashMap<>(route.params());
        switch (route.id()) {
            case "random-teleport-nether-confirm" -> values.putIfAbsent("profileId", "nether");
            case "random-teleport-end-confirm" -> values.putIfAbsent("profileId", "end");
            case "random-teleport-overworld" -> values.putIfAbsent("profileId", "overworld");
            default -> { }
        }
        if (route.id().startsWith("random-teleport-")) {
            values.putIfAbsent("serverId", instanceId());
        }
        return Map.copyOf(values);
    }

    private LocalData localData(Player player) {
        var current = instanceId();
        var players = onlinePlayers.get().stream()
            .map(value -> new LocalData.OnlinePlayer(value.getUniqueId().toString(), value.getName(), current))
            .toList();
        return new LocalData(docs.get(), players);
    }

    private void render(Player player) {
        var current = sessions.get(player.getUniqueId());
        if (current != null) renderer.render(player, locale(player), current);
    }

    private String ids() {
        return ids.nextSessionId();
    }

    private static UiRequest responseRequest(UiMsg msg) {
        return switch (msg) {
            case UiMsg.DataLoaded loaded -> loaded.request();
            case UiMsg.DataEmpty empty -> empty.request();
            case UiMsg.DataDenied denied -> denied.request();
            case UiMsg.DataFailed failed -> failed.request();
            case UiMsg.StaleAvailable stale -> stale.request();
            default -> null;
        };
    }

    private static String instanceId() {
        return System.getenv().getOrDefault("LKJMC_INSTANCE_ID", "paper");
    }
}
