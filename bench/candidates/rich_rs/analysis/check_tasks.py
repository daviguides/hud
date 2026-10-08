import sys, difflib
from pathlib import Path
sys.path.insert(0, "/Users/daviguides/work/sources/hud/bench/scripts")
import tasks as T
BIN = Path("/Users/daviguides/work/sources/hud/bench/candidates/rich_rs/tasks/target/release")
M = {"t01-table":"t01","t02-panel":"t02","t03-progress":"t03","t04-tree":"t04","t05-error":"t05","t06-markup":"t06","t07-pipe":"t07","t08-env":"t08"}
sel = sys.argv[1:] or list(M)
for tid in sel:
    task = T.find_task(tid)
    b = BIN / M[tid]
    if not b.exists():
        print("MISSING", tid); continue
    for run in task["runs"]:
        out = T.capture([str(b)], run)
        want = T.load_golden(tid, run)
        if run["check"] == "bytes":
            ok = out == want
            print(("PASS" if ok else "FAIL"), tid, run["name"])
            if not ok:
                for l in difflib.unified_diff(want.decode().splitlines(), out.decode("utf-8","replace").splitlines(), "want", "got", lineterm="", n=0):
                    print("   ", repr(l))
                print("    got_tail", repr(out[-60:]))
        else:
            got = T.screen_snapshot(out)
            ok = got == want
            print(("PASS" if ok else "FAIL"), tid, run["name"])
            if not ok:
                print("--- want\n" + T.screen_text(want) + "\n--- got\n" + T.screen_text(got))
                for i, (g, w) in enumerate(zip(got, want)):
                    if g != w: print("   row", i, "got", g, "\n         want", w); break
