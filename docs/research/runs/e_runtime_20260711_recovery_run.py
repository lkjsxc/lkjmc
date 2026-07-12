import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import time

from e_runtime_20260711_recovery import alive, stop_pid


SCRIPT = Path(__file__).with_name("e_runtime_20260711_recovery.py")


def wait_ready(path):
    for _ in range(40):
        if path.is_file():
            return json.loads(path.read_text(encoding="utf-8"))
        time.sleep(0.05)
    raise TimeoutError("crashed coordinator did not persist its claim")


def save(path, value):
    path.write_text(json.dumps(value, sort_keys=True, indent=2) + "\n", encoding="utf-8")


def recovery_run(raw):
    journal, ready = raw / "recovery-journal.jsonl", raw / "recovery-ready.json"
    result, pids = {"state": "FAIL", "reason": "recovery did not run"}, []
    coordinator = recovery = None
    try:
        coordinator = subprocess.Popen([sys.executable, str(SCRIPT), "hold", str(journal), str(ready)],
                                       stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, start_new_session=True)
        claim = wait_ready(ready)
        pids.append(claim["pid"])
        os.kill(coordinator.pid, signal.SIGKILL)
        coordinator.wait(timeout=2)
        output = coordinator.communicate(timeout=2)
        (raw / "recovery-crash.txt").write_text((output[0] + output[1])[:8192], encoding="utf-8")
        recovery = subprocess.Popen([sys.executable, str(SCRIPT), "recover", str(journal)], stdout=subprocess.PIPE,
                                    stderr=subprocess.PIPE, text=True, start_new_session=True)
        stdout, stderr = recovery.communicate(timeout=10)
        (raw / "recovery-restart.txt").write_text((stdout + stderr)[:8192], encoding="utf-8")
        recovered = json.loads(stdout)
        pids.append(recovered["fresh_pid"])
        valid = coordinator.returncode == -signal.SIGKILL and claim["coordinator_pid"] == coordinator.pid
        valid = valid and recovered["coordinator_pid"] == coordinator.pid and recovered["stale_pid"] == claim["pid"]
        valid = valid and recovered["stale_was_alive"] and recovered["fenced"] and recovered["stale_rejected"]
        valid = valid and recovered["generation"] == 2 and recovered["fresh_alive"] and recovered["fresh_stopped"]
        valid = valid and recovery.returncode == 0 and not recovered["survivors"]
        result = {"state": "PASS" if valid else "FAIL", "crash_exit": coordinator.returncode,
                  "restart_exit": recovery.returncode, "claim": claim, "recovery": recovered}
    except (OSError, TimeoutError, ValueError, subprocess.TimeoutExpired) as error:
        result = {"state": "FAIL", "reason": str(error)}
    finally:
        for process in (coordinator, recovery):
            if process is not None and process.poll() is None:
                os.kill(process.pid, signal.SIGKILL)
                process.wait(timeout=2)
        for pid in pids:
            stop_pid(pid)
        survivors = [pid for pid in pids if alive(pid)]
        result["cleanup"] = {"owned_pids": pids, "survivors": survivors}
        if survivors:
            result["state"] = "FAIL"
        save(raw / "recovery-result.json", result)
    return result
