package com.lkjmc.common.ui.document;

import java.util.ArrayList;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.Set;

public final class MenuDocumentValidator {
    private static final Set<String> THEMES = Set.of("root", "network", "travel", "claims", "economy",
        "social", "profile", "settings", "staff", "adventure", "danger", "docs");
    private static final Set<String> CONFIRMATIONS = Set.of("deletes-durable-state",
        "overwrites-named-durable-state", "creates-durable-world-state", "writes-named-durable-state",
        "stops-server", "forceful-server-mutation", "starts-durable-resources",
        "starts-temporary-infrastructure", "affects-other-players", "changes-moderation-state",
        "paid-dimension-change");

    private MenuDocumentValidator() {}

    public static List<ValidationError> validate(MenuDocumentSet set) {
        var errors = new ArrayList<ValidationError>();
        for (var document : set.documents()) {
            validateDocument(set, document, errors);
        }
        return List.copyOf(errors);
    }

    private static void validateDocument(MenuDocumentSet set, MenuDocument document,
                                         List<ValidationError> errors) {
        if (!document.id().matches("[a-z0-9]+(?:-[a-z0-9]+)*")) {
            errors.add(new InvalidId(document.id()));
        }
        if (!THEMES.contains(document.theme())) {
            errors.add(new InvalidTheme(document.id(), document.theme()));
        }
        if (document.size() != 27 && document.size() != 54) {
            errors.add(new InvalidSize(document.id(), document.size()));
        }
        if (document.parent() != null && !set.contains(document.parent())) {
            errors.add(new UnknownParent(document.id(), document.parent()));
        }
        validateParams(document, errors);
        validateKind(document, errors);
        validateRegions(document, errors);
        validateSlots(set, document, errors);
        validateConfirmation(document, errors);
    }

    private static void validateParams(MenuDocument document, List<ValidationError> errors) {
        var names = new HashSet<String>();
        for (var param : document.params()) {
            if (!param.name().matches("[A-Za-z][A-Za-z0-9_-]*")) {
                errors.add(new InvalidParam(document.id(), param.name()));
            }
            if (!names.add(param.name())) {
                errors.add(new DuplicateParam(document.id(), param.name()));
            }
        }
    }

    private static void validateKind(MenuDocument document, List<ValidationError> errors) {
        if (document.kind() == MenuDocument.Kind.LIST && document.list() == null) {
            errors.add(new KindRule(document.id(), "list routes require list grammar"));
        }
        if (document.kind() != MenuDocument.Kind.LIST && document.list() != null) {
            errors.add(new KindRule(document.id(), "only list routes may declare list grammar"));
        }
        if (document.kind() == MenuDocument.Kind.STATIC && document.data() != null) {
            errors.add(new KindRule(document.id(), "static routes may not declare data"));
        }
    }

    private static void validateRegions(MenuDocument document, List<ValidationError> errors) {
        if (document.list() == null) {
            return;
        }
        if (!RegionCatalog.exists(document.list().region())) {
            errors.add(new UnknownRegion(document.id(), document.list().region()));
        }
        if (document.list().reserved() != null && !RegionCatalog.exists(document.list().reserved())) {
            errors.add(new UnknownRegion(document.id(), document.list().reserved()));
        }
    }

    private static void validateSlots(MenuDocumentSet set, MenuDocument document,
                                      List<ValidationError> errors) {
        var occupied = new HashSet<Integer>();
        var chrome = chromeSlots(document);
        for (var slot : document.staticSlots()) {
            if (slot.slot() >= document.size()) {
                errors.add(new SlotOutOfBounds(document.id(), slot.slot()));
            }
            if (!occupied.add(slot.slot())) {
                errors.add(new DuplicateSlot(document.id(), slot.slot()));
            }
            if (chrome.contains(slot.slot())) {
                errors.add(new ChromeCollision(document.id(), slot.slot()));
            }
            validateAction(set, document, slot.action(), errors);
        }
    }

    private static Set<Integer> chromeSlots(MenuDocument document) {
        var slots = new HashSet<Integer>();
        if (document.chrome().info() != null) { slots.add(4); }
        if (document.chrome().mainMenu()) { slots.add(45); }
        if (document.chrome().back()) { slots.add(49); }
        if (document.chrome().refresh()) { slots.add(50); }
        if (document.chrome().close()) { slots.add(document.size() == 27 ? 26 : 53); }
        return slots;
    }

    private static void validateConfirmation(MenuDocument document, List<ValidationError> errors) {
        if (document.kind() == MenuDocument.Kind.CONFIRM) {
            if (document.size() != 27) {
                errors.add(new KindRule(document.id(), "confirm routes must be 27 slots"));
            }
            if (document.confirmation() == null || !CONFIRMATIONS.contains(document.confirmation())) {
                errors.add(new InvalidConfirmation(document.id(), document.confirmation()));
            }
        } else if (document.confirmation() != null) {
            errors.add(new InvalidConfirmation(document.id(), document.confirmation()));
        }
    }

    private static void validateAction(MenuDocumentSet set, MenuDocument source, DocumentAction action,
                                       List<ValidationError> errors) {
        if (action instanceof DocumentAction.Open open) {
            validateOpen(set, source, open, errors);
        } else if (action instanceof DocumentAction.Daemon daemon) {
            validateTokens(source, daemon.body(), errors);
        }
    }

    private static void validateOpen(MenuDocumentSet set, MenuDocument source, DocumentAction.Open open,
                                     List<ValidationError> errors) {
        var target = set.document(open.route());
        if (target.isEmpty()) {
            errors.add(new UnknownTarget(source.id(), open.route()));
            return;
        }
        var declared = new HashSet<String>();
        var required = new HashSet<String>();
        for (var param : target.get().params()) {
            declared.add(param.name());
            if (param.required()) { required.add(param.name()); }
        }
        for (var key : open.params().keySet()) {
            if (!declared.contains(key)) { errors.add(new UndeclaredParam(source.id(), open.route(), key)); }
        }
        for (var needed : required) {
            if (!open.params().containsKey(needed)) {
                errors.add(new MissingOpenParam(source.id(), open.route(), needed));
            }
        }
        validateTokens(source, open.params(), errors);
    }

    private static void validateTokens(MenuDocument source, Map<String, String> values,
                                       List<ValidationError> errors) {
        var declared = new HashSet<String>();
        source.params().forEach(param -> declared.add(param.name()));
        values.values().stream().filter(value -> value != null && value.startsWith("@param."))
            .map(value -> value.substring("@param.".length()))
            .filter(name -> !declared.contains(name))
            .forEach(name -> errors.add(new InvalidAction(source.id(), "unknown param token: " + name)));
    }

    public sealed interface ValidationError permits InvalidId, InvalidTheme, InvalidSize, UnknownParent,
        InvalidParam, DuplicateParam, KindRule, UnknownRegion, SlotOutOfBounds, DuplicateSlot,
        ChromeCollision, InvalidConfirmation, UnknownTarget, UndeclaredParam, MissingOpenParam,
        InvalidAction {}
    public record InvalidId(String id) implements ValidationError {}
    public record InvalidTheme(String id, String theme) implements ValidationError {}
    public record InvalidSize(String id, int size) implements ValidationError {}
    public record UnknownParent(String id, String parent) implements ValidationError {}
    public record InvalidParam(String id, String param) implements ValidationError {}
    public record DuplicateParam(String id, String param) implements ValidationError {}
    public record KindRule(String id, String rule) implements ValidationError {}
    public record UnknownRegion(String id, String region) implements ValidationError {}
    public record SlotOutOfBounds(String id, int slot) implements ValidationError {}
    public record DuplicateSlot(String id, int slot) implements ValidationError {}
    public record ChromeCollision(String id, int slot) implements ValidationError {}
    public record InvalidConfirmation(String id, String reason) implements ValidationError {}
    public record UnknownTarget(String sourceId, String targetId) implements ValidationError {}
    public record UndeclaredParam(String sourceId, String targetId, String param) implements ValidationError {}
    public record MissingOpenParam(String sourceId, String targetId, String param) implements ValidationError {}
    public record InvalidAction(String id, String reason) implements ValidationError {}
}
