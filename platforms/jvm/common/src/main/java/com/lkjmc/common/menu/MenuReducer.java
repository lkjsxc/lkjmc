package com.lkjmc.common.menu;

import java.util.List;
import java.util.Optional;

public final class MenuReducer {
    private MenuReducer() {}

    public static MenuRendererModel render(MenuSpec spec, MenuContext context, DynamicMenuModel data) {
        return new MenuRendererModel(spec, new MenuState(spec.id(), 0), spec.slots());
    }

    public static MenuDecision click(MenuSpec spec, MenuState state, MenuClick click) {
        if (!click.topInventory() || click.slot() < 0) {
            return noop();
        }
        var slot = findSlot(spec, click.slot());
        if (slot.isEmpty()) {
            return unknownOrEmpty(click);
        }
        var failure = metadataFailure(slot.get(), state, click);
        if (failure.isPresent()) {
            return failure(failure.get());
        }
        if (slot.get().item().inert()) {
            return noop();
        }
        return new MenuDecision(effects(slot.get().action()));
    }

    private static Optional<SlotSpec> findSlot(MenuSpec spec, int slot) {
        return spec.slots().stream().filter(value -> value.slot() == slot).findFirst();
    }

    private static MenuDecision unknownOrEmpty(MenuClick click) {
        if (click.metadata() == null && click.actionKey() == null) {
            return noop();
        }
        return failure(MenuFailure.UNKNOWN_METADATA);
    }

    private static Optional<MenuFailure> metadataFailure(SlotSpec slot, MenuState state, MenuClick click) {
        var expected = MenuAction.key(slot.action());
        if (click.metadata() == null) {
            if (click.actionKey() == null) {
                return slot.item().inert() ? Optional.empty() : Optional.of(MenuFailure.UNKNOWN_METADATA);
            }
            return expected.equals(click.actionKey()) ? Optional.empty() : Optional.of(MenuFailure.UNKNOWN_METADATA);
        }
        var metadata = click.metadata();
        if (!metadata.sessionId().isBlank() && !state.sessionId().isBlank()
            && !metadata.sessionId().equals(state.sessionId())) {
            return Optional.of(MenuFailure.STALE_SESSION);
        }
        if (state.renderEpoch() > 0 && metadata.renderEpoch() != state.renderEpoch()) {
            return Optional.of(MenuFailure.STALE_EPOCH);
        }
        if (!metadata.menuId().equals(state.current()) || !metadata.route().equals(state.route())) {
            return Optional.of(MenuFailure.ROUTE_MISMATCH);
        }
        if (metadata.slot() != slot.slot() || !expected.equals(metadata.actionKey())) {
            return Optional.of(MenuFailure.UNKNOWN_METADATA);
        }
        return Optional.empty();
    }

    private static List<MenuEffect> effects(MenuAction action) {
        return switch (action) {
            case MenuAction.None ignored -> List.of();
            case MenuAction.OpenRoute open -> List.of(new MenuEffect.OpenRoute(open.route()));
            case MenuAction.Back ignored -> List.of(new MenuEffect.OpenPrevious());
            case MenuAction.Close ignored -> List.of(new MenuEffect.CloseMenu());
            case MenuAction.RefreshRoute ignored -> List.of(new MenuEffect.RefreshRoute());
            case MenuAction.RunPlayerCommand command -> List.of(new MenuEffect.RunPlayerCommand(command.command()));
            case MenuAction.DaemonCommand command -> List.of(new MenuEffect.SendDaemonCommand(command.command(), command.body()));
            case MenuAction.Transfer transfer -> List.of(new MenuEffect.TransferPlayer(transfer.serverId()));
            case MenuAction.Confirm confirm -> List.of(new MenuEffect.OpenRoute(confirm.route()));
            case MenuAction.Disabled disabled -> List.of(new MenuEffect.SendMessage(disabled.reasonKey()));
            case MenuAction.TextInput input -> List.of(new MenuEffect.PromptText(input.promptKey(), input.commandPrefix()));
            case MenuAction.Select ignored -> List.of(new MenuEffect.SendMessage(MenuFailure.UNHANDLED_ACTION.messageKey()));
            case MenuAction.Purchase ignored -> List.of(new MenuEffect.SendMessage(MenuFailure.UNHANDLED_ACTION.messageKey()));
            case MenuAction.Toggle ignored -> List.of(new MenuEffect.SendMessage(MenuFailure.UNHANDLED_ACTION.messageKey()));
        };
    }

    private static MenuDecision failure(MenuFailure failure) {
        return new MenuDecision(List.of(new MenuEffect.SendMessage(failure.messageKey())), failure);
    }

    private static MenuDecision noop() {
        return new MenuDecision(List.of());
    }
}
