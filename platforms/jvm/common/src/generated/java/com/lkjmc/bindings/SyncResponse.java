package com.lkjmc.bindings;

public sealed interface SyncResponse permits TypedSnapshot, FeedResponse, ReloadRequired {}
