import re

EFFECT_METHODS = ("start", "stop", "status", "observe", "adopt", "logs", "delete", "shutdown")
METHOD_PATTERN = "|".join(EFFECT_METHODS)
DIRECT_EFFECT = re.compile(rf"\b(?P<receiver>[A-Za-z_]\w*)\s*\.\s*(?P<method>{METHOD_PATTERN})\s*\(")
RUNTIME_FACTORY_EFFECT = re.compile(
    rf"\b(?P<owner>[A-Za-z_]\w*)\s*\.\s*runtime\s*\(\s*\)\s*"
    rf"\.\s*(?P<method>{METHOD_PATTERN})\s*\("
)
RUNTIME_FIELD_EFFECT = re.compile(
    rf"\b(?P<owner>[A-Za-z_]\w*)\s*\.\s*runtime\s*\.\s*(?P<method>{METHOD_PATTERN})\s*\("
)
QUALIFIED_EFFECT = re.compile(
    rf"(?:\bRuntimeAdapter\s*(?:::|>\s*::)|\bas\s+RuntimeAdapter\s*>\s*::)"
    rf"\s*(?P<method>{METHOD_PATTERN})\s*\("
)
TYPE_BINDING = re.compile(
    r"\b(?P<name>[A-Za-z_]\w*)\s*(?<!:):(?!:)\s*[^,)=;\n]*\bRuntimeAdapter\b"
)
APP_STATE_BINDING = re.compile(
    r"\b(?P<name>[A-Za-z_]\w*)\s*(?<!:):(?!:)\s*[^,)=;\n]*\bAppState\b"
)
LET_ASSIGNMENT = re.compile(
    r"\blet\s+(?:mut\s+)?(?P<name>[A-Za-z_]\w*)"
    r"(?:\s*(?<!:):(?!:)\s*[^=;]+)?\s*=\s*(?![=>])(?P<expression>[^;]+);"
)
PLAIN_ASSIGNMENT = re.compile(
    r"(?<![\w.])(?P<name>[A-Za-z_]\w*)\s*=\s*(?![=>])(?P<expression>[^;]+);"
)


def rust_code(text):
    """Remove comments and literals so source discovery has no textual false positives."""
    output = []
    index = 0
    state = "code"
    while index < len(text):
        pair = text[index:index + 2]
        char = text[index]
        if state == "code" and pair == "//":
            state = "line"
            output.extend("  ")
            index += 2
        elif state == "code" and pair == "/*":
            state = "block"
            output.extend("  ")
            index += 2
        elif state == "line" and char == "\n":
            state = "code"
            output.append(char)
            index += 1
        elif state == "block" and pair == "*/":
            state = "code"
            output.extend("  ")
            index += 2
        elif state in {"line", "block"}:
            output.append("\n" if char == "\n" else " ")
            index += 1
        elif state == "code" and char == '"':
            state = "string"
            output.append(" ")
            index += 1
        elif state == "string" and char == "\\":
            output.extend("  ")
            index += 2
        elif state == "string" and char == '"':
            state = "code"
            output.append(" ")
            index += 1
        elif state == "string":
            output.append("\n" if char == "\n" else " ")
            index += 1
        else:
            output.append(char)
            index += 1
    return "".join(output)


def runtime_effect_calls(text):
    code = rust_code(text)
    aliases = {match.group("name") for match in TYPE_BINDING.finditer(code)}
    app_states = {match.group("name") for match in APP_STATE_BINDING.finditer(code)}
    app_states.add("state")
    if re.search(r"\bimpl\s+AppState\b", code):
        app_states.add("self")
    changed = True
    while changed:
        changed = False
        assignments = list(LET_ASSIGNMENT.finditer(code)) + list(PLAIN_ASSIGNMENT.finditer(code))
        for match in assignments:
            name = match.group("name")
            expression = match.group("expression")
            words = set(re.findall(r"\b[A-Za-z_]\w*\b", expression))
            from_factory = any(
                re.search(rf"\b{re.escape(owner)}\s*\.\s*runtime\s*\(\s*\)", expression)
                for owner in app_states
            )
            from_field = any(
                re.search(rf"\b{re.escape(owner)}\s*\.\s*runtime\b", expression)
                for owner in app_states
            )
            if from_factory or from_field or words.intersection(aliases):
                if name not in aliases:
                    aliases.add(name)
                    changed = True
    calls = [match.group("method") for match in QUALIFIED_EFFECT.finditer(code)]
    calls.extend(
        match.group("method") for match in RUNTIME_FACTORY_EFFECT.finditer(code)
        if match.group("owner") in app_states
    )
    calls.extend(
        match.group("method") for match in RUNTIME_FIELD_EFFECT.finditer(code)
        if match.group("owner") in app_states
    )
    calls.extend(
        match.group("method") for match in DIRECT_EFFECT.finditer(code)
        if match.group("receiver") in aliases
        and not re.search(r"\.\s*$", code[:match.start()])
    )
    return sorted(calls)
