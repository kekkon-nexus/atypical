#!/bin/sh
# commit-lint latency bench. Override knobs via env: RUNS=1000 bun run bench
set -eu

cd "$(dirname "$0")/.."

BIN=${BIN:-target/release/commit-lint}
FIX=${FIX:-benches/fixtures}
OUT=${OUT:-benches/results.md}
WARMUP=${WARMUP:-10}
RUNS=${RUNS:-100}

command -v hyperfine >/dev/null 2>&1 || {
  echo "need hyperfine  ->  https://github.com/sharkdp/hyperfine" >&2
  exit 2
}

bun run build
[ -x "$BIN" ] || { echo "no binary at $BIN" >&2; exit 2; }

# fixtures: each tool's valid msg (exits 0) + invalid msg (exits 1)
mkdir -p "$FIX"
printf 'add(exe)[int]: initial commit linting\n' > "$FIX/atypical-valid.txt"
printf 'feat[pre](lib): oops\n' > "$FIX/atypical-invalid.txt"
printf 'feat(api): add thing\n' > "$FIX/conventional-valid.txt"
printf 'bogus(api): add thing\n' > "$FIX/conventional-invalid.txt"

# commitlint: bun install (benches workspace)
CL=""
if [ -x benches/node_modules/.bin/commitlint ]; then
  CL="benches/node_modules/.bin/commitlint"
elif [ -x node_modules/.bin/commitlint ]; then
  CL="node_modules/.bin/commitlint"
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
  if [ -n "$CL" ]; then
    echo "deps   : @commitlint/cli + config-conventional ($(node benches/footprint.mjs benches @commitlint/cli @commitlint/config-conventional))"
    echo "node   : $(command -v node) ($(wc -c <"$(command -v node)") bytes)"
  fi
  echo '```'
  echo
} >"$OUT"

TMP=$(mktemp)

# one table. -N: no shell (matches the git hook), -i: tolerate the invalid lanes' exit 1
set -- \
  --command-name "atypical (valid)" "$BIN -- $FIX/atypical-valid.txt" \
  --command-name "atypical (invalid)" "$BIN -- $FIX/atypical-invalid.txt"

if [ -n "$CL" ]; then
  CLR="$CL --config benches/commitlint.config.mjs --edit"
  set -- "$@" \
    --command-name "commitlint (valid)" "$CLR $FIX/conventional-valid.txt" \
    --command-name "commitlint (invalid)" "$CLR $FIX/conventional-invalid.txt"
else
  echo ">> skipping commitlint  ->  bun install"
fi

hyperfine -N -i --metrics time_wall_clock:ms --warmup "$WARMUP" --runs "$RUNS" \
  --export-markdown "$TMP" "$@"
cat "$TMP" >>"$OUT"

rm -f "$TMP"
echo ">> wrote $OUT"
