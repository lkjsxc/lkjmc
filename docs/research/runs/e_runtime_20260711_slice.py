import json
import os
import signal
import subprocess
import threading
import time


class Journal:
    def __init__(self, path):
        self.file = open(path, "a", encoding="utf-8")
        self.lock, self.count = threading.Lock(), 0

    def add(self, event, instance, result=""):
        with self.lock:
            self.file.write(json.dumps({"event": event, "instance": instance, "result": result}) + "\n")
            self.file.flush()
            os.fsync(self.file.fileno())
            self.count += 1

    def close(self):
        self.file.close()


class NullJournal:
    count = 0

    def add(self, _event, _instance, _result=""):
        pass

    def close(self):
        pass


class Slice:
    def __init__(self):
        self.state, self.children, self.events = {}, {}, []
        self.lock = threading.Lock()

    def start(self, instance, barrier=None):
        with self.lock:
            if self.state.get(instance) == "running":
                return "deduped", None
        if barrier is not None:
            barrier.wait(timeout=1)
        child = subprocess.Popen(["sh", "-c", "exec sleep 30"], start_new_session=True)
        time.sleep(0.01)
        if child.poll() is not None:
            return "observation_failed", None
        with self.lock:
            self.state[instance] = "running"
            self.children[child.pid] = (instance, child)
            self.events.append(("effect", instance, child.pid))
        return "started", child.pid

    def fail_effect(self, instance):
        status = subprocess.run(["sh", "-c", "exit 7"], check=False).returncode
        self.events.append(("effect_failed", instance, status))
        return "effect_failed", None

    def fail_observation(self, instance):
        child = subprocess.Popen(["sh", "-c", "exit 0"], start_new_session=True)
        child.wait(timeout=1)
        self.events.append(("observation_failed", instance, child.returncode))
        return "observation_failed", None

    @staticmethod
    def stop(child):
        if child.poll() is None:
            try:
                os.killpg(child.pid, signal.SIGTERM)
            except ProcessLookupError:
                pass
        try:
            child.wait(timeout=2)
        except subprocess.TimeoutExpired:
            try:
                os.killpg(child.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            child.wait(timeout=2)
        return child.poll() is not None

    def fence(self, instance, pid):
        with self.lock:
            entry = self.children.get(pid)
        if entry is None:
            return "untracked", None
        if not self.stop(entry[1]):
            return "fence_failed", None
        with self.lock:
            self.state[instance] = "fenced"
            self.events.append(("fenced", instance, pid))
        return "fenced", None

    def shutdown(self):
        with self.lock:
            owned = list(self.children.items())
        stopped = [pid for pid, (_instance, child) in owned if self.stop(child)]
        survivors = [pid for pid, (_instance, child) in owned if child.poll() is None]
        with self.lock:
            self.children.clear()
            for _pid, (instance, _child) in owned:
                self.state[instance] = "stopped"
            self.events.append(("shutdown", "all", len(stopped)))
        return {"owned_pids": [pid for pid, _entry in owned], "stopped": stopped, "survivors": survivors}
