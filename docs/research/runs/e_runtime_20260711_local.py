import concurrent.futures as futures
import threading
import time

from e_runtime_20260711_coordination import Direct, Keyed, Mailbox
from e_runtime_20260711_slice import Journal, NullJournal, Slice


COORDINATORS = {"direct": Direct, "keyed": Keyed, "mailbox": Mailbox}
SPECS = [("uncoordinated-baseline", "direct", 2, False), ("keyed", "keyed", 2, False),
         ("mailbox", "mailbox", 2, False), ("bounded-keyed", "keyed", 2, False),
         ("async-keyed", "keyed", 8, False), ("bounded-journal", "direct", 2, True),
         ("async-journal", "direct", 8, True), ("bounded-journal-keyed", "keyed", 2, True),
         ("async-journal-keyed", "keyed", 8, True), ("bounded-journal-mailbox", "mailbox", 2, True),
         ("async-journal-mailbox", "mailbox", 8, True)]


def run_scenario(kind, workers, journal_path, journalled):
    coordinator = COORDINATORS[kind]()
    slice_, journal, release = Slice(), Journal(journal_path) if journalled else NullJournal(), threading.Event()

    def call(instance, action, barrier=None, entered=None):
        journal.add("intent", instance, action)

        def work():
            if action == "hold":
                entered.set()
                if not release.wait(timeout=2):
                    return "hold_timeout", None
                return slice_.start(instance)
            methods = {"start": lambda: slice_.start(instance), "race": lambda: slice_.start(instance, barrier),
                       "effect": lambda: slice_.fail_effect(instance), "observe": lambda: slice_.fail_observation(instance)}
            return methods[action]()

        outcome, pid = coordinator.call(instance, work)
        journal.add("outcome", instance, {"state": outcome, "pid": pid})
        return outcome, pid

    shutdown = {"owned_pids": [], "stopped": [], "survivors": []}
    try:
        barrier = threading.Barrier(2) if kind == "direct" else None
        with futures.ThreadPoolExecutor(max_workers=workers) as pool:
            action = "race" if barrier else "start"
            same = [pool.submit(call, "same", action, barrier) for _ in range(2)]
            same = [future.result(timeout=3)[0] for future in same]
            entered = threading.Event()
            held = pool.submit(call, "hung", "hold", None, entered)
            entered_ok = entered.wait(timeout=1)
            started = time.monotonic()
            peer = pool.submit(call, "other", "start").result(timeout=1)[0]
            peer_ms = round((time.monotonic() - started) * 1000, 1)
            peer_while_held = entered_ok and not held.done() and not release.is_set()
            release.set()
            held_outcome = held.result(timeout=3)[0]
        stale, pid = call("stale", "start")
        fenced = slice_.fence("stale", pid)[0] if stale == "started" else "missing"
        effect, observation = call("effect", "effect")[0], call("observe", "observe")[0]
        shutdown = slice_.shutdown()
        journal.add("shutdown", "all", shutdown)
        safe = same.count("started") == 1 and peer == "started" and peer_ms < 150
        safe = safe and entered_ok and peer_while_held and held_outcome == "started"
        safe = safe and fenced == "fenced" and effect == "effect_failed" and observation == "observation_failed"
        return {"safe": safe and not shutdown["survivors"], "same_effects": same.count("started"),
                "peer_ms": peer_ms, "hold_entered": entered_ok, "peer_while_held": peer_while_held,
                "journal_events": journal.count, "shutdown": shutdown}
    finally:
        release.set()
        if slice_.children:
            slice_.shutdown()
        coordinator.close()
        journal.close()


def local_run(raw):
    results = {}
    for name, kind, workers, journalled in SPECS:
        rows = [run_scenario(kind, workers, raw / f"{name}-{attempt}.jsonl", journalled) for attempt in range(3)]
        safe, unsafe = all(row["safe"] for row in rows), kind == "direct"
        results[name] = {"state": "EXPECTED_UNSAFE" if unsafe and not safe else "PASS" if safe else "FAIL",
                         "scheduler": "bounded" if workers == 2 else "async-dispatch", "workers": workers,
                         "journal": journalled, "same_effects": [row["same_effects"] for row in rows],
                         "peer_ms": [row["peer_ms"] for row in rows], "holds": [row["hold_entered"] for row in rows],
                         "peer_while_held": [row["peer_while_held"] for row in rows],
                         "survivors": [row["shutdown"]["survivors"] for row in rows]}
    return results
