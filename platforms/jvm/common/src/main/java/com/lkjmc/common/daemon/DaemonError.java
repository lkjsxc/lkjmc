package com.lkjmc.common.daemon;

public record DaemonError(String code, String message, boolean retryable) {}
