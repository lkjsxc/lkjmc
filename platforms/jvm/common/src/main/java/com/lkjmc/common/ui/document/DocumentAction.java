package com.lkjmc.common.ui.document;

import java.util.LinkedHashMap;
import java.util.Map;

public sealed interface DocumentAction permits DocumentAction.None, DocumentAction.Open,
    DocumentAction.Back, DocumentAction.Close, DocumentAction.Refresh, DocumentAction.Command,
    DocumentAction.Daemon, DocumentAction.Input, DocumentAction.Transfer, DocumentAction.Message,
    DocumentAction.Disabled, DocumentAction.Page {

    static DocumentAction none() {
        return new None();
    }

    static String key(DocumentAction action) {
        return switch (action) {
            case None ignored -> "none";
            case Open open -> "open:" + open.route();
            case Back ignored -> "back";
            case Close ignored -> "close";
            case Refresh ignored -> "refresh";
            case Command command -> "command:" + command.value();
            case Daemon daemon -> "daemon:" + daemon.command();
            case Input input -> "input:" + input.commandPrefix();
            case Transfer transfer -> "transfer:" + transfer.serverId();
            case Message message -> "message:" + message.key();
            case Disabled disabled -> "disabled:" + disabled.reasonKey();
            case Page page -> "page:" + page.direction();
        };
    }

    static Map<String, String> payload(DocumentAction action, Map<String, String> routeParams) {
        var values = new LinkedHashMap<String, String>();
        values.put("type", type(action));
        switch (action) {
            case None none -> { }
            case Back back -> { }
            case Close close -> { }
            case Refresh refresh -> { }
            case Open open -> {
                values.put("route", open.route());
                open.params().forEach((key, value) -> values.put("param." + key, resolve(value, routeParams)));
            }
            case Command command -> values.put("command", command.value());
            case Daemon daemon -> {
                values.put("command", daemon.command());
                daemon.body().forEach((key, value) -> values.put("body." + key, resolve(value, routeParams)));
                values.put("ok", daemon.ok());
                values.put("fail", daemon.fail());
                values.put("refresh", Boolean.toString(daemon.refreshOnOk()));
            }
            case Input input -> {
                values.put("prompt", input.prompt());
                values.put("commandPrefix", input.commandPrefix());
            }
            case Transfer transfer -> values.put("serverId", transfer.serverId());
            case Message message -> {
                values.put("key", message.key());
                message.args().forEach((key, value) -> values.put("arg." + key, resolve(value, routeParams)));
            }
            case Disabled disabled -> values.put("key", disabled.reasonKey());
            case Page page -> values.put("direction", page.direction());
        }
        return Map.copyOf(values);
    }

    private static String type(DocumentAction action) {
        return switch (action) {
            case None ignored -> "none";
            case Open ignored -> "open";
            case Back ignored -> "back";
            case Close ignored -> "close";
            case Refresh ignored -> "refresh";
            case Command ignored -> "command";
            case Daemon ignored -> "daemon";
            case Input ignored -> "input";
            case Transfer ignored -> "transfer";
            case Message ignored -> "message";
            case Disabled ignored -> "disabled";
            case Page ignored -> "page";
        };
    }

    private static String resolve(String value, Map<String, String> routeParams) {
        if (value != null && value.startsWith("@param.")) {
            return routeParams.getOrDefault(value.substring("@param.".length()), "");
        }
        return value == null ? "" : value;
    }

    record None() implements DocumentAction {}
    record Open(String route, Map<String, String> params) implements DocumentAction {
        public Open { params = Map.copyOf(params == null ? Map.of() : params); }
    }
    record Back() implements DocumentAction {}
    record Close() implements DocumentAction {}
    record Refresh() implements DocumentAction {}
    record Command(String value) implements DocumentAction {}
    record Daemon(String command, Map<String, String> body, String ok, String fail,
                  boolean refreshOnOk) implements DocumentAction {
        public Daemon { body = Map.copyOf(body == null ? Map.of() : body); }
    }
    record Input(String prompt, String commandPrefix) implements DocumentAction {}
    record Transfer(String serverId) implements DocumentAction {}
    record Message(String key, Map<String, String> args) implements DocumentAction {
        public Message(String key) {
            this(key, Map.of());
        }
        public Message { args = Map.copyOf(args == null ? Map.of() : args); }
    }
    record Disabled(String reasonKey) implements DocumentAction {}
    record Page(String direction) implements DocumentAction {}
}
