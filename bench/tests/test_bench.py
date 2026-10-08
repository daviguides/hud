"""Harness self-tests (evaluation.md, pilot validity rule 4): the harness must be right before any candidate is scored."""

import json
import shutil
import subprocess
import sys
from pathlib import Path

import pytest

import capability
import compare
import tasks
import width_check
from common import BENCH, load_jsonl, rich_width, wcwidth_width

PY = str(BENCH / ".venv" / "bin" / "python")


def test_corpus_minimums():
    cases = load_jsonl(BENCH / "cases" / "correctness.jsonl")
    assert len(cases) >= 200
    per = {}
    for c in cases:
        per[c["feature"]] = per.get(c["feature"], 0) + 1
    assert set(per) == {"style", "markup", "table", "panel", "tree", "progress", "error"}
    assert min(per.values()) >= 10
    assert len(load_jsonl(BENCH / "cases" / "width_corpus.jsonl")) >= 500
    assert len(capability.matrix()) == 40
    for c in cases:
        assert c["color_system"] in ("truecolor", "256", "standard", "none")
    for f in per:
        assert {c["color_system"] for c in cases if c["feature"] == f} == {"truecolor", "256", "standard", "none"}


def test_generation_is_deterministic(tmp_path):
    for script in ("gen_cases.py", "gen_width_corpus.py"):
        before = {p: p.read_bytes() for p in (BENCH / "cases").glob("*.jsonl")}
        subprocess.run([PY, str(BENCH / "scripts" / script)], check=True, capture_output=True)
        after = {p: p.read_bytes() for p in (BENCH / "cases").glob("*.jsonl")}
        assert before == after


def test_rich_scores_100_percent_on_its_own_corpus(tmp_path):
    subprocess.run([PY, str(BENCH / "scripts" / "render_reference.py"), str(BENCH / "cases" / "correctness.jsonl"),
                    str(tmp_path)], check=True, capture_output=True)
    per, failures = compare.compare(tmp_path, BENCH / "cases" / "correctness.jsonl", BENCH / "golden" / "correctness")
    rows, overall = compare.summarize(per)
    assert overall["rate"] == 1.0 and overall["style_markup_100"] and not failures


def test_compare_detects_difference_and_never_counts_unsupported_as_pass(tmp_path):
    shutil.copytree(BENCH / "golden" / "correctness", tmp_path / "c")
    cand = tmp_path / "c"
    (cand / "table-000.ansi").write_bytes((cand / "table-000.ansi").read_bytes() + b"x")
    (cand / "tree-000.ansi").unlink()
    (cand / "tree-000.unsupported").write_bytes(b"")
    per, failures = compare.compare(cand, BENCH / "cases" / "correctness.jsonl", BENCH / "golden" / "correctness")
    assert per["table"]["fail"] == 1 and per["tree"]["unsupported"] == 1
    rows, overall = compare.summarize(per)
    assert overall["pass"] == 208 and overall["denominator"] == 210
    assert failures[0]["id"] == "table-000"


def test_compare_excludes_classified_deviations(tmp_path):
    shutil.copytree(BENCH / "golden" / "correctness", tmp_path / "c")
    cand = tmp_path / "c"
    (cand / "panel-001.ansi").write_bytes(b"different")
    (cand / "classifications.json").write_text(json.dumps({"panel-001": {"class": "documented_deviation", "reason": "x"}}))
    per, _ = compare.compare(cand, BENCH / "cases" / "correctness.jsonl", BENCH / "golden" / "correctness")
    rows, overall = compare.summarize(per)
    assert per["panel"]["excluded_documented_deviation"] == 1 and overall["denominator"] == 209 and overall["rate"] == 1.0


def test_width_reference_has_no_unresolved_and_resolutions_are_hand_listed():
    ref = load_jsonl(BENCH / "golden" / "width" / "width_ref.jsonl")
    assert len(ref) == 500
    resolved = [r for r in ref if r["status"] == "resolved"]
    assert len(resolved) == 13
    for r in ref:
        if r["status"] == "agree":
            assert r["rich"] == r["wcwidth"] == r["ref"]
    texts = {r["id"]: r["text"] for r in load_jsonl(BENCH / "cases" / "width_corpus.jsonl")}
    for r in resolved:
        assert rich_width(texts[r["id"]]) == r["rich"] != wcwidth_width(texts[r["id"]])


def test_width_checker_passes_reference_and_flags_a_naive_candidate(tmp_path):
    good = tmp_path / "good"
    width_check.write_reference(good)
    rep = width_check.run(good)
    assert rep["width"]["rate"] == 1.0 and rep["fold"]["grapheme_splits"] == 0
    assert rep["tables"]["misaligned_rows"] == 0
    bad = tmp_path / "bad"
    width_check.write_reference(bad)
    items = load_jsonl(BENCH / "cases" / "width_corpus.jsonl")
    with (bad / "width.jsonl").open("w") as f:  # naive candidate: one cell per code point
        for it in items:
            f.write(json.dumps({"id": it["id"], "width": len(it["text"])}) + "\n")
    with (bad / "fold.jsonl").open("w") as f:  # splits at code points
        for it in items:
            for w in range(1, 41):
                t = it["text"]
                f.write(json.dumps({"id": it["id"], "w": w, "lines": [t[i:i + w] for i in range(0, len(t), w)] or [""]},
                                   ensure_ascii=False) + "\n")
    rep = width_check.run(bad)
    assert rep["width"]["rate"] < 0.99
    assert rep["fold"]["grapheme_splits"] > 0


def test_capability_expectation_is_written_independently_of_rich():
    out = subprocess.run([PY, str(BENCH / "scripts" / "capability.py"), "check", "--", PY,
                          str(BENCH / "reference" / "python" / "cap.py")], capture_output=True, text=True).stdout
    mism = {ln.split()[1].rstrip(":") for ln in out.splitlines() if ln.startswith("MISMATCH")}
    assert mism == {f"cap-{d}-{e}-{s}" for d in ("truecolor", "256", "16") for e, s in (("CLICOLOR", "tty"), ("CLICOLOR_FORCE", "pipe"))}
    assert "34/40" in out


def test_capability_classifier():
    assert capability.classify(b"\x1b[1;38;2;255;136;0mx\x1b[0m\n") == {"escapes": True, "bold": True, "color_class": "truecolor"}
    assert capability.classify(b"\x1b[1m\x1b[38;5;208mx\x1b[0m") == {"escapes": True, "bold": True, "color_class": "256"}
    assert capability.classify(b"\x1b[1;91mx\x1b[0m")["color_class"] == "standard"
    assert capability.classify(b"x\n") == {"escapes": False, "bold": False, "color_class": "none"}


def test_dx_python_references_pass_their_own_goldens():
    for task in tasks.SPEC["tasks"]:
        for run in task["runs"]:
            r = tasks.check_run(task, run, [PY, str(BENCH / task["ref"])])
            assert r["ok"], (task["id"], run["name"])


def test_screen_check_ignores_sgr_encoding_but_not_what_the_user_sees():
    a = tasks.screen_snapshot(b"\x1b[1;31mhi\x1b[0m there\n")
    b = tasks.screen_snapshot(b"\x1b[1m\x1b[31mhi\x1b[0m there\n")
    c = tasks.screen_snapshot(b"\x1b[31mhi\x1b[0m there\n")
    assert a == b and a != c


def test_dx_task_fails_on_wrong_output():
    task = tasks.find_task("t04-tree")
    r = tasks.check_run(task, task["runs"][0], ["printf", "hud/\\n"])
    assert not r["ok"]


def test_speed_goldens_verify_for_rich_and_ratio_reproduces_within_10_percent(tmp_path):
    import speed
    ok, *_ = speed.verify("S1", [PY, str(BENCH / "reference/python/speed/s1.py")])
    assert ok
    bench = [PY, str(BENCH / "reference/python/speed/bench.py")]
    for w in ("S3", "S4"):
        ok, *_ = speed.verify(w, bench)
        assert ok
    a = speed.time_inproc(bench, "S4", iterations=8, warmup=2)
    b = speed.time_inproc(bench, "S4", iterations=8, warmup=2)
    r, lo, hi = speed.ratio_ci(a, b)
    assert 0.9 <= r <= 1.1


def test_loc_rule(tmp_path):
    import loc
    p = tmp_path / "x.rs"
    p.write_text("// c\nuse std::io;\n\n/* block */\nfn main() {\n    println!(\"hi\"); // trailing\n}\n")
    assert loc.count(p) == 4
