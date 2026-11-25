"""usage: what a run's sessions cost, as a share of the week (the retro after M0, decision 4).

    engine/tools/usage IDS-FILE|ID ...  [--projects DIR]

Each argument is an ids file (one session id per line, words after it ignored; `launch`
writes one) or a session id. The ids are deduplicated first, then each one's transcript
and its subagents' are found under ~/.claude*/projects/*/ (or DIR/*/), and every call is
counted once by its message id across all of them, so an id listed twice, or a call seen
in two files, counts once: a session's line counts the calls no earlier line counted, and
says how many it left to them. Weights and calibration are those of
tmp/session8c/tools/usage.py: input 1, cache reads 0.1, output 5, cache writes 1.25 (5 min)
or 2 (1 h), and 8.6 M weighted tokens to 1 % of the week.

Those weights are Opus's, and so is the calibration. A call on another model in PRICED
counts at that model's list price beside Opus 5.5's (Fable 5.2: twice on input, output and
cache writes, the same on cache reads; platform.claude.com, 2026-11-15), since a run's
budget counts Fable (the user, 2026-11-15; engine/PLAYBOOK.md, D10). When a session
ran on another model than Opus, a `by model:` block splits the total by the model each
call names, with what a priced model's tokens would be as Opus's and their dollars at list
price. A model with no price here counts as Opus, and its line says so.

The total line is the run's share of the week, the figure a budget counts. After it, two
lines estimate the account's own two weekly meters (`/usage`), in points of that meter,
which this tool's figure isn't: the week meter, for every model, and the Fable meter, for
Fable alone. Each sums each model's part times its factor in METERS, read on the account's
meters (engine/PLAYBOOK.md, D10), and each line says so.

When Orca answers (`orca account list --json`; `TALLY_ORCA` names another binary), a last
line gives the two meters as Orca reads them, in whole points with the time Orca read
them, and the same line is appended to `meters.log` beside the first ids file, so a run
holds a reading at each measurement (the Orca study, 2026-11-25: they replace D10's
screenshots). Nothing is printed when Orca doesn't answer.
"""

import datetime
import glob
import json
import os
import re
import subprocess
import sys

PER_PERCENT = 8.6e6
WEIGHTS = {"input_tokens": 1, "cache_read_input_tokens": 0.1, "output_tokens": 5}
OPUS, FABLE = "claude-opus-5-5", "claude-fable-5-2"
# model: (its weights beside Opus's: input, output and cache writes; cache reads), then
# dollars per million tokens: input, output, cache reads, cache writes 5 min and 1 h
PRICED = {
    OPUS: ((1, 1), (4, 20, 0.20, 5, 8)),
    FABLE: ((2, 1), (8, 40, 0.20, 10, 16)),
}
# meter: ({model: its factor on this tool's figure for that model's part; None for any
# other model}, where the factors were read). The readings (engine/PLAYBOOK.md, D10): three
# all-Opus weeks of 92 to 94 % by this tool each filled one week, so Opus's factor is about
# 1; with it, three stretches put Fable's at 1.7 (2026-11-17 22:00 to 11-18 08:40, clean:
# 15.06 and 5.18 % against 24 points), at least 2.1 (2026-11-21 14:48 to 11-22 08:00: 10.3
# and 7.0 against 25 or more) and at most 2.2 (2026-11-16 16:00 to 11-18 08:40: 43.5 and
# 15.0 against 76 or fewer). The Fable meter rose 22 points on the clean stretch.
METERS = {
    "week meter": ({OPUS: 1.0, FABLE: 2.0, None: 1.0}, "three readings to 2026-11-22, Fable 1.7 to 2.2"),
    "Fable meter": ({FABLE: 4.2}, "one stretch, 2026-11-18"),
}
ORCA_TIMEOUT = 20
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


def tokens(usage):
    """A call's tokens: input, output, cache reads, cache writes kept 5 minutes and 1 hour."""
    split = usage.get("cache_creation") or {}
    w5, w1 = (split.get("ephemeral_5m_input_tokens") or 0, split.get("ephemeral_1h_input_tokens") or 0) if split \
        else (usage.get("cache_creation_input_tokens") or 0, 0)
    return usage.get("input_tokens") or 0, usage.get("output_tokens") or 0, usage.get("cache_read_input_tokens") or 0, w5, w1


def weighted(usage, factors=None):
    """The call in weighted tokens, at its model's list price beside Opus's (or at `factors`)."""
    i, o, r, w5, w1 = tokens(usage)
    rest, read = factors or PRICED.get(usage.get("model"), PRICED[OPUS])[0]
    return rest * (WEIGHTS["input_tokens"] * i + WEIGHTS["output_tokens"] * o + 1.25 * w5 + 2 * w1) \
        + read * WEIGHTS["cache_read_input_tokens"] * r


def dollars(usage, prices):
    return sum(n * p for n, p in zip(tokens(usage), prices)) / 1e6


def by_model(calls):
    """The `by model:` lines, or none when every call ran on Opus."""
    models = {}
    for usage in calls.values():
        models.setdefault(usage.get("model") or "an unnamed model", []).append(usage)
    if set(models) <= {OPUS}:
        return []
    lines = ["by model:"]
    for model in sorted(models):
        total = sum(weighted(u) for u in models[model])
        line = f"  {model}: {len(models[model])} calls, {total / 1e6:.2f} M weighted, about {total / PER_PERCENT:.2f} % of the week"
        if model in PRICED and model != OPUS:
            as_opus = sum(weighted(u, PRICED[OPUS][0]) for u in models[model])
            line += (f", at its list price beside Opus's; as Opus's tokens {as_opus / PER_PERCENT:.2f} %; "
                     f"${sum(dollars(u, PRICED[model][1]) for u in models[model]):.2f}")
        elif model not in PRICED:
            line += ", counted as Opus's: no list price here"
        lines.append(line)
    return lines


def meters(calls):
    """The two meter lines: each meter's estimate, each model's part times its factor."""
    parts = {}
    for u in calls.values():
        parts[u.get("model")] = parts.get(u.get("model"), 0.0) + weighted(u) / PER_PERCENT
    order = sorted(parts, key=lambda m: (m != OPUS, m or ""))  # Opus first
    lines = []
    for meter, (factors, read) in METERS.items():
        terms = [(m or "an unnamed model", parts[m], factors.get(m, factors.get(None))) for m in order]
        terms = [t for t in terms if t[2] is not None and t[1] > 0]
        est = sum(p * f for _, p, f in terms)
        what = " and ".join(f"{m}'s part {p:.2f} % times {f:g}" for m, p, f in terms) or "no call it counts"
        lines.append(f"{meter}, the account's, not this run's share: about {est:.1f} points ({what}; {read})")
    return lines


def orca_meters():
    """Orca's reading of the account's week and Fable meters, as a line, or None."""
    try:
        r = subprocess.run([os.environ.get("TALLY_ORCA") or "orca", "account", "list", "--json"], capture_output=True,
                           text=True, timeout=ORCA_TIMEOUT, check=False)
        if r.returncode != 0:
            return None
        claude = json.loads(r.stdout)["result"]["rateLimits"]["claude"]
        week, fable = claude["weekly"]["usedPercent"], claude["fableWeekly"]["usedPercent"]
        read = datetime.datetime.fromtimestamp(claude["updatedAt"] / 1000, datetime.timezone.utc)
    except (OSError, ValueError, KeyError, TypeError, OverflowError, subprocess.TimeoutExpired):
        return None
    if not all(isinstance(v, (int, float)) for v in (week, fable)):
        return None
    return (f"Orca's meters, the account's: week {week:g} points, Fable {fable:g} points "
            f"(read {read.strftime('%Y-%m-%dT%H:%M:%SZ')})")


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
                        calls[m.get("id") or entry.get("uuid")] = {**m["usage"], "model": m.get("model")}
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
        earlier = f" ({len(calls) - len(fresh)} more counted in an earlier line)" if len(fresh) < len(calls) else ""
        print(f"session {s}: {len(fresh)} calls, {total / 1e6:.2f} M weighted, about {total / PER_PERCENT:.2f} % of the week{earlier}")
    for line in by_model(seen):
        print(line)
    print(f"all {len(sessions)} sessions: about {grand / PER_PERCENT:.2f} % of the week, the run's share")
    for line in meters(seen):
        print(line)
    orca = orca_meters()
    if orca:
        print(orca)
        logged = next((a for a in argv if os.path.isfile(a)), None)
        if logged:
            stamp = datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
            try:
                with open(os.path.join(os.path.dirname(os.path.realpath(logged)), "meters.log"), "a", encoding="utf-8") as f:
                    f.write(f"{stamp} {orca}\n")
            except OSError:
                pass
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
