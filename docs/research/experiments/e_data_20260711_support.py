"""Effects owned by the bounded E-DATA research harness."""
import hashlib
import json
import socket
import subprocess
import time
import uuid
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
SQL = Path(__file__).with_name("e-data-20260711.sql")
LIMIT = 8192


def digest(value):
    return hashlib.sha256(value).hexdigest()


def record(result, raw, label, command, code, output, classification="observed"):
    data = output.encode() if isinstance(output, str) else output
    data = data[:LIMIT]
    path = raw / f"{label}.log"
    path.write_bytes(data)
    result["commands"].append({"label": label, "exit": code, "log": path.name,
        "sha256": digest(data), "command": command, "classification": classification})
    return code, data.decode("utf-8", "replace")


def invoke(command, timeout=240, **kwargs):
    try:
        done = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
            text=True, check=False, timeout=timeout, **kwargs)
        return done.returncode, done.stdout
    except (OSError, subprocess.TimeoutExpired) as error:
        return 127, str(error)


def logged(result, raw, label, command, classification="observed", **kwargs):
    code, output = invoke(command, **kwargs)
    return record(result, raw, label, command, code, output, classification)


def compose(base, *args):
    return ["docker", "compose", "--project-name", base[0], "-f", str(base[1]), *args]


def psql(base, sql):
    return compose(base, "exec", "-T", "postgres", "psql", "-X", "-U", "lkjmc",
        "-d", "lkjmc", "-v", "ON_ERROR_STOP=1", "-At"), sql


def query(result, raw, base, label, sql):
    command, input_text = psql(base, sql)
    return logged(result, raw, label, command, input=input_text)


def probe(result, name, code, output, marker):
    result["probes"][name] = "PASS" if code == 0 and marker in output else "FAIL"


def free_port():
    with socket.socket() as listener:
        listener.bind(("127.0.0.1", 0))
        return listener.getsockname()[1]


def wait_postgres(base):
    command = compose(base, "exec", "-T", "postgres", "pg_isready", "-U", "lkjmc", "-d", "lkjmc")
    for _ in range(60):
        code, output = invoke(command, timeout=10)
        if code == 0:
            return code, output
        time.sleep(1)
    return code, output


def wait_lock(base, sql):
    command, input_text = psql(base, sql)
    for _ in range(80):
        code, output = invoke(command, input=input_text)
        if code == 0 and output.strip() == "t":
            return True
        time.sleep(.1)
    return False


def start_sql(base, sql):
    command, _ = psql(base, sql)
    command = [*command, "-c", sql]
    process = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
        text=True)
    return process, command


def finish_sql(result, raw, label, process, command, classification="observed"):
    output, _ = process.communicate(timeout=30)
    return record(result, raw, label, command, process.returncode, output, classification)


def cli(result, raw, label, env, *args, classification="observed"):
    return logged(result, raw, label, [str(ROOT / "target/debug/lkjmc"), *args],
        cwd=ROOT, env=env, timeout=1800, classification=classification)


def start_daemon(result, raw, env):
    path = raw / "daemon.sock"
    command = [str(ROOT / "target/debug/lkjmc-daemon"), "--socket", str(path), "--http", "none",
        "--database-url", env["LKJMC_DATABASE_URL"], "--config-root", str(raw / "config"),
        "--log-root", str(raw / "logs"), "--jar-root", str(raw / "jars"), "--data-root", str(raw / "data")]
    process = subprocess.Popen(command, cwd=ROOT, env=env, stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT, text=True)
    for _ in range(100):
        if path.exists():
            return process, path, command
        if process.poll() is not None:
            break
        time.sleep(.1)
    output, _ = process.communicate(timeout=5)
    record(result, raw, "daemon-start", command, process.returncode, output, "daemon-start-failed")
    raise RuntimeError("daemon socket was unavailable")


def stop_daemon(result, raw, process, command):
    process.terminate()
    try:
        output, _ = process.communicate(timeout=30)
    except subprocess.TimeoutExpired:
        process.kill()
        output, _ = process.communicate(timeout=10)
    record(result, raw, "daemon-stop", command, process.returncode, output, "expected-termination")


def socket_command(result, raw, label, path, command, body):
    payload = json.dumps({"requestId": str(uuid.uuid4()), "actor": {"kind": "cli", "name": "e-data"},
        "command": command, "body": body}, separators=(",", ":")).encode()
    request = b"POST /command HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: " + str(len(payload)).encode() + b"\r\n\r\n" + payload
    try:
        with socket.socket(socket.AF_UNIX) as client:
            client.settimeout(30)
            client.connect(str(path))
            client.sendall(request)
            output = b""
            while chunk := client.recv(8192):
                output += chunk
        return record(result, raw, label, ["unix-http", command], 0, output)
    except OSError as error:
        return record(result, raw, label, ["unix-http", command], 127, str(error))


def candidate_sql(schema):
    return SQL.read_text(encoding="utf-8").replace("__SCHEMA__", schema)


def archive(result, raw, base, schema):
    command = compose(base, "exec", "-T", "postgres", "pg_dump", "-U", "lkjmc", "-d", "lkjmc", "-Fc")
    done = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False, timeout=240)
    (raw / "cutover.dump").write_bytes(done.stdout)
    outcome = record(result, raw, "dump-private-cutover", command, done.returncode, done.stderr,
        "public-and-private-schema-archive")
    result["commands"][-1]["archive"] = "cutover.dump"
    result["commands"][-1]["archiveSha256"] = digest(done.stdout)
    return outcome


def restore(result, raw, base):
    command = compose(base, "exec", "-T", "postgres", "pg_restore", "-U", "lkjmc", "-d", "lkjmc")
    done = subprocess.run(command, input=(raw / "cutover.dump").read_bytes(), stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT, check=False, timeout=240)
    return record(result, raw, "restore-private-cutover", command, done.returncode, done.stdout)
