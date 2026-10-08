#!/usr/bin/env bash
# width-runner contract of pilot.py for hud: the width, fold and truncate files, the Unicode table
# cases as tables/ and the panel and tree outputs of the same cases (cases_runner --widgets).
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
bench="$(cd "$here/../.." && pwd)"
bin="$here/target/release"
out="$1"
mkdir -p "$out/tables"
"$bin/width_runner" "$out"
"$bin/cases_runner" "$bench/cases/table_unicode.jsonl" "$out/table_unicode_cases"
cp "$out"/table_unicode_cases/tw-*.ansi "$out/tables/"
rm -rf "$out/table_unicode_cases"
"$bin/cases_runner" --widgets "$bench/cases/table_unicode.jsonl" "$out"
