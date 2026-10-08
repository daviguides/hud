"""Shared helpers for the hud bench scripts."""

import gzip
import json
import os
import re
import select
import subprocess
import unicodedata
from pathlib import Path

import regex
import wcwidth
from rich.cells import cell_len

BENCH = Path(__file__).resolve().parent.parent
ANSI_RE = re.compile(rb"\x1b\[[0-9;?]*[ -/]*[@-~]|\x1b\][^\x07\x1b]*(?:\x07|\x1b\\)")
GRAPHEME_RE = regex.compile(r"\X")


def load_jsonl(path):
    with open(path, encoding="utf-8") as f:
        return [json.loads(line) for line in f if line.strip()]


def read_bytes(path):
    path = Path(path)
    if path.suffix == ".gz":
        return gzip.decompress(path.read_bytes())
    return path.read_bytes()


def strip_ansi(data: bytes) -> bytes:
    return ANSI_RE.sub(b"", data)


def graphemes(text: str):
    return GRAPHEME_RE.findall(text)


def rich_width(text: str) -> int:
    return cell_len(text)


def wcwidth_width(text: str) -> int:
    w = wcwidth.wcswidth(text)
    if w < 0:
        w = wcwidth.wcswidth("".join(c for c in text if unicodedata.category(c) != "Cc"))
    return w


def cluster_width(text: str) -> int:
    """Third reference: UAX #29 clusters, each as wide as its first code point.

    A cluster containing U+FE0F (emoji presentation) or a regional-indicator pair,
    a ZWJ sequence of emoji, or an emoji modifier sequence is 2 cells. Control and
    format characters are 0.
    """
    total = 0
    for g in graphemes(text):
        first = g[0]
        if unicodedata.category(first) in ("Cc", "Cf", "Zl", "Zp"):
            total += 0
            continue
        w = max(wcwidth.wcwidth(first), 0)
        if "️" in g or (len(g) >= 2 and all("\U0001F1E6" <= c <= "\U0001F1FF" for c in g[:2])):
            w = 2
        elif "‍" in g and w == 2:
            w = 2
        elif any("\U0001F3FB" <= c <= "\U0001F3FF" for c in g[1:]):
            w = 2
        total += w
    return total


def fold_ref(text: str, w: int):
    """Greedy hard fold at grapheme boundaries: lines of at most w cells.

    A cluster wider than w goes alone on its line. Zero-width clusters stay on
    the current line. Concatenating the lines gives back the input exactly.
    """
    lines, cur, cur_w = [], "", 0
    for g in graphemes(text):
        gw = rich_width(g)
        if cur and cur_w + gw > w:
            lines.append(cur)
            cur, cur_w = "", 0
        cur += g
        cur_w += gw
    if cur or not lines:
        lines.append(cur)
    return lines


def truncate_ref(text: str, w: int) -> str:
    return fold_ref(text, w)[0]


def crate_info(project, crate):
    """Resolve a dependency of `project` through cargo metadata: its lib target name (which can differ from the
    package name, e.g. package rs-rich, lib `rich`), its source directory and the target directory."""
    meta = json.loads(subprocess.run(["cargo", "metadata", "--format-version", "1"], cwd=project,
                                     capture_output=True, text=True, check=True).stdout)
    pkg = next(p for p in meta["packages"] if p["name"] == crate)
    lib = next(t for t in pkg["targets"] if any(k in ("lib", "rlib", "proc-macro") for k in t["kind"]))
    return {"lib_name": lib["name"], "src_dir": Path(pkg["manifest_path"]).parent,
            "target_dir": Path(meta["target_directory"]), "version": pkg["version"]}


def read_pty(master, slave, proc, timeout=120):
    """Everything `proc` wrote to the pty, however fast it exits.

    On macOS the last close of the slave side can discard output the master has not read yet, so
    the parent keeps its own slave descriptor open until the process is gone and the master is
    drained; reading until EIO (the old way) lost the whole output of about one run in 600.
    """
    import time

    chunks = []
    deadline = time.monotonic() + timeout
    try:
        while True:
            ready, _, _ = select.select([master], [], [], 0.05)
            if ready:
                data = os.read(master, 65536)
                if not data:
                    break
                chunks.append(data)
                continue
            if proc.poll() is not None:
                while select.select([master], [], [], 0)[0]:
                    data = os.read(master, 65536)
                    if not data:
                        break
                    chunks.append(data)
                break
            if time.monotonic() > deadline:
                proc.kill()
                raise subprocess.TimeoutExpired(proc.args, timeout)
    except OSError:
        pass
    finally:
        proc.wait(timeout=timeout)
        os.close(slave)
        os.close(master)
    return b"".join(chunks)
