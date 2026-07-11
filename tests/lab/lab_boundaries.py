"""PostgreSQL and Gradle boundaries with their explicit safety policies."""
from __future__ import annotations

import os
import re
from pathlib import Path
import shutil
from urllib.parse import unquote, urlsplit

from lab_harness import Lab, ROOT
from lab_outcomes import Blocked, Skip


def postgres_real(lab: Lab) -> None:
    url = os.environ.get("LKJMC_LAB_POSTGRES_URL")
    if not url:
        raise Skip("LKJMC_LAB_POSTGRES_URL is unset")
    if not disposable_postgres_url(url):
        raise Blocked("PostgreSQL target is not a disposable loopback laboratory database")
    if os.environ.get("LKJMC_LAB_POSTGRES_DISPOSABLE") != "1":
        raise Blocked("explicit PostgreSQL target requires LKJMC_LAB_POSTGRES_DISPOSABLE=1")
    if shutil.which("psql") is None:
        raise Skip("psql is unavailable")
    lab.secrets.add(url)
    try:
        _psql(lab, "postgres-create", f"CREATE SCHEMA {lab.schema}")
        query = f"CREATE TABLE {lab.schema}.probe (value integer); INSERT INTO {lab.schema}.probe VALUES (1); SELECT value FROM {lab.schema}.probe"
        if _psql(lab, "postgres-query", query).strip() != "1":
            raise Blocked("PostgreSQL did not return the inserted value")
    finally:
        if _psql(lab, "postgres-drop", f"DROP SCHEMA IF EXISTS {lab.schema} CASCADE", True) is None:
            raise Blocked("PostgreSQL schema cleanup failed")


def disposable_postgres_url(value: str) -> bool:
    try:
        parsed = urlsplit(value)
        database, port = unquote(parsed.path[1:]), parsed.port
    except ValueError:
        return False
    return (
        parsed.scheme in {"postgres", "postgresql"}
        and parsed.hostname in {"localhost", "127.0.0.1", "::1"}
        and parsed.path == f"/{database}"
        and bool(re.fullmatch(r"lkjmc_lab_[a-z0-9_]+", database))
        and not parsed.query and not parsed.fragment and (port is None or 1 <= port <= 65535)
    )


def _psql(lab: Lab, label: str, query: str, allow_failure: bool = False) -> str | None:
    url = os.environ["LKJMC_LAB_POSTGRES_URL"]
    command = ["psql", "--no-psqlrc", "--quiet", "--tuples-only", "--no-align", "-v", "ON_ERROR_STOP=1", "-d", url, "-c", query]
    code, output = lab.run(label, command, 30)
    if code and not allow_failure:
        raise Blocked(f"{label} failed")
    return output if code == 0 else None

