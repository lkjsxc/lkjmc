package com.lkjmc.bindings;

public sealed interface SyncResponse permits TypedSnapshot, SnapshotUnavailable, FeedResponse, ReloadRequired, SyncUnavailable {}
