import re


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


def matching_open(text, close):
    if close < 0 or text[close] != ")":
        return -1
    depth = 0
    for index in range(close, -1, -1):
        if text[index] == ")":
            depth += 1
        elif text[index] == "(":
            depth -= 1
            if depth == 0:
                return index
    return -1


def matching_brace(text, opening):
    depth = 0
    for index in range(opening, len(text)):
        if text[index] == "{":
            depth += 1
        elif text[index] == "}":
            depth -= 1
            if depth == 0:
                return index
    return len(text)


def strip_wrappers(expression):
    value = expression.strip()
    changed = True
    while changed and value:
        changed = False
        if value.startswith("(") and matching_open(value, len(value) - 1) == 0:
            value = value[1:-1].strip()
            changed = True
            continue
        borrow = re.match(r"^(?:&\s*(?:mut\s+)?|\*)", value)
        if borrow:
            value = value[borrow.end():].strip()
            changed = True
    return value


def receiver_before(code, dot):
    end = dot
    while end and code[end - 1].isspace():
        end -= 1
    if not end:
        return None
    if code[end - 1] == ")":
        start = matching_open(code, end - 1)
        if start < 0:
            return None
        cursor = start - 1
        while cursor >= 0 and code[cursor].isspace():
            cursor -= 1
        if cursor >= 0 and (code[cursor].isalnum() or code[cursor] == "_"):
            while cursor >= 0 and (code[cursor].isalnum() or code[cursor] in "_.:"):
                cursor -= 1
            start = cursor + 1
        return code[start:end]
    start = end
    while start and (code[start - 1].isalnum() or code[start - 1] in "_.:"):
        start -= 1
    if start == end or (start and code[start - 1] == "."):
        return None
    return code[start:end]
