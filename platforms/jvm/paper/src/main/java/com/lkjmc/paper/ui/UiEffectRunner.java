package com.lkjmc.paper.ui;

import com.google.gson.JsonObject;
import com.lkjmc.common.daemon.DaemonClient;
import com.lkjmc.common.daemon.DaemonResponse;
import com.lkjmc.common.i18n.MessageCatalog;
import com.lkjmc.common.ui.binding.BindingDecodeException;
import com.lkjmc.common.ui.binding.BindingRegistry;
import com.lkjmc.common.ui.binding.BindingResult;
import com.lkjmc.common.ui.binding.MenuBinding;
import com.lkjmc.common.ui.kernel.DaemonRequestPlan;
import com.lkjmc.common.ui.kernel.TextRef;
import com.lkjmc.common.ui.kernel.UiEffect;
import com.lkjmc.common.ui.kernel.UiModel;
import com.lkjmc.common.ui.kernel.UiMsg;
import com.lkjmc.common.ui.kernel.UiRequest;
import com.lkjmc.paper.SchedulerBridge;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;
import org.bukkit.entity.Player;

public final class UiEffectRunner implements UiSessionService.Effects {
    private final SchedulerBridge scheduler;
    private final Optional<DaemonClient> daemon;
    private final BindingRegistry bindings;
    private final UiStaleCache stale;
    private final UiTextInput textInput;
    private final UiText text;
    private final MessageCatalog catalog;
    private final UiTransferPort transfers;

    public UiEffectRunner(SchedulerBridge scheduler, Optional<DaemonClient> daemon,
                          BindingRegistry bindings, UiStaleCache stale, UiTextInput textInput,
                          UiText text, MessageCatalog catalog, UiTransferPort transfers) {
        this.scheduler = scheduler;
        this.daemon = daemon == null ? Optional.empty() : daemon;
        this.bindings = bindings;
        this.stale = stale;
        this.textInput = textInput;
        this.text = text;
        this.catalog = catalog;
        this.transfers = transfers;
    }

    @Override
    public void run(Player player, UiEffect effect, UiModel model, UiSessionService sessions) {
        switch (effect) {
            case UiEffect.LoadData load -> load(player, load, model, sessions);
            case UiEffect.SendDaemon command -> sendDaemon(player, command, sessions);
            case UiEffect.RunCommand command -> player.performCommand(command.command());
            case UiEffect.Transfer transfer -> transfer(player, transfer.serverId(), sessions.locale(player));
            case UiEffect.Message message -> message(player, sessions.locale(player), message.text());
            case UiEffect.PromptText prompt -> textInput.start(player, prompt.prompt(), prompt.commandPrefix(), sessions);
            case UiEffect.CloseInventory ignored -> player.closeInventory();
        }
    }

    private void load(Player player, UiEffect.LoadData effect, UiModel model, UiSessionService sessions) {
        var request = effect.request().forPlayer(player.getUniqueId().toString());
        var binding = bindings.require(effect.plan().binding());
        var ctx = sessions.context(player, model);
        var plan = merge(effect.plan(), binding.plan(ctx));
        if ("local".equals(plan.source())) {
            decode(player, binding, new JsonObject(), ctx, model, request, sessions);
        } else if (daemon.isEmpty()) {
            failLoad(player, model, request, sessions, "daemon.not_configured");
        } else if (commands(plan).isEmpty() || commands(plan).size() > 2) {
            failLoad(player, model, request, sessions, "menu.decode." + binding.id());
        } else {
            var futures = commands(plan).stream()
                .map(command -> daemon.get().send(UiDaemonRequests.request(player, command, plan.body()))).toList();
            CompletableFuture.allOf(futures.toArray(CompletableFuture[]::new)).whenComplete((ok, error) ->
                scheduler.runPlayer(player, () -> completeLoad(player, binding, futures, error, model, request, sessions)));
        }
    }

    private void completeLoad(Player player, MenuBinding binding, List<CompletableFuture<DaemonResponse>> futures,
                              Throwable error, UiModel model, UiRequest request, UiSessionService sessions) {
        if (!sessions.accepts(player, request)) return;
        if (error != null) {
            failLoad(player, model, request, sessions, UiDaemonRequests.diagnostic(error));
            return;
        }
        var responses = futures.stream().map(CompletableFuture::join).toList();
        var failed = responses.stream().filter(response -> response == null || !response.ok()).findFirst();
        if (failed.isPresent()) {
            failLoad(player, model, request, sessions, UiDaemonRequests.diagnostic(failed.get()));
            return;
        }
        decode(player, binding, UiDaemonRequests.merge(responses), sessions.context(player, model), model, request, sessions);
    }

    private void decode(Player player, MenuBinding binding, JsonObject body,
                        com.lkjmc.common.ui.binding.BindingContext ctx, UiModel model,
                        UiRequest request, UiSessionService sessions) {
        if (!sessions.accepts(player, request)) return;
        try {
            switch (binding.decode(body, ctx)) {
                case BindingResult.Data data -> {
                    stale.remember(player.getUniqueId(), model.route(), data.view());
                    sessions.dispatch(player, new UiMsg.DataLoaded(data.view(), request));
                }
                case BindingResult.Empty ignored -> sessions.dispatch(player, new UiMsg.DataEmpty(request));
                case BindingResult.Denied ignored -> sessions.dispatch(player, new UiMsg.DataDenied(request));
            }
        } catch (BindingDecodeException error) {
            failLoad(player, model, request, sessions, error.code());
        } catch (RuntimeException error) {
            failLoad(player, model, request, sessions, "menu.decode." + binding.id());
        }
    }

    private void failLoad(Player player, UiModel model, UiRequest request, UiSessionService sessions, String code) {
        if (!sessions.accepts(player, request)) return;
        stale.find(player.getUniqueId(), model.route()).ifPresentOrElse(
            view -> sessions.dispatch(player, new UiMsg.StaleAvailable(view, code, request)),
            () -> sessions.dispatch(player, new UiMsg.DataFailed(code, request)));
    }

    private void sendDaemon(Player player, UiEffect.SendDaemon effect, UiSessionService sessions) {
        var request = effect.request().forPlayer(player.getUniqueId().toString());
        if (daemon.isEmpty()) {
            if (sessions.completeMutation(player, request)) {
                diagnosticOrFallback(player, sessions.locale(player), "daemon.not_configured", effect.fail());
            }
            return;
        }
        var command = commands(effect.plan()).stream().findFirst().orElse(effect.plan().command());
        daemon.get().send(UiDaemonRequests.request(player, command, effect.plan().body())).whenComplete((response, error) ->
            scheduler.runPlayer(player, () -> completeDaemon(player, effect, request, sessions, response, error)));
    }

    private void completeDaemon(Player player, UiEffect.SendDaemon effect, UiRequest request,
                                UiSessionService sessions, DaemonResponse response, Throwable error) {
        if (!sessions.completeMutation(player, request)) return;
        if (error != null || response == null || !response.ok()) {
            var code = error == null ? UiDaemonRequests.diagnostic(response) : UiDaemonRequests.diagnostic(error);
            diagnosticOrFallback(player, sessions.locale(player), code, effect.fail());
            return;
        }
        message(player, sessions.locale(player), effect.ok());
        if (effect.refreshOnOk()) sessions.dispatch(player, new UiMsg.RefreshRequested());
    }

    private void transfer(Player player, String serverId, String locale) {
        player.closeInventory();
        message(player, locale, TextRef.key("menu.transfer.sending"));
        transfers.transfer(player, serverId);
    }

    private void diagnosticOrFallback(Player player, String locale, String code, TextRef fallback) {
        var title = diagnostic(code, "title");
        if (catalog.render(locale, ((TextRef.Key) title).key()).equals(((TextRef.Key) title).key())) {
            message(player, locale, fallback);
        } else {
            message(player, locale, title);
            message(player, locale, diagnostic(code, "hint"));
        }
    }

    private void message(Player player, String locale, TextRef ref) { player.sendMessage(text.chat(locale, ref)); }
    private static DaemonRequestPlan merge(DaemonRequestPlan requested, DaemonRequestPlan planned) {
        var commands = requested.commands().isEmpty() ? planned.commands() : requested.commands();
        var source = requested.source().isBlank() ? planned.source() : requested.source();
        return new DaemonRequestPlan(planned.binding(), source, planned.command(), planned.params(), planned.body(), commands);
    }
    private static List<String> commands(DaemonRequestPlan plan) {
        return !plan.commands().isEmpty() ? plan.commands() : plan.command().isBlank() ? List.of() : List.of(plan.command());
    }
    private static TextRef diagnostic(String code, String suffix) {
        if (code != null && code.startsWith("menu.decode.")) {
            return TextRef.key("diagnostic.menu.decode." + suffix, Map.of("route", code.substring(12)));
        }
        return TextRef.key("diagnostic." + (code == null ? "daemon.command_failed" : code) + "." + suffix);
    }
}
