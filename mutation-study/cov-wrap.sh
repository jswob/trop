#!/bin/sh
# nextest run-wrapper: each test's own LLVM profile, named by its binary and test name
bin="$1"; name=""; prev=""
for a in "$@"; do [ "$prev" = "--exact" ] && name="$a"; prev="$a"; done
b="$(basename "$bin" | sed 's/-[0-9a-f]\{16\}$//')"
safe="$(printf '%s' "$name" | tr -c 'A-Za-z0-9_.' '_')"
export LLVM_PROFILE_FILE="$COV_OUT/${b}@@${safe}.%p.profraw"
exec "$@"
