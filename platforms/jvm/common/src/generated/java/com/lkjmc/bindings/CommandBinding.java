package com.lkjmc.bindings;

public record CommandBinding(String name, CommandEffect effect, String response, CommandErrorBoundary errors) {}
