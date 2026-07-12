import os
import re
import signal
import subprocess
import time


def postgres_run(raw, root, capture):
    project = f"lkjmc-eruntime-{os.getpid()}"
    compose = ["docker", "compose", "-p", project, "-f", str(root / "docker-compose.yml")]
    holder, holder_pids = None, []
    result = {"state": "BLOCKED", "reason": "not attempted", "project": project}

    def query(name, sql):
        command = compose + ["exec", "-T", "postgres", "psql", "-v", "ON_ERROR_STOP=1", "-qAt",
                             "-U", "lkjmc", "-d", "lkjmc", "-c", sql]
        outcome = capture(name, command)
        return outcome["out"].strip() if outcome["code"] == 0 else None

    def wait_for(name, expected, attempts=40):
        for poll in range(attempts):
            value = query(f"{name}-{poll}", f"select pg_try_advisory_lock(hashtext('e-runtime-same'))")
            if value == expected:
                return True
            time.sleep(0.05)
        return False

    def crash(name):
        nonlocal holder
        if holder is None:
            return None
        os.killpg(holder.pid, signal.SIGKILL)
        stdout, stderr = holder.communicate(timeout=5)
        (raw / f"{name}.txt").write_text((stdout + stderr)[:8192], encoding="utf-8")
        code, holder = holder.returncode, None
        return code

    try:
        if capture("docker-version", ["docker", "compose", "version"])["code"] != 0:
            result = {"state": "BLOCKED", "reason": "docker compose unavailable", "project": project}
            return result
        if capture("compose-up", compose + ["up", "-d", "postgres"], timeout=120)["code"] != 0:
            result = {"state": "BLOCKED", "reason": "Compose PostgreSQL did not start", "project": project}
            return result
        for attempt in range(30):
            ready = capture(f"postgres-ready-{attempt}", compose + ["exec", "-T", "postgres", "pg_isready",
                            "-U", "lkjmc", "-d", "lkjmc"])
            if ready["code"] == 0:
                break
            time.sleep(1)
        else:
            result = {"state": "BLOCKED", "reason": "Compose PostgreSQL was not ready", "project": project}
            return result
        setup = "create table e_runtime_fence (id text primary key, generation bigint not null); create table e_runtime_effect (id text, generation bigint, outcome text);"
        if query("postgres-setup", setup) is None:
            result = {"state": "BLOCKED", "reason": "Compose PostgreSQL was not SQL-ready", "project": project}
            return result
        rows = []
        for repeat in range(3):
            query(f"postgres-reset-{repeat}", "truncate e_runtime_effect, e_runtime_fence")
            app = f"e-runtime-holder-{repeat}"
            hold = f"select set_config('application_name', '{app}', false); select pg_advisory_lock(hashtext('e-runtime-same')); insert into e_runtime_fence values ('same', 1); select pg_sleep(30)"
            command = compose + ["exec", "-T", "postgres", "psql", "-v", "ON_ERROR_STOP=1", "-qAt",
                                 "-U", "lkjmc", "-d", "lkjmc", "-c", hold]
            holder = subprocess.Popen(command, text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                      start_new_session=True)
            holder_pids.append(holder.pid)
            locked = wait_for(f"postgres-locked-{repeat}", "f")
            backend = query(f"postgres-backend-{repeat}", f"select pid from pg_stat_activity where application_name='{app}'")
            started = time.monotonic()
            other = query(f"postgres-other-{repeat}", "select pg_try_advisory_lock(hashtext('e-runtime-other'))")
            other_ms = round((time.monotonic() - started) * 1000, 1)
            crash_exit = crash(f"postgres-crash-{repeat}")
            released_after_crash = wait_for(f"postgres-released-after-crash-{repeat}", "t", attempts=5)
            terminated = False
            if not released_after_crash and backend and backend.isdigit():
                terminated = query(f"postgres-terminate-backend-{repeat}", f"select pg_terminate_backend({backend})") == "t"
            released = released_after_crash or wait_for(f"postgres-released-after-loss-{repeat}", "t")
            acquire = "select pg_advisory_lock(hashtext('e-runtime-same')); insert into e_runtime_fence values ('same', 2) on conflict (id) do update set generation=excluded.generation returning generation; select pg_advisory_unlock(hashtext('e-runtime-same'))"
            if released:
                values = [int(value) for value in re.findall(r"\d+", query(f"postgres-reacquire-{repeat}", acquire) or "")]
                stale = query(f"postgres-stale-{repeat}", "insert into e_runtime_effect select 'same', 1, 'stale' where (select generation from e_runtime_fence where id='same')=1 returning outcome")
                fresh = query(f"postgres-fresh-{repeat}", "insert into e_runtime_effect select 'same', 2, 'fresh' where (select generation from e_runtime_fence where id='same')=2 returning outcome")
            else:
                values, stale, fresh = [], None, None
            rows.append({"locked": locked, "other": other, "other_ms": other_ms, "holder_pid": holder_pids[-1],
                         "backend_pid": int(backend) if backend and backend.isdigit() else 0, "crash_exit": crash_exit,
                         "released_after_crash": released_after_crash, "backend_terminated": terminated,
                         "connection_released": released, "generation": values[0] if values else 0,
                         "stale": stale, "fresh": fresh})
        passed = all(row["locked"] and row["other"] == "t" and row["other_ms"] < 1000 for row in rows)
        passed = passed and all(row["crash_exit"] == -signal.SIGKILL and row["connection_released"] for row in rows)
        passed = passed and all(row["released_after_crash"] or row["backend_terminated"] for row in rows)
        passed = passed and all(row["generation"] == 2 and not row["stale"] and row["fresh"] == "fresh" for row in rows)
        result = {"state": "PASS" if passed else "FAIL", "project": project, "attempts": rows}
    except (OSError, subprocess.TimeoutExpired) as error:
        result = {"state": "BLOCKED", "reason": f"Compose execution unavailable: {error}", "project": project}
    finally:
        if holder is not None and holder.poll() is None:
            os.killpg(holder.pid, signal.SIGKILL)
            holder.communicate(timeout=5)
        down = capture("compose-down", compose + ["down", "-v", "--remove-orphans"], timeout=120)
        remaining = capture("compose-ps-after-down", compose + ["ps", "-q"])
        live = [pid for pid in holder_pids if pid > 0 and _alive(pid)]
        services = remaining["out"].split()
        result["cleanup"] = {"holder_pids": holder_pids, "live_holder_pids": live,
                             "compose_services": services, "down_code": down["code"]}
        if live or services or down["code"] != 0:
            result["state"] = "FAIL"
    return result


def _alive(pid):
    try:
        os.kill(pid, 0)
        return True
    except ProcessLookupError:
        return False
