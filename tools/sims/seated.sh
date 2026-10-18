#!/usr/bin/env bash
# seated.sh SIM DIR "SEEDS" ARM...: one-hour house games with the eight house programs exported from this checkout and
# seated by source, JOBS (9) at a time. Each ARM is NAME:PATCH:FLAGS, PATCH a file in tools/sims/variants without its
# .patch (- for none) and FLAGS extra sim flags, comma-separated. Writes DIR/NAME/house-SEED.json, the shape summary.py
# reads. Arms that differ only in PATCH compare house code on one engine binary. Works with macOS's bash 3.2.
set -euo pipefail
sim=$(cd "$(dirname "$1")" && pwd)/$(basename "$1"); dir=$2; seeds=$3; shift 3
here=$(cd "$(dirname "$0")/../.." && pwd)
mkdir -p "$dir"; dir=$(cd "$dir" && pwd)
work=$(mktemp -d)
cleanup() { git -C "$here" worktree remove --force "$work/df" >/dev/null 2>&1 || true; rm -rf "$work"; }
trap cleanup EXIT
git -C "$here" worktree add -q --detach "$work/df" HEAD
houses="kublai arjuna templar atlantis fleet manifesto nullptr nations"
for arm in "$@"; do
  IFS=: read -r name patch flags <<< "$arm"
  git -C "$work/df" checkout -q -- shared factions src
  if [ "$patch" != - ]; then git -C "$work/df" apply "$here/tools/sims/variants/$patch.patch"; fi
  mkdir -p "$dir/$name/src"
  for h in $houses; do node "$work/df/tools/source.mjs" "$h" "$dir/$name/src/$h.rs" > /dev/null; done
  echo "${flags//,/ }" > "$dir/$name/flags"
done
one() {
  local name=$1 seed=$2 i=0 seats=""
  for h in $houses; do seats="$seats --seat $i=$dir/$name/src/$h.rs"; i=$((i + 1)); done
  # shellcheck disable=SC2086
  "$sim" watch --cpu-rts --structures --house --world globe --factions 8 --provinces 44 --max-factions 16 --max-provinces 200 \
    --series 60 --seed "$seed" --secs "${SECS:-3600}" --battles $(cat "$dir/$name/flags") $seats \
    > "$dir/$name/house-$seed.json" 2> "$dir/$name/house-$seed.err"
}
for seed in $seeds; do
  for arm in "$@"; do
    while [ "$(jobs -rp | wc -l)" -ge "${JOBS:-9}" ]; do sleep 2; done
    one "${arm%%:*}" "$seed" &
  done
done
wait
echo "done: $# arms, seeds $seeds"
