"""Docs metric for a candidate crate: documented public items and items with a doc example.

  docs_coverage.py <candidate-project-dir> <crate-name>

Runs `cargo +nightly rustdoc -p <crate> -- -Z unstable-options --show-coverage` from a project that depends on
the crate at the pinned version (rustdoc coverage is nightly-only). Targets in evaluation.md: 100% documented,
100% of public types with a compiling doctest; the doctest share is reported as the `examples` column, and
compile status comes from `cargo test --doc` run in a pristine copy of the published source of the crate
(cargo refuses `cargo test --doc -p <dependency>` for non-workspace members). Both runs use the lib target.
"""

import json
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from common import crate_info  # noqa: E402


def main():
    project, crate = Path(sys.argv[1]), sys.argv[2]
    cov = subprocess.run(["cargo", "+nightly", "rustdoc", "-p", crate, "--lib", "--", "-Z", "unstable-options", "--show-coverage"],
                         cwd=project, capture_output=True, text=True)
    gen = re.search(r'Generated output into "([^"]+)"', cov.stdout + cov.stderr)
    table = Path(gen.group(1)).read_text() if gen else cov.stdout
    total = next((ln for ln in table.splitlines() if ln.strip().startswith("| Total")), None)
    out = {"crate": crate, "raw_total_row": total}
    if total:
        nums = re.findall(r"([0-9.]+)%", total)
        cells = [c.strip() for c in total.strip("|").split("|")]
        out.update(documented=int(cells[1]), documented_pct=float(nums[0]), with_examples=int(cells[3]),
                   with_examples_pct=float(nums[1]))
    else:
        out["error"] = cov.stderr[-400:]
    info = crate_info(project, crate)
    with tempfile.TemporaryDirectory() as tmp:
        copy = Path(tmp) / "crate"
        shutil.copytree(info["src_dir"], copy, ignore=shutil.ignore_patterns("target"))
        lock = Path(project) / "Cargo.lock"
        if lock.exists() and not (copy / "Cargo.lock").exists():
            shutil.copy(lock, copy / "Cargo.lock")
        doctests = subprocess.run(["cargo", "test", "--doc"], cwd=copy, capture_output=True, text=True,
                                  env={**__import__("os").environ, "CARGO_TARGET_DIR": str(Path(tmp) / "target")})
    m = re.search(r"test result: (\w+)\. (\d+) passed; (\d+) failed; (\d+) ignored", doctests.stdout)
    if m:
        out["doctests"] = {"result": m.group(1), "passed": int(m.group(2)), "failed": int(m.group(3)),
                           "ignored": int(m.group(4)), "source": "published crate source copy"}
    else:
        out["doctests_error"] = (doctests.stdout + doctests.stderr)[-400:]
    print(json.dumps(out, indent=1))


if __name__ == "__main__":
    main()
