package com.lkjmc.common.diagnostic;

import com.google.gson.Gson;
import java.time.Duration;
import java.util.concurrent.ArrayBlockingQueue;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.atomic.AtomicLong;
import java.util.function.Consumer;

public final class DiagnosticEmitter implements AutoCloseable {
    public static final int CAPACITY = 128;
    private final ArrayBlockingQueue<DiagnosticEvent> queue = new ArrayBlockingQueue<>(CAPACITY);
    private final AtomicBoolean closed = new AtomicBoolean();
    private final AtomicLong dropped = new AtomicLong();
    private final Consumer<String> sink;
    private final Thread worker;

    public DiagnosticEmitter(String owner, Consumer<String> sink) {
        if (owner == null || owner.isBlank() || owner.length() > 96 || sink == null)
            throw new IllegalArgumentException("bounded diagnostic owner and sink required");
        this.sink = sink;
        worker = Thread.ofVirtual().name("lkjmc-diagnostic-" + owner).start(this::drain);
    }

    public boolean emit(DiagnosticEvent event) {
        if (event == null || closed.get()) return false;
        boolean accepted = queue.offer(event);
        if (!accepted) dropped.incrementAndGet();
        return accepted;
    }

    public long dropped() { return dropped.get(); }
    public int pending() { return queue.size(); }

    @Override
    public void close() { close(Duration.ofSeconds(2)); }

    public boolean close(Duration timeout) {
        if (closed.compareAndSet(false, true)) worker.interrupt();
        try {
            worker.join(Math.max(1, timeout.toMillis()));
            if (worker.isAlive()) worker.interrupt();
            return !worker.isAlive() && queue.isEmpty();
        } catch (InterruptedException interrupted) {
            Thread.currentThread().interrupt();
            worker.interrupt();
            return false;
        }
    }

    private void drain() {
        Gson gson = new Gson();
        while (!closed.get() || !queue.isEmpty()) {
            try {
                DiagnosticEvent event = queue.poll(25, TimeUnit.MILLISECONDS);
                if (event != null) sink.accept(gson.toJson(event));
            } catch (InterruptedException interrupted) {
                if (!closed.get()) Thread.currentThread().interrupt();
            } catch (RuntimeException sinkFailure) {
                dropped.incrementAndGet();
            }
        }
    }
}
