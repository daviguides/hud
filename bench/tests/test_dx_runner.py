"""Self-tests of the DX first-try runner (evaluation.md, pilot validity rule 4): no model is ever called."""

import json
import subprocess
import sys

import pytest

import dx_runner as dx
from common import BENCH

PY = str(BENCH / ".venv" / "bin" / "python")


@pytest.fixture(scope="module")
def hud():
    return dx.hud_candidate()


def test_prompt_is_the_template_with_the_crate_and_the_task_filled_in(hud):
    prompt = dx.render_prompt(hud, "t03-progress")
    assert "`hud` version " + hud["version"] in prompt
    assert "fetch index" in prompt and "{crate}" not in prompt and "{version}" not in prompt
    assert "<task-id>" not in prompt
    assert prompt.rstrip().endswith("nothing after it.")


def test_the_last_rust_block_of_the_reply_is_the_program():
    reply = "first\n```rust\nfn a() {}\n```\nthen\n```rust\nfn main() {}\n```\n"
    assert dx.extract_rust(reply) == "fn main() {}\n"
    assert dx.extract_rust("no code here") is None


def test_literal_output_is_flagged_and_a_real_solution_is_not(hud):
    lines = dx.target_lines("t03-progress")
    assert lines
    cheat = "fn main() {\n" + "".join(f'    println!("{ln}");\n' for ln in lines) + "}\n"
    assert dx.literal_output(cheat, "t03-progress")
    for task, path in hud["solutions"].items():
        assert not dx.literal_output(path.read_text(), task), task


def test_audit_accepts_reads_inside_the_mirror_and_flags_everything_else(tmp_path):
    mirror = tmp_path / "mirror"
    (mirror / "docs").mkdir(parents=True)
    (mirror / "README.md").write_text("x")
    run = tmp_path / "run"
    run.mkdir()
    (run / "docs").symlink_to(mirror / "docs")
    (run / "README.md").symlink_to(mirror / "README.md")

    def events(*uses):
        return [{"type": "assistant", "message": {"content": [{"type": "tool_use", "name": n, "input": i} for n, i in uses]}}]

    read, bad = dx.audit(events(("Read", {"file_path": str(run / "README.md")}), ("Glob", {"pattern": "docs/**/*.html"})),
                         run, mirror)
    assert read and not bad
    _, bad = dx.audit(events(("Read", {"file_path": "/etc/hosts"})), run, mirror)
    assert bad == ["Read /etc/hosts"]
    _, bad = dx.audit(events(("Read", {"file_path": "../secret"})), run, mirror)
    assert bad
    _, bad = dx.audit(events(("Bash", {"command": "ls"})), run, mirror)
    assert bad == ["tool Bash"]
    read, bad = dx.audit(events(("Grep", {"pattern": "Progress", "path": "docs"})), run, mirror)
    assert read and not bad
    read, bad = dx.audit([], run, mirror)
    assert not read and not bad


def test_report_counts_only_real_attempts_and_labels_small_samples():
    rows = ([{"model": "m", "task": "a", "status": "success", "loc": 5}] * 4
            + [{"model": "m", "task": "a", "status": "candidate_failure", "loc": 7},
               {"model": "m", "task": "b", "status": "success", "loc": 9},
               {"model": "m", "task": "b", "status": "harness_error", "loc": None},
               {"model": "m", "task": "c", "status": "protocol_violation", "loc": None},
               {"model": "m", "task": "d", "status": "unsupported", "loc": None}])
    out = dx.report(rows)["m"]
    assert (out["runs"], out["success"]) == (6, 5)
    assert out["per_task"] == {"a": 0.8, "b": 1.0}
    assert out["excluded"] == {"harness_error": 1, "protocol_violation": 1, "unsupported": 1}
    assert out["pipeline_check"] and out["gate"] == "inconclusive (pipeline check)"
    assert out["ci95_over_tasks"] == dx.bootstrap_ci([0.8, 1.0])


def test_the_gate_is_read_only_on_a_full_sample():
    rows = [{"model": "m", "task": f"t{i % 8}", "status": "success" if i % 5 else "candidate_failure", "loc": 3}
            for i in range(40)]
    assert dx.report(rows)["m"]["gate"] == "pass"
    rows[1]["status"] = rows[2]["status"] = "candidate_failure"
    assert dx.report(rows)["m"]["gate"] == "fail"
    assert dx.report([{**r, "model": "mock-primary"} for r in rows])["mock-primary"]["gate"] == "inconclusive (pipeline check)"


def test_estimate_prints_tokens_and_a_cost_only_when_prices_are_configured():
    config = dx.load_config()
    plain = dx.estimate(config)
    assert [p["runs"] for p in plain["plan"]] == [8 * 5, 8 * 3]
    assert all(p["cost_usd"] is None for p in plain["plan"])
    config["primary"]["id"] = "m"
    config["estimate"]["prices_usd_per_mtok"] = {"m": {"input": 3.0, "output": 15.0}}
    priced = dx.estimate(config)["plan"][0]
    assert priced["cost_usd"] == pytest.approx((priced["input_tokens"] * 3.0 + priced["output_tokens"] * 15.0) / 1e6)


def test_the_real_agent_can_only_read_and_never_runs_without_the_flags(tmp_path):
    cmd = dx.ClaudeCli("bare").command("prompt", tmp_path, "some-model", dx.load_config()["limits"])
    assert cmd[:2] == ["claude", "-p"] and "--bare" in cmd
    assert cmd[cmd.index("--tools") + 1] == "Read,Grep,Glob"
    assert cmd[cmd.index("--permission-mode") + 1] == "dontAsk"
    assert "Bash" not in " ".join(cmd) and "--max-turns" in cmd
    for argv, text in ((["run"], "choose --dry-run"), (["run", "--execute"], "needs both model ids")):
        proc = subprocess.run([PY, str(BENCH / "scripts" / "dx_runner.py"), *argv, "--work", str(tmp_path)],
                              capture_output=True, text=True)
        assert proc.returncode != 0 and text in proc.stderr


@pytest.fixture(scope="module")
def prepared(hud, tmp_path_factory):
    work = tmp_path_factory.mktemp("dx")
    return work, dx.prepare_mirror(hud, work / "mirror"), dx.prepare_sandbox(hud, work / "sandbox")


@pytest.mark.parametrize("mode, status, reason", [
    ("solution", "success", None),
    ("broken", "candidate_failure", "compile error"),
    ("wrong", "candidate_failure", "output differs"),
    ("peek", "protocol_violation", "Read /etc/hosts"),
])
def test_dry_run_pipeline_classifies_a_good_a_broken_a_wrong_and_a_peeking_agent(hud, prepared, mode, status, reason):
    work, mirror, sandbox = prepared
    row = dx.one_run(hud, dx.MockAgent(hud, mode), "mock", "t03-progress", 1, work, mirror, sandbox, dx.load_config()["limits"])
    assert row["status"] == status
    assert reason is None or reason in row["reason"]
    if mode == "solution":
        assert row["loc"] and (work / "runs" / "mock-t03-progress-1" / "transcript.jsonl").exists()


def test_a_task_without_a_solution_is_unsupported_not_a_pass(hud, prepared):
    work, mirror, sandbox = prepared
    row = dx.one_run(hud, dx.MockAgent(hud), "mock", "t05-error", 1, work, mirror, sandbox, dx.load_config()["limits"])
    assert row["status"] == "unsupported"
