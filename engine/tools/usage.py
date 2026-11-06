"""usage: what a run's sessions cost, as a share of the week (the retro after M0, decision 4).

    engine/tools/usage IDS-FILE|ID ...  [--projects DIR]

Each argument is an ids file (one session id per line, words after it ignored; `launch`
writes one) or a session id. The ids are deduplicated first, then each one's transcript
and its subagents' are found under ~/.claude*/projects/*/ (or DIR/*/), and every call is
counted once by its message id across all of them, so an id listed twice, or a call seen
in two files, counts once. Weights and calibration are those of
tmp/session8c/tools/usage.py: input 1, cache reads 0.1, output 5, cache writes 1.25 (5 min)
or 2 (1 h), and 8.6 M weighted tokens to 1 % of the week.
"""

import glob
import json
import os
import re
import sys

PER_PERCENT = 8.6e6
WEIGHTS = {"input_tokens": 1, "cache_read_input_tokens": 0.1, "output_tokens": 5}
UUID = re.compile(r"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$")


def ids_of(args):
    seen = []
    for a in args:
        words = [a] if UUID.match(a) else []
        if not words:
            with open(a, encoding="utf-8") as f:
                words = [line.split()[0] for line in f if line.strip() and not line.lstrip().startswith("#")]
        for w in words:
            if not UUID.match(w):
                raise ValueError(f"{w!r} isn't a session id")
            if w not in seen:
                seen.append(w)
    return seen


def weighted(usage):
    total = sum(w * (usage.get(k) or 0) for k, w in WEIGHTS.items())
    split = usage.get("cache_creation") or {}
    if split:
        total += 1.25 * (split.get("ephemeral_5m_input_tokens") or 0) + 2 * (split.get("ephemeral_1h_input_tokens") or 0)
    else:
        total += 1.25 * (usage.get("cache_creation_input_tokens") or 0)
    return total


def calls_of(session, bases):
    calls = {}
    for b in bases:
        for path in [os.path.join(b, session + ".jsonl")] + sorted(glob.glob(os.path.join(b, session, "subagents", "*.jsonl"))):
            if not os.path.isfile(path):
                continue
            with open(path, encoding="utf-8", errors="replace") as f:
                for line in f:
                    try:
                        entry = json.loads(line)
                    except ValueError:
                        continue
                    m = entry.get("message") or {}
                    if entry.get("type") == "assistant" and isinstance(m, dict) and m.get("usage"):
                        calls[m.get("id") or entry.get("uuid")] = m["usage"]
    return calls


def main(argv):
    projects = None
    if "--projects" in argv:
        k = argv.index("--projects")
        projects, argv = argv[k + 1] if k + 1 < len(argv) else "", argv[:k] + argv[k + 2 :]
    if not argv:
        print("usage: engine/tools/usage IDS-FILE|ID ... [--projects DIR]", file=sys.stderr)
        return 64
    try:
        sessions = ids_of(argv)
    except (OSError, ValueError) as e:
        print(f"usage: {e}", file=sys.stderr)
        return 1
    roots = [projects] if projects else [os.path.expanduser(b + "/projects") for b in ("~/.claude", "~/.claude-personal")]
    bases = sorted(d for r in roots for d in glob.glob(os.path.join(r, "*")) if os.path.isdir(d))
    grand, seen = 0.0, {}
    for s in sessions:
        calls = calls_of(s, bases)
        fresh = {k: v for k, v in calls.items() if k not in seen}
        seen.update(fresh)
        total = sum(weighted(u) for u in fresh.values())
        grand += total
        print(f"session {s}: {len(fresh)} calls, {total / 1e6:.2f} M weighted, about {total / PER_PERCENT:.2f} % of the week")
    print(f"all {len(sessions)} sessions: about {grand / PER_PERCENT:.2f} % of the week")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
