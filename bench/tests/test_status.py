"""Self-tests of corpus 5 (Spinner, Status): the generated spinner table equals Rich, the corpus meets its
pre-registered minimums, and its goldens are reproducible (foundation/status-criteria.md)."""

import json
import subprocess

from rich._spinners import SPINNERS

from common import BENCH, load_jsonl

PY = str(BENCH / ".venv" / "bin" / "python")


def test_the_generated_spinner_table_equals_rich():
    out = subprocess.run([PY, str(BENCH / "scripts" / "gen_spinners.py"), "--check"], capture_output=True, text=True)
    assert out.returncode == 0, out.stdout + out.stderr
    assert len(SPINNERS) == 73


def test_corpus_5_meets_its_pre_registered_minimums():
    cases = load_jsonl(BENCH / "cases" / "status.jsonl")
    per = {}
    for c in cases:
        per[c["feature"]] = per.get(c["feature"], 0) + 1
    assert per["spinner"] >= 146
    assert per["status"] >= 30
    names = {c["renderable"]["name"] for c in cases if c["feature"] == "spinner"}
    assert names == set(SPINNERS), "every animation is in the corpus"
    for c in cases:
        assert (BENCH / "golden" / "status" / f"{c['id']}.ansi").exists()


def test_a_status_case_is_reproducible_in_a_fresh_process():
    from render_status_reference import render_case_isolated

    case = next(c for c in load_jsonl(BENCH / "cases" / "status.jsonl") if c["feature"] == "status" and c["renderable"]["terminal"])
    golden = (BENCH / "golden" / "status" / f"{case['id']}.ansi").read_bytes()
    assert render_case_isolated(case) == golden
    assert golden, "an interactive status writes something"


def test_a_status_on_a_stream_that_is_not_a_terminal_writes_nothing():
    cases = [c for c in load_jsonl(BENCH / "cases" / "status.jsonl") if c["feature"] == "status" and not c["renderable"]["terminal"]]
    assert cases
    for c in cases:
        assert (BENCH / "golden" / "status" / f"{c['id']}.ansi").read_bytes() == b""
