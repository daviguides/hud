"""Write spec/tasks/<id>.md from spec/tasks.json and the goldens (target embedded verbatim)."""

import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from common import BENCH  # noqa: E402
from tasks import SPEC, load_golden, screen_text  # noqa: E402

OUT = BENCH / "spec" / "tasks"

STREAM = {
    "pipe": "stdout is a pipe (not a terminal). Terminal width is NOT detectable: it comes only from the COLUMNS variable.",
    "tty": "stdout is a pseudo-terminal sized {cols} columns x {rows} rows.",
}


def attr_lines(rows):
    lines = []
    for y, row in enumerate(rows, 1):
        for s in row:
            marks = []
            if s["fg"] != "default":
                marks.append(f"fg {s['fg']}")
            if s["bg"] != "default":
                marks.append(f"bg {s['bg']}")
            for key, name in (("bold", "bold"), ("italic", "italic"), ("underline", "underline"),
                              ("strike", "strikethrough"), ("reverse", "reverse")):
                if s[key]:
                    marks.append(name)
            if marks:
                lines.append(f"- line {y}: `{s['text']}` : {', '.join(marks)}")
    return lines or ["- no cell carries any color or attribute"]


def render(task):
    out = [f"# {task['id']}: {task['title']}", "", "## Statement", "", task["statement"], ""]
    for run in task["runs"]:
        out += [f"## Run `{run['name']}`", ""]
        env = " ".join(f"{k}={v}" for k, v in sorted(run["env"].items()))
        out += [f"- Stream: {STREAM[run['stream']].format(cols=SPEC['terminal']['cols'], rows=SPEC['terminal']['rows'])}",
                f"- Environment: `{env}`"]
        if run["check"] == "bytes":
            out += ["- Check: stdout bytes are identical to the target.", ""]
            want = load_golden(task["id"], run).decode("utf-8")
            out += ["Target:", "", "```text", want.rstrip("\n"), "```", ""]
        else:
            rows = load_golden(task["id"], run)
            out += ["- Check: the final terminal screen equals the target cell by cell (character, foreground, background, bold, italic, underline, strikethrough, reverse). How the escape sequences are written does not matter.", ""]
            out += ["Target screen text:", "", "```text", screen_text(rows), "```", "", "Cells carrying color or attributes (every other cell is default):", ""]
            out += attr_lines(rows) + [""]
    return "\n".join(out)


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    for task in SPEC["tasks"]:
        (OUT / f"{task['id']}.md").write_text(render(task))
        print("spec", f"spec/tasks/{task['id']}.md")


if __name__ == "__main__":
    main()
