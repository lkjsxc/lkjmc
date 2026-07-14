import re

from runtime_adoption_syntax import rust_code

RESERVED_EFFECT_METHODS = (
    "runtime_start",
    "runtime_stop",
    "runtime_status",
    "runtime_observe",
    "runtime_adopt",
    "runtime_logs",
    "runtime_delete",
    "runtime_shutdown",
)
METHOD_PATTERN = "|".join(RESERVED_EFFECT_METHODS)
EFFECT_CALL = re.compile(rf"(?:\.|::)\s*(?:r#)?(?P<method>{METHOD_PATTERN})\s*\(")


def runtime_effect_calls(text):
    """Return every reserved runtime method call after removing non-code text."""
    code = rust_code(text)
    return sorted(match.group("method") for match in EFFECT_CALL.finditer(code))
