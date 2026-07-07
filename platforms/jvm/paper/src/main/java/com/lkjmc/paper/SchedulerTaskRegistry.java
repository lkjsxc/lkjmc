package com.lkjmc.paper;

import io.papermc.paper.threadedregions.scheduler.ScheduledTask;
import java.util.List;
import java.util.Set;
import java.util.concurrent.ConcurrentHashMap;

final class SchedulerTaskRegistry {
    private final Set<ScheduledTask> tasks = ConcurrentHashMap.newKeySet();

    void track(ScheduledTask task) {
        tasks.add(task);
    }

    void complete(ScheduledTask task) {
        tasks.remove(task);
    }

    int size() {
        return tasks.size();
    }

    void cancelAll() {
        for (ScheduledTask task : List.copyOf(tasks)) {
            task.cancel();
        }
        tasks.clear();
    }
}
