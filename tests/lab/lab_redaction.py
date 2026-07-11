"""Secret-safe text redaction for retained laboratory artifacts."""
from __future__ import annotations

import json
import re
from typing import Any, Iterable

NAME = r"[A-Za-z0-9_.-]*(?:password|passwd|token|secret|authorization|api[-_]?key|credential|key)[A-Za-z0-9_.-]*"
AUTH = re.compile(r"(?im)(authorization\s*:\s*(?:bearer|basic)\s+)[^\r\n]*")
URI_USERINFO = re.compile(r"(?i)(\b[a-z][a-z0-9+.-]*:(?://)?)[^@]*@")
SENSITIVE_QUERY = re.compile(rf"(?i)([?&]\s*{NAME}\s*=\s*)[^&#\r\n]*")
URL_QUERY = re.compile(r"(?i)((?:\b[a-z][a-z0-9+.-]*:(?://)?|//)[^\r\n]*\?)[^#\r\n]*")
JSON_VALUE = re.compile(
    rf"(?i)([\"']?{NAME}[\"']?\s*:\s*)(?:\"(?:\\.|[^\"])*\"|'(?:\\.|[^'])*'|[^,\s}}\]]+)"
)
KEY_LINE = re.compile(rf"(?im)^(\s*[\"']?{NAME}[\"']?\s*[:=]\s*).*$")
SENSITIVE = re.compile(r"(?i)(password|passwd|token|secret|authorization|api[-_]?key|credential|key)")


def redact_text(text: str, secrets: Iterable[str]) -> str:
    """Redact full JSON documents and common log/config/header forms."""
    value = _redact_json_document(text)
    for secret in sorted((item for item in secrets if item), key=len, reverse=True):
        value = value.replace(secret, "<redacted>")
    value = URI_USERINFO.sub(r"\1<redacted>@", value)
    value = SENSITIVE_QUERY.sub(r"\1<redacted>", value)
    value = URL_QUERY.sub(r"\1<redacted>", value)
    value = AUTH.sub(r"\1<redacted>", value)
    value = JSON_VALUE.sub(r"\1<redacted>", value)
    return KEY_LINE.sub(r"\1<redacted>", value)


def _redact_json_document(text: str) -> str:
    try:
        return json.dumps(_redact_json(json.loads(text)), separators=(",", ":"))
    except json.JSONDecodeError:
        return text


def _redact_json(value: Any) -> Any:
    if isinstance(value, dict):
        return {
            key: "<redacted>" if SENSITIVE.search(str(key)) else _redact_json(item)
            for key, item in value.items()
        }
    if isinstance(value, list):
        return [_redact_json(item) for item in value]
    return value
