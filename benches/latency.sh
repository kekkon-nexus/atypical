#!/bin/sh
# commit-lint latency bench. Override knobs via env: RUNS=1000 mise run bench:latency
set -eu

cd "$(dirname "$0")/.."

BIN=${BIN:-target/release/commit-lint}
BIN_PACKED=${BIN_PACKED:-target/dist/commit-lint}
FIX=${FIX:-benches/fixtures}
OUT=${OUT:-benches/results.md}
WARMUP=${WARMUP:-10}
RUNS=${RUNS:-100}

command -v hyperfine >/dev/null 2>&1 || {
  echo "need hyperfine  ->  mise install" >&2
  exit 2
}

mise run build:rust
[ -x "$BIN" ] || { echo "no binary at $BIN" >&2; exit 2; }

# fixtures: each tool's valid msg (exits 0) + one invalid
# TODO: collapse to one string after the conventional preset
mkdir -p "$FIX"
printf 'add(exe)[int]: initial commit linting\n' > "$FIX/atypical-valid.txt"
printf 'feat[pre](lib): oops\n' > "$FIX/atypical-invalid.txt"
printf 'feat(api): add thing\n' > "$FIX/conventional-valid.txt"

# machine info
{
  echo "# latency"
  echo
  echo '```'
  echo "date   : $(date -u +%FT%TZ)"
  echo "host   : $(uname -srm)"
  echo "cpu    : $(grep -m1 'model name' /proc/cpuinfo 2>/dev/null | cut -d: -f2- | sed 's/^ *//' || uname -p)"
  echo "rustc  : $(rustc --version 2>/dev/null || echo '?')"
  echo "binary : $BIN ($(wc -c <"$BIN") bytes)"
  echo '```'
  echo
} >"$OUT"

TMP=$(mktemp)

# commitlint (vendored): npm --prefix benches install
CL=""
if [ -x benches/node_modules/.bin/commitlint ]; then
  CL="benches/node_modules/.bin/commitlint"
elif command -v commitlint >/dev/null 2>&1; then
  CL="commitlint"
fi

# skip commitlint if it can't resolve the config, else we'd just bench a crash
if [ -n "$CL" ] &&
  ! ERR=$($CL --config benches/commitlint.config.mjs --edit "$FIX/conventional-valid.txt" 2>&1); then
  echo ">> $CL cannot lint the fixture, skipping it:" >&2
  echo "$ERR" | head -3 | sed 's/^/   /' >&2
  CL=""
fi

# one table. -N: no shell (matches the git hook), -i: tolerate the invalid lane's exit 1
set -- \
  --command-name "atypical (valid)" "$BIN -- $FIX/atypical-valid.txt" \
  --command-name "atypical (invalid)" "$BIN -- $FIX/atypical-invalid.txt" \
  --command-name "atypical (valid + packed)" "$BIN_PACKED -- $FIX/atypical-invalid.txt" \
  --command-name "atypical (invalid + packed)" "$BIN_PACKED -- $FIX/atypical-invalid.txt" \

if [ -n "$CL" ]; then
  set -- "$@" --command-name "commitlint (valid)" \
    "$CL --config benches/commitlint.config.mjs --edit $FIX/conventional-valid.txt"
else
  echo ">> skipping commitlint  ->  npm --prefix benches install"
fi

hyperfine -N -i --time-unit millisecond --warmup "$WARMUP" --runs "$RUNS" \
  --export-markdown "$TMP" "$@"
cat "$TMP" >>"$OUT"

rm -f "$TMP"
echo ">> wrote $OUT"
