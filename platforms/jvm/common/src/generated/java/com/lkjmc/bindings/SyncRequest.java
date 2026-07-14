package com.lkjmc.bindings;

public sealed interface SyncRequest permits SnapshotRequest, FeedRequest {}
