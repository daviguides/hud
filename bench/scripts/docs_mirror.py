"""Local mirror of what a developer sees on docs.rs, for the DX first-try test.

  docs_mirror.py <project-dir> <crate> <outdir>

`cargo doc --no-deps -p <crate>` at the pinned version, copied without the `src/` source-view pages (docs.rs shows
them but the protocol grants documentation, not implementation), plus the crate README. The agent reads this
directory and nothing else.
"""

import shutil
import subprocess
import sys
from pathlib import Path


def main():
    project, crate, out = Path(sys.argv[1]), sys.argv[2], Path(sys.argv[3])
    subprocess.run(["cargo", "doc", "--no-deps", "-p", crate], cwd=project, check=True, capture_output=True)
    doc_root = project / "target" / "doc" / crate.replace("-", "_")
    if out.exists():
        shutil.rmtree(out)
    shutil.copytree(doc_root, out / "docs", ignore=shutil.ignore_patterns("src"))
    meta = subprocess.run(["cargo", "metadata", "--format-version", "1"], cwd=project, capture_output=True, text=True,
                          check=True).stdout
    import json
    pkg = next(p for p in json.loads(meta)["packages"] if p["name"] == crate)
    readme = Path(pkg["manifest_path"]).parent / (pkg.get("readme") or "README.md")
    if readme.exists():
        shutil.copy(readme, out / "README.md")
    print(f"{crate} {pkg['version']}: {sum(1 for _ in (out / 'docs').rglob('*.html'))} doc pages, README {'yes' if readme.exists() else 'no'} -> {out}")


if __name__ == "__main__":
    main()
