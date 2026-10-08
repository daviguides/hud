"""Public API metrics of a candidate crate from rustdoc JSON: name parity with Rich and API friction.

  api_surface.py json <project-dir> <crate>      writes and prints the rustdoc JSON path (nightly rustdoc, lib target; file named by the lib name)
  api_surface.py parity <rustdoc.json>           which of the 40 Rich names exist (normalized exact match)
  api_surface.py friction <rustdoc.json>         public functions with >3 positional params or Option params
  api_surface.py idiom <rustdoc.json> <Type>...  methods of the types that take `&mut self` or return a `Result`

Name normalization: lower case, underscores removed; `Type.member` matches a method of `Type`, an associated
item, an enum variant or a trait item of that name; `module.item` matches an item inside a module of that name.
Parity here is existence by name only. "Recognizable signature" is a manual review of the matched items, recorded
in the candidate report next to this output.
"""

import json
import subprocess
import sys
from collections import defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from common import crate_info  # noqa: E402

BENCH = Path(__file__).resolve().parent.parent


def norm(s):
    return s.replace("_", "").lower()


def load(path):
    return json.loads(Path(path).read_text())


def build(doc):
    idx = doc["index"]
    top, members, fns = set(), defaultdict(set), []

    def public(it):
        return it.get("visibility") == "public"

    def add_fn(owner, it):
        sig = it["inner"]["function"]["sig"]
        params = [(n, t) for n, t in sig["inputs"] if n != "self"]
        opts = [n for n, t in params if isinstance(t, dict) and "resolved_path" in t
                and t["resolved_path"]["path"].split("::")[-1] == "Option"]
        self_mut = any(n == "self" and isinstance(t, dict) and (t.get("borrowed_ref") or {}).get("is_mutable")
                       for n, t in sig["inputs"])
        out = sig.get("output")
        result = (isinstance(out, dict) and "resolved_path" in out
                  and out["resolved_path"]["path"].split("::")[-1] == "Result")
        fns.append({"owner": owner, "name": it["name"], "positional": len(params), "option_params": opts,
                    "self_mut": self_mut, "returns_result": result})

    for it in idx.values():
        name, inner = it.get("name"), it.get("inner", {})
        if not name or not public(it):
            continue
        kind = next(iter(inner), None)
        if kind in ("struct", "enum", "trait", "function", "module", "type_alias", "constant", "static", "macro", "use", "union"):
            top.add(norm(name))
        if kind == "function":
            add_fn("", it)
        if kind == "module":
            for cid in inner["module"]["items"]:
                c = idx.get(str(cid)) or idx.get(cid)
                if c and c.get("name") and public(c):
                    members[norm(name)].add(norm(c["name"]))
        if kind == "enum":
            for vid in inner["enum"]["variants"]:
                v = idx.get(str(vid)) or idx.get(vid)
                if v and v.get("name"):
                    members[norm(name)].add(norm(v["name"]))
        if kind == "trait":
            for cid in inner["trait"]["items"]:
                c = idx.get(str(cid)) or idx.get(cid)
                if c and c.get("name"):
                    members[norm(name)].add(norm(c["name"]))
    for it in idx.values():
        imp = it.get("inner", {}).get("impl")
        if not imp or imp.get("trait") is not None:
            continue
        tgt = imp["for"].get("resolved_path", {}).get("path", "").split("::")[-1]
        for cid in imp["items"]:
            c = idx.get(str(cid)) or idx.get(cid)
            if not c or not c.get("name") or not public(c):
                continue
            members[norm(tgt)].add(norm(c["name"]))
            if "function" in c["inner"]:
                add_fn(tgt, c)
    return top, members, fns


def parity(doc):
    names = json.loads((BENCH / "spec" / "name_parity.json").read_text())["names"]
    top, members, _ = build(doc)
    found = {}
    for n in names:
        if "." in n:
            a, b = n.split(".", 1)
            found[n] = norm(b) in members.get(norm(a), set())
        else:
            found[n] = norm(n) in top
    return found


def friction(doc):
    _, _, fns = build(doc)
    many = [f for f in fns if f["positional"] > 3]
    opt = [f for f in fns if f["option_params"]]
    padded = [f for f in fns if len(f["option_params"]) >= 2 or (f["positional"] > 3 and f["option_params"])]
    return {"public_functions": len(fns), "more_than_3_positional": many, "with_option_params": opt,
            "none_padding_candidates": padded}


def idiom(doc, owners):
    """Public methods of `owners` that need `&mut self` or return a `Result`: friction on the idiomatic path."""
    _, _, fns = build(doc)
    mine = [f for f in fns if f["owner"] in owners]
    return {"methods": len(mine), "needing_mut_self": [f for f in mine if f["self_mut"]],
            "returning_result": [f for f in mine if f["returns_result"]]}


def main():
    mode = sys.argv[1]
    if mode == "json":
        project, crate = Path(sys.argv[2]), sys.argv[3]
        subprocess.run(["cargo", "+nightly", "rustdoc", "-p", crate, "--lib", "--", "-Z", "unstable-options",
                        "--output-format", "json"], cwd=project, check=True, capture_output=True)
        info = crate_info(project, crate)
        print(info["target_dir"] / "doc" / (info["lib_name"].replace("-", "_") + ".json"))
    elif mode == "parity":
        found = parity(load(sys.argv[2]))
        hit = sum(found.values())
        print(f"name parity (existence by name): {hit}/{len(found)} = {hit / len(found):.0%}")
        print("missing:", [n for n, ok in found.items() if not ok])
    elif mode == "idiom":
        r = idiom(load(sys.argv[2]), set(sys.argv[3:]))
        print(f"methods of {sys.argv[3:]}: {r['methods']}; needing &mut self: {len(r['needing_mut_self'])}; "
              f"returning Result: {len(r['returning_result'])}")
        for x in r["needing_mut_self"] + r["returning_result"]:
            print("  ", x["owner"] + "::" + x["name"], "&mut self" if x["self_mut"] else "-> Result")
    elif mode == "friction":
        f = friction(load(sys.argv[2]))
        print(f"public functions: {f['public_functions']}; >3 positional: {len(f['more_than_3_positional'])}; "
              f"with Option params: {len(f['with_option_params'])}; None-padding candidates: {len(f['none_padding_candidates'])}")
        for x in f["none_padding_candidates"][:10]:
            print("  ", x["owner"] + "::" + x["name"], x["positional"], x["option_params"])


if __name__ == "__main__":
    main()
