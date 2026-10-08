"""Criteria S1 to S4 of v0.8 (foundation/structured-output-criteria.md) for hud, from files and nothing else.

  structured.py [--out DIR]    runs the adapters, checks the documents, writes pilot/hud/structured.{json,txt}

S1  every JSON document (corpus 4, 700 cases, nested) validates against spec/structured-json.schema.json with an
    independent validator, and equals the document built in Python from the case (gen_structured_cases.py)
S2  the same value rendered 50 times on 5 widths (in-process, report.json of the runner) and in two processes
    gives the same bytes
S3  every document equals json.dumps(json.loads(doc), indent=2, ensure_ascii=False) + newline; the library's
    own reader writes every document back identically (report.json)
S4  plain output has no ESC and equals the rich output with its escape sequences removed: corpus 4, corpus 1
    (210), table_unicode (12) and the Columns/Layout cases of corpus 3 (60; the Live and progress_live cases are
    redraw streams, not renders, see changelog 48)
"""

import argparse
import json
import os
import re
import subprocess
import sys
import tempfile
from pathlib import Path

import jsonschema

sys.path.insert(0, str(Path(__file__).parent))
from common import BENCH  # noqa: E402

BIN = BENCH / "candidates" / "hud" / "target" / "release"
SCHEMA = json.loads((BENCH / "spec" / "structured-json.schema.json").read_text())
ANSI = re.compile(r"\x1b\[[0-9;?]*[ -/]*[@-~]|\x1b\]8;[^\x1b]*\x1b\\")


def run(cmd, env_extra=None):
    env = {**os.environ, **(env_extra or {})}
    env.pop("HUD_FORMAT", None)
    env.update(env_extra or {})
    subprocess.run(cmd, check=True, env=env, stdout=subprocess.DEVNULL)


def read(path):
    return Path(path).read_bytes().decode("utf-8")


def corpus4(tmp):
    cases = BENCH / "cases" / "structured.jsonl"
    a, b = Path(tmp) / "a", Path(tmp) / "b"
    run([str(BIN / "structured_runner"), str(cases), str(a)])
    run([str(BIN / "structured_runner"), str(cases), str(b)])
    expected = {json.loads(l)["id"]: json.loads(l)["json"]
                for l in (BENCH / "golden" / "structured" / "expected.jsonl").read_text().splitlines() if l}
    ids = sorted(expected)
    validator = jsonschema.Draft202012Validator(SCHEMA)
    out = {"cases": len(ids), "s1_valid": 0, "s1_equals_oracle": 0, "s3_canonical": 0, "s2_process_equal": 0,
           "s4_plain_no_esc": 0, "s4_plain_equals_stripped_rich": 0}
    failures = []
    for cid in ids:
        text = read(a / f"{cid}.json")
        try:
            validator.validate(json.loads(text))
            out["s1_valid"] += 1
        except Exception as error:  # noqa: BLE001
            failures.append((cid, "schema", str(error)[:200]))
            text = text if text else ""
        if text == expected[cid]:
            out["s1_equals_oracle"] += 1
        else:
            failures.append((cid, "oracle", ""))
        try:
            canonical = text == json.dumps(json.loads(text), indent=2, ensure_ascii=False) + "\n"
        except ValueError:
            canonical = False
        if canonical:
            out["s3_canonical"] += 1
        else:
            failures.append((cid, "canonical", ""))
        if text == read(b / f"{cid}.json") and read(a / f"{cid}.plain") == read(b / f"{cid}.plain"):
            out["s2_process_equal"] += 1
        plain, rich = read(a / f"{cid}.plain"), read(a / f"{cid}.rich")
        if "\x1b" not in plain:
            out["s4_plain_no_esc"] += 1
        if ANSI.sub("", rich) == plain:
            out["s4_plain_equals_stripped_rich"] += 1
        else:
            failures.append((cid, "plain-vs-rich", ""))
    out["runner_report"] = json.loads((a / "report.json").read_text())
    out["failures"] = failures[:20]
    out["failure_count"] = len(failures)
    return out


def existing(tmp, name, runner, cases, extra, features=None):
    """Corpus 1, table_unicode and corpus 3: rich with the default environment, plain with HUD_FORMAT=plain."""
    rich_dir, plain_dir = Path(tmp) / f"{name}-rich", Path(tmp) / f"{name}-plain"
    run([str(BIN / runner), str(cases), str(rich_dir), *extra])
    run([str(BIN / runner), str(cases), str(plain_dir), *extra], {"HUD_FORMAT": "plain"})
    total = no_esc = equal = 0
    bad = []
    for line in cases.read_text().splitlines():
        if not line.strip():
            continue
        case = json.loads(line)
        if features and case["feature"] not in features:
            continue
        cid = case["id"]
        rich_p, plain_p = rich_dir / f"{cid}.ansi", plain_dir / f"{cid}.ansi"
        if not rich_p.exists() or not plain_p.exists():
            continue
        total += 1
        plain, rich = read(plain_p), read(rich_p)
        no_esc += "\x1b" not in plain
        if ANSI.sub("", rich) == plain:
            equal += 1
        else:
            bad.append(cid)
    return {"cases": total, "no_esc": no_esc, "equals_stripped_rich": equal, "failures": bad[:20]}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", default=str(BENCH / "pilot" / "hud"))
    args = ap.parse_args()
    with tempfile.TemporaryDirectory(prefix="hud-structured-") as tmp:
        result = {"corpus4": corpus4(tmp)}
        result["corpus1"] = existing(tmp, "c1", "cases_runner", BENCH / "cases" / "correctness.jsonl", [])
        result["table_unicode"] = existing(tmp, "tu", "cases_runner", BENCH / "cases" / "table_unicode.jsonl", [])
        result["corpus3_columns_layout"] = existing(
            tmp, "c3", "live_layout_runner", BENCH / "cases" / "live_layout.jsonl", [], {"columns", "layout"})
    out = Path(args.out)
    out.mkdir(parents=True, exist_ok=True)
    (out / "structured.json").write_text(json.dumps(result, indent=1, ensure_ascii=False) + "\n")
    c4 = result["corpus4"]
    n = c4["cases"]
    s4_total = sum(result[k]["cases"] for k in ("corpus1", "table_unicode", "corpus3_columns_layout")) + n
    s4_ok = (sum(result[k]["equals_stripped_rich"] and min(result[k]["no_esc"], result[k]["equals_stripped_rich"])
                 for k in ("corpus1", "table_unicode", "corpus3_columns_layout")) + min(
        c4["s4_plain_no_esc"], c4["s4_plain_equals_stripped_rich"]))
    rep = c4["runner_report"]
    lines = [
        f"S1 valid under the schema: {c4['s1_valid']}/{n}; equal to the Python oracle: {c4['s1_equals_oracle']}/{n}",
        f"S2 in-process: {rep['determinism_mismatches']} mismatches over {rep['json_renders']} renders "
        f"({len(rep['widths'])} widths x {rep['repeats']} repeats); two processes equal: {c4['s2_process_equal']}/{n}",
        f"S3 canonical: {c4['s3_canonical']}/{n}; library round trip failures: {rep['roundtrip_failures']}",
        f"S4 plain vs stripped rich: corpus4 {c4['s4_plain_equals_stripped_rich']}/{n} (no ESC {c4['s4_plain_no_esc']}/{n}); "
        f"corpus1 {result['corpus1']['equals_stripped_rich']}/{result['corpus1']['cases']}; "
        f"table_unicode {result['table_unicode']['equals_stripped_rich']}/{result['table_unicode']['cases']}; "
        f"corpus3 columns+layout {result['corpus3_columns_layout']['equals_stripped_rich']}/{result['corpus3_columns_layout']['cases']}",
        f"S4 total: {s4_ok}/{s4_total}",
        f"unsupported cases: {len(rep['unsupported'])}; failures listed: {c4['failure_count']}",
    ]
    (out / "structured.txt").write_text("\n".join(lines) + "\n")
    print("\n".join(lines))
    for f in c4["failures"][:10]:
        print("  FAIL", f)


if __name__ == "__main__":
    main()
