import re

RAW_STRING = re.compile(r'(?:br|rb|cr|r)(?P<hashes>#{0,255})"')
CHAR_LITERAL = re.compile(r"(?:b)?'(?:\\(?:.|u\{[0-9A-Fa-f_]+\}|x[0-9A-Fa-f]{2})|[^\\'\n])'")


def blank(text):
    return "".join("\n" if char == "\n" else " " for char in text)


def quoted_end(text, opening, quote):
    index = opening + 1
    while index < len(text):
        if text[index] == "\\":
            index += 2
        elif text[index] == quote:
            return index + 1
        else:
            index += 1
    return None


def block_comment_end(text, opening):
    index = opening + 2
    depth = 1
    while index < len(text) and depth:
        if text.startswith("/*", index):
            depth += 1
            index += 2
        elif text.startswith("*/", index):
            depth -= 1
            index += 2
        else:
            index += 1
    return index


def rust_code(text):
    """Blank Rust comments and string/character literals while preserving positions."""
    output = []
    index = 0
    while index < len(text):
        if text.startswith("//", index):
            end = text.find("\n", index)
            end = len(text) if end < 0 else end
        elif text.startswith("/*", index):
            end = block_comment_end(text, index)
        else:
            raw = RAW_STRING.match(text, index)
            if raw:
                marker = '"' + raw.group("hashes")
                closing = text.find(marker, raw.end())
                end = len(text) if closing < 0 else closing + len(marker)
            else:
                prefix = 0
                if text.startswith(('b"', 'c"'), index):
                    prefix = 1
                if text[index + prefix:index + prefix + 1] == '"':
                    end = quoted_end(text, index + prefix, '"') or len(text)
                else:
                    character = CHAR_LITERAL.match(text, index)
                    if character:
                        end = character.end()
                    else:
                        output.append(text[index])
                        index += 1
                        continue
        output.append(blank(text[index:end]))
        index = end
    return "".join(output)
