"""Lines-of-code rule for the DX metric.

Format with the language's standard formatter (rustfmt --edition 2024 for .rs, ruff format for .py),
then count lines that are neither blank nor comment-only. `use`/`import` lines, the entry-point
wrapper and data literals all count: the metric is "how much code did the task take".
"""

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from common import BENCH  # noqa: E402


def fmt(path: Path) -> str:
    src = path.read_text()
    if path.suffix == ".rs":
        r = subprocess.run(["rustfmt", "--edition", "2024", "--emit", "stdout"], input=src, text=True,
                           capture_output=True)
        text = r.stdout
        # rustfmt --emit stdout prefixes the file name; drop that header line
        return "\n".join(text.split("\n")[2:]) if text.startswith("stdin") else text
    if path.suffix == ".py":
        r = subprocess.run(["uvx", "ruff", "format", "--stdin-filename", path.name, "-"], input=src, text=True,
                           capture_output=True)
        return r.stdout or src
    raise SystemExit(f"unsupported extension: {path.suffix}")


def count(path: Path) -> int:
    text = fmt(path)
    text = re.sub(r"/\*.*?\*/", "", text, flags=re.S)
    n = 0
    for line in text.splitlines():
        s = line.strip()
        if not s or s.startswith(("//", "#")):
            continue
        n += 1
    return n


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("files", nargs="*", type=Path)
    ap.add_argument("--python-baseline", action="store_true", help="write golden/tasks/python_loc.json")
    args = ap.parse_args()
    if args.python_baseline:
        files = sorted((BENCH / "reference" / "python" / "tasks").glob("t*.py"))
        data = {f.stem.split("_")[0]: count(f) for f in files}
        out = BENCH / "golden" / "tasks" / "python_loc.json"
        out.write_text(json.dumps(data, indent=1, sort_keys=True) + "\n")
        print(data)
        return
    for f in args.files:
        print(f"{count(f):4d} {f}")


if __name__ == "__main__":
    main()
