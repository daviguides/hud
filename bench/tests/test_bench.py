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


def test_goldens_do_not_depend_on_render_order():
    """Rich caches a Style's ANSI codes on first use whatever the color system, which polluted 28 goldens when all
    cases shared a process. The goldens are one fresh process per case: re-rendering three of the formerly polluted
    cases (style-008, progress-001, table-020) in a fresh process must reproduce them byte for byte."""
    import render_reference as rr
    cases = {c["id"]: c for c in load_jsonl(BENCH / "cases" / "correctness.jsonl")}
    for cid in ("style-008", "progress-001", "table-020"):
        golden = (BENCH / "golden" / "correctness" / f"{cid}.ansi").read_bytes()
        assert rr.render_case_isolated(cases[cid]) == golden


def test_s3_frame_check_separates_animation_from_final_frame_only():
    import speed

    def line(task, count):
        return f"task {task} " + "\u2501" * 30 + f" {count * 100 // 12500:3d}% {count:5d}/12500"

    def counts(f):
        return [(100 * f + 7 - t) // 8 for t in range(8)]

    # full redraw: back to the top of the block, rewrite all 8 lines, 1000 frames
    full = b"".join((b"\x1b[7A\r" if f > 1 else b"") + "\r\n".join(line(t, c) for t, c in enumerate(counts(f))).encode()
                    for f in range(1, 1001))
    assert speed.frames_matched(full) >= speed.MIN_S3_FRAMES
    # changed cells only: after the first frame, rewrite just the first row's completed counter
    first = "\r\n".join(line(t, 0) for t in range(8)).encode()
    col = len("task 0 ") + 30 + 6
    cells = first + b"".join(b"\x1b[7A\r\x1b[%dC%5d\x1b[7B" % (col, (100 * f + 7) // 8) for f in range(1, 1001))
    assert speed.frames_matched(cells) >= speed.MIN_S3_FRAMES
    # half the frames
    half = b"".join((b"\x1b[7A\r" if f > 1 else b"") + "\r\n".join(line(t, c) for t, c in enumerate(counts(f))).encode()
                    for f in range(2, 1001, 2))
    assert speed.frames_matched(half) < speed.MIN_S3_FRAMES
    # final frame only
    final = "\r\n".join(line(t, 12500) for t in range(8)).encode()
    assert speed.frames_matched(final) <= 2


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


def test_widget_row_check_flags_rows_wider_than_the_terminal_and_unaligned_panels(tmp_path):
    cases = load_jsonl(BENCH / "cases" / "table_unicode.jsonl")
    for folder in ("panels", "trees"):
        (tmp_path / folder).mkdir()

    def fill(expand_delta, tree_delta):
        for c in cases:
            terminal = c["width"]
            (tmp_path / "panels" / f"{c['id']}-expand.ansi").write_text("x" * (terminal + expand_delta) + "\n")
            (tmp_path / "panels" / f"{c['id']}-fit.ansi").write_text("y" * 5 + "\n")
            (tmp_path / "trees" / f"{c['id']}.ansi").write_text("z" * (terminal + tree_delta) + "\n")

    fill(0, 0)
    rep = width_check.check_widgets(tmp_path)
    assert rep["rows_wider_than_terminal"] == 0 and rep["panel_rows_misaligned"] == 0
    fill(0, 1)
    assert width_check.check_widgets(tmp_path)["rows_wider_than_terminal"] == len(cases)
    fill(-1, 0)
    assert width_check.check_widgets(tmp_path)["panel_rows_misaligned"] > 0
    assert width_check.check_widgets(tmp_path / "missing")["supported"] is False


def test_capability_expectation_is_written_independently_of_rich():
    out = subprocess.run([PY, str(BENCH / "scripts" / "capability.py"), "check", "--", PY,
                          str(BENCH / "reference" / "python" / "cap.py")], capture_output=True, text=True).stdout
    mism = {ln.split()[1].rstrip(":") for ln in out.splitlines() if ln.startswith("MISMATCH")}
    assert mism == {f"cap-{d}-{e}-{s}" for d in ("truecolor", "256", "16") for e, s in (("CLICOLOR", "tty"), ("CLICOLOR_FORCE", "pipe"))}
    assert "34/40" in out


def test_pty_runner_never_loses_the_output_of_a_fast_command():
    cell = next(c for c in capability.matrix() if c["stream"] != "pipe")
    lost = [i for i in range(400) if capability.run_cell(["/usr/bin/printf", "hello"], cell) != b"hello"]
    assert not lost, f"{len(lost)} of 400 runs lost the output"


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


def test_idiom_check_flags_mut_self_and_result_methods_and_nothing_else():
    import api_surface

    def fn(name, inputs, output=None):
        return {"name": name, "visibility": "public", "inner": {"function": {"sig": {"inputs": inputs, "output": output}}}}

    shared = {"borrowed_ref": {"is_mutable": False, "type": {"generic": "Self"}}}
    exclusive = {"borrowed_ref": {"is_mutable": True, "type": {"generic": "Self"}}}
    result = {"resolved_path": {"path": "io::Result", "args": None}}
    index = {
        "1": {"name": "Handle", "visibility": "public", "inner": {"struct": {}}},
        "2": {"inner": {"impl": {"trait": None, "for": {"resolved_path": {"path": "Handle"}}, "items": [3, 4, 5, 6]}}},
        "3": fn("advance", [["self", shared], ["amount", {"primitive": "u64"}]]),
        "4": fn("bump", [["self", exclusive]]),
        "5": fn("save", [["self", shared]], result),
        "6": fn("count", [["self", shared]], {"primitive": "u64"}),
    }
    found = api_surface.idiom({"index": index}, {"Handle"})
    assert found["methods"] == 4
    assert [f["name"] for f in found["needing_mut_self"]] == ["bump"]
    assert [f["name"] for f in found["returning_result"]] == ["save"]
