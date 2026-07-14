import re

from runtime_adoption_syntax import matching_brace, receiver_before, rust_code, strip_wrappers

EFFECT_METHODS = ("start", "stop", "status", "observe", "adopt", "logs", "delete", "shutdown")
METHOD_PATTERN = "|".join(EFFECT_METHODS)
EFFECT_CALL = re.compile(rf"\.\s*(?P<method>{METHOD_PATTERN})\s*\(")
QUALIFIED_EFFECT = re.compile(
    rf"(?:\bRuntimeAdapter\s*(?:::|>\s*::)|\bas\s+RuntimeAdapter\s*>\s*::)"
    rf"\s*(?P<method>{METHOD_PATTERN})\s*\("
)
GENERIC_CONSTRAINT = re.compile(
    r"(?:[<,]\s*|\bwhere\s+)(?P<name>[A-Z]\w*)\s*:\s*"
    r"(?P<bounds>[^,>{};\n]*\bRuntimeAdapter\b)"
)
TYPED_BINDING = re.compile(
    r"\b(?P<name>[A-Za-z_]\w*)\s*(?<!:):(?!:)\s*(?P<type>[^,)=;\n]+)"
)
LET_ASSIGNMENT = re.compile(
    r"\blet\s+(?:mut\s+)?(?P<name>[A-Za-z_]\w*)"
    r"(?:\s*(?<!:):(?!:)\s*[^=;]+)?\s*=\s*(?![=>])(?P<expression>[^;]+);"
)
PLAIN_ASSIGNMENT = re.compile(
    r"(?<![\w.])(?P<name>[A-Za-z_]\w*)\s*=\s*(?![=>])(?P<expression>[^;]+);"
)
DECLARATION = re.compile(r"\b(?:fn|impl|trait|struct|enum)\b")


def identity_source(expression, aliases, app_states):
    value = strip_wrappers(expression)
    if value in aliases:
        return True
    clone = re.fullmatch(r"(.+)\.clone\s*\(\s*\)", value)
    if clone:
        return identity_source(clone.group(1), aliases, app_states)
    clone = re.fullmatch(r"(?:(?:std\s*::\s*sync\s*::\s*)?Arc|Clone)\s*::\s*clone\s*\((.*)\)", value)
    if clone:
        return identity_source(clone.group(1), aliases, app_states)
    field = re.fullmatch(r"([A-Za-z_]\w*)\s*\.\s*runtime", value)
    factory = re.fullmatch(r"([A-Za-z_]\w*)\s*\.\s*runtime\s*\(\s*\)", value)
    return bool(
        (field and field.group(1) in app_states)
        or (factory and factory.group(1) in app_states)
    )


def generic_regions(code):
    declarations = list(DECLARATION.finditer(code))
    regions = {}
    constraints = list(GENERIC_CONSTRAINT.finditer(code))
    for constraint in constraints:
        owners = [item for item in declarations if item.start() < constraint.start()]
        opening = code.find("{", constraint.end())
        semicolon = code.find(";", constraint.end())
        if not owners or opening < 0 or (semicolon >= 0 and semicolon < opening):
            continue
        region = (owners[-1].start(), matching_brace(code, opening))
        regions.setdefault(constraint.group("name"), []).append(region)
    return constraints, regions


def constrained_binding(type_words, position, regions):
    return any(
        start <= position <= end
        for word in type_words
        for start, end in regions.get(word, [])
    )


def runtime_effect_calls(text):
    code = rust_code(text)
    constraints, regions = generic_regions(code)
    generic_types = {match.group("name") for match in constraints}
    binding_code = GENERIC_CONSTRAINT.sub(
        lambda match: re.sub(r"[^\n]", " ", match.group(0)), code
    )
    aliases = set()
    app_states = {"state"}
    for match in TYPED_BINDING.finditer(binding_code):
        name = match.group("name")
        type_words = set(re.findall(r"\b[A-Za-z_]\w*\b", match.group("type")))
        if name not in generic_types and (
            "RuntimeAdapter" in type_words
            or constrained_binding(type_words, match.start(), regions)
        ):
            aliases.add(name)
        if "AppState" in type_words:
            app_states.add(name)
    if re.search(r"\bimpl\s+AppState\b", code):
        app_states.add("self")
    assignments = list(LET_ASSIGNMENT.finditer(code)) + list(PLAIN_ASSIGNMENT.finditer(code))
    changed = True
    while changed:
        changed = False
        for match in assignments:
            name = match.group("name")
            if name not in aliases and identity_source(match.group("expression"), aliases, app_states):
                aliases.add(name)
                changed = True
    calls = [match.group("method") for match in QUALIFIED_EFFECT.finditer(code)]
    for match in EFFECT_CALL.finditer(code):
        receiver = receiver_before(code, match.start())
        if receiver and identity_source(receiver, aliases, app_states):
            calls.append(match.group("method"))
    return sorted(calls)
