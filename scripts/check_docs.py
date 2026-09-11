"""Check repository Markdown with the Python standard library; no network access."""

from pathlib import Path
import re
import sys
from urllib.parse import unquote, urlsplit


ROOT = Path(__file__).resolve().parents[1]
EXCLUDED = {".git", ".venv", "target", "dist", "node_modules", "__pycache__"}
FENCE = re.compile(r"^\s{0,3}(`{3,}|~{3,})(.*)$")
LINK = re.compile(r"!?\[[^\]\n]*\]\(\s*(<[^>]+>|[^\s)]+)(?:\s+[\"'][^\n]*[\"'])?\s*\)")


def prose_lines(text):
    """Exclude fenced examples from link checks; report an unclosed fence."""
    opened = None
    lines = []
    for number, line in enumerate(text.splitlines(), 1):
        match = FENCE.match(line)
        if opened:
            if (match and match[1][0] == opened[0]
                    and len(match[1]) >= opened[1] and not match[2].strip()):
                opened = None
            continue
        if match:
            opened = (match[1][0], len(match[1]), number)
        else:
            lines.append((number, line))
    return lines, opened


def anchors(text):
    result = set()
    counts = {}
    for _, line in prose_lines(text)[0]:
        match = re.match(r"^\s{0,3}#{1,6}\s+(.+?)\s*#*\s*$", line)
        if not match:
            continue
        title = re.sub(r"<[^>]*>", "", match[1]).lower()
        title = re.sub(r"[^\w\- ]", "", title).replace(" ", "-")
        count = counts.get(title, 0)
        counts[title] = count + 1
        result.add(f"{title}-{count}" if count else title)
    return result


def main():
    errors = []
    documents = {}
    for required in ("README.md", "CONTRIBUTING.md", "LICENSE"):
        if not (ROOT / required).is_file():
            errors.append(f"Missing required file: {required}")
    for path in sorted(ROOT.rglob("*")):
        if not path.is_file() or path.suffix.lower() != ".md":
            continue
        relative = path.relative_to(ROOT)
        if EXCLUDED.intersection(relative.parts):
            continue
        try:
            content = path.read_text(encoding="utf-8")
        except UnicodeError as error:
            errors.append(f"{relative}: invalid UTF-8: {error}")
            continue
        documents[path.resolve()] = content
        if not content.strip():
            errors.append(f"{relative}: empty Markdown file")
        if not content.endswith("\n"):
            errors.append(f"{relative}: missing final newline")
        if "\x00" in content or "\ufffd" in content or "\ufeff" in content:
            errors.append(f"{relative}: unexpected NUL, replacement character, or BOM")
        if re.search(r"^(?:<{7}|>{7}|={7})\s*$|^(?:<{7}|>{7}) ", content, re.MULTILINE):
            errors.append(f"{relative}: possible unresolved merge marker")

    for path, content in documents.items():
        relative = path.relative_to(ROOT)
        lines, opened = prose_lines(content)
        if opened:
            errors.append(f"{relative}:{opened[2]}: unclosed code fence")
        for number, line in lines:
            for match in LINK.finditer(line):
                destination = match[1].strip("<>")
                parsed = urlsplit(destination)
                if parsed.scheme or parsed.netloc:
                    continue
                target = (path.parent / unquote(parsed.path)).resolve() if parsed.path else path
                try:
                    target.relative_to(ROOT)
                except ValueError:
                    errors.append(f"{relative}:{number}: link leaves repository: {destination}")
                    continue
                if not target.exists():
                    errors.append(f"{relative}:{number}: missing link target: {destination}")
                elif parsed.fragment and target in documents:
                    if unquote(parsed.fragment) not in anchors(documents[target]):
                        errors.append(f"{relative}:{number}: missing heading: {destination}")
    if errors:
        print("\n".join(errors), file=sys.stderr)
        return 1
    print(f"Checked {len(documents)} Markdown files: UTF-8, fences, merge markers, and local inline links are valid.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
