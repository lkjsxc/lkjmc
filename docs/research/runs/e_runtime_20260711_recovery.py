#!/usr/bin/env python3
import argparse
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import time


def append(path, event, **fields):
    row = {"event": event, **fields}
    with path.open("a", encoding="utf-8") as file:
        file.write(json.dumps(row, sort_keys=True) + "\n")
        file.flush()
        os.fsync(file.fileno())
    return row


def save(path, value):
    with path.open("w", encoding="utf-8") as file:
        json.dump(value, file, sort_keys=True)
        file.write("\n")
        file.flush()
        os.fsync(file.fileno())


def alive(pid):
    stat = Path(f"/proc/{pid}/stat")
    if stat.exists():
        try:
            return stat.read_text(encoding="utf-8").split(") ", 1)[1].split()[0] != "Z"
        except (IndexError, OSError):
            return False
    try:
        os.kill(pid, 0)
        return True
    except ProcessLookupError:
        return False


def stop_pid(pid):
    if alive(pid):
        try:
            os.killpg(pid, signal.SIGTERM)
        except ProcessLookupError:
            pass
    for _ in range(40):
        if not alive(pid):
            return True
        time.sleep(0.05)
    return False


def spawn():
    return subprocess.Popen(["sh", "-c", "exec sleep 30"], start_new_session=True,
                            stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)


def hold(journal, ready):
    child = spawn()
    claim = append(journal, "claimed", coordinator_pid=os.getpid(), generation=1, pid=child.pid)
    save(ready, claim)
    try:
        while True:
            time.sleep(1)
    finally:
        stop_pid(child.pid)


def claims(journal):
    rows = [json.loads(line) for line in journal.read_text(encoding="utf-8").splitlines()]
    return [row for row in rows if row["event"] == "claimed"]


def recover(journal):
    previous = claims(journal)[-1]
    stale_pid, old_generation = previous["pid"], previous["generation"]
    was_alive = alive(stale_pid)
    fenced = stop_pid(stale_pid)
    generation = old_generation + 1
    append(journal, "fenced", stale_pid=stale_pid, stale_alive=was_alive, fenced=fenced)
    stale_rejected = fenced and old_generation < generation
    append(journal, "stale_rejected", stale_generation=old_generation, generation=generation,
           rejected=stale_rejected)
    child = spawn()
    fresh_alive = alive(child.pid)
    append(journal, "reacquired", coordinator_pid=os.getpid(), generation=generation, pid=child.pid)
    fresh_stopped = stop_pid(child.pid)
    try:
        child.wait(timeout=2)
    except subprocess.TimeoutExpired:
        fresh_stopped = False
    survivors = [pid for pid in (stale_pid, child.pid) if alive(pid)]
    append(journal, "shutdown", owned_pids=[stale_pid, child.pid], survivors=survivors)
    return {"coordinator_pid": previous["coordinator_pid"], "stale_pid": stale_pid,
            "stale_was_alive": was_alive, "fenced": fenced, "stale_rejected": stale_rejected,
            "generation": generation, "recovery_pid": os.getpid(), "fresh_pid": child.pid,
            "fresh_alive": fresh_alive, "fresh_stopped": fresh_stopped, "survivors": survivors}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("action", choices=("hold", "recover"))
    parser.add_argument("journal", type=Path)
    parser.add_argument("ready", type=Path, nargs="?")
    args = parser.parse_args()
    if args.action == "hold":
        if args.ready is None:
            parser.error("hold requires ready path")
        hold(args.journal, args.ready)
        return 0
    print(json.dumps(recover(args.journal), sort_keys=True))
    return 0


if __name__ == "__main__":
    sys.exit(main())
