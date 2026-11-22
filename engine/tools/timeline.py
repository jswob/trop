"""timeline: where a run's hours and its cost went, from its sessions' transcripts (the
process study after M2, change 6; the user's "Yes to changes 5, 6 and 7, go ahead",
2026-11-19; engine/PLAYBOOK.md, D14).

    engine/tools/timeline IDS-FILE ... [--projects DIR]

Each IDS-FILE is a run's ids file, one session a line: `<id> <role> <job> <model>` as
`launch` writes it, or `<id> <role>` from before jobs (a `lead-coordinator` role word
names the job too). It prints four Markdown tables, in local time, for the gate's report:

  1. the builders' turns by kind (plan, build, fix, last fix, gate, other): turns,
     wall hours, hours inside `check quick` and `check commit`, cost;
  2. the checks inside builder turns: runs, the median, the red runs and their minutes;
  3. the build worktree's gaps over 10 minutes (no builder turn running), each with the
     other jobs' turns that ran in it;
  4. the cost by job, the builders' split by kind of turn.

A turn starts at a prompt that isn't a tool result and ends at its session's last entry
before the next. A session's job is its ids line's; without one, it is read from the role
and the first prompt's first line (a lead session's "review … fix" is a fix review, "review
… whole" the gate's review, "review" a group review, "gate" the gate). A builder turn's
kind is read from its prompt's first line, without its parentheses, in this order: "plan
only"; "gate"; "last fix", "third turn" or "fix review"; "fix", "second turn" or "finding";
a session's first prompt, "fresh session", "builder group" or "turn <n>" is a build; a
compaction's summary or a notification keeps its session's last kind; any other is other.
A builder turn that starts after the run's first `gate` session started is a gate turn
(the skill engine-coordinator's §2 names the words a start message's first line uses). A
check's time is its call's, from the call to its result; a check is red when its result is
an error (a nonzero exit) or its output has a red line (`check quick: red at …`, or a
`FAIL <step>` line from before 2026-11-16), since a check piped through `tail` exits 0.

The gaps run from the run's first turn to the last builder turn's end; what follows is
printed as one line. The coordinators' turns (`coordinator`, `round-lead`) aren't listed
in a gap: they run throughout.

Cost counts each call once across every session, at `usage`'s weights and prices (the same
function), so its total is `usage`'s.

It reads Claude Code's transcripts, whose format can change under it: an entry whose type
isn't in KNOWN, a line that isn't JSON (but a last line still being written), or a user
or assistant entry of another shape stops it with exit 1 and the file and line. Add a new
type to KNOWN in a rules step once you know it carries nothing this tool reads.
"""

import glob
import json
import os
import re
import statistics
import sys
from datetime import datetime

TOOLS = os.path.dirname(os.path.realpath(__file__))
sys.path.insert(0, TOOLS)
import usage  # noqa: E402  one set of weights and prices

# The entries the transcripts held through tally 2.3 (2026-11-22). The first three are read;
# the rest carry nothing this tool needs. Since tally 2.2.1 (#246): `derived-from` names the
# sessions one came from (a resume, a compaction, a launch), `data-flow-records` the tools
# its calls used, `data-flow-incomplete` a list of codes; none has a time or a prompt.
READ = ("user", "assistant", "system")
KNOWN = READ + ("attachment", "api-request-blob", "api-request", "api-request-shape", "last-prompt", "session-owner",
                "atis-latch", "mode", "permission-mode", "pr-link", "queue-operation", "ai-title", "custom-title",
                "agent-name", "tools-used", "bridge-session", "cost-state", "file-history-delta",
                "file-history-snapshot", "summary", "derived-from", "data-flow-records", "data-flow-incomplete")
GAP_MIN = 10
RED = re.compile(r"^(FAIL \S|check (quick|commit): red\b)", re.M)
CHECKS = (("quick", re.compile(r"tools/check(\.py)?\s+quick\b")), ("commit", re.compile(r"tools/check(\.py)?\s+commit\b")))
KINDS = ("plan", "build", "fix", "last fix", "gate", "other")
LEADS = ("coordinator", "round-lead")
UUID = usage.UUID


class Unknown(Exception):
    """A transcript this tool can't read: the format changed."""


def ts(s):
    return datetime.fromisoformat(s.replace("Z", "+00:00")).timestamp()


def clock(t, day=True):
    return datetime.fromtimestamp(t).strftime("%m-%d %H:%M" if day else "%H:%M")


def sessions_of(paths):
    """[(id, role, job or None)] from the ids files, each id once, in order."""
    out, seen = [], set()
    for path in paths:
        with open(path, encoding="utf-8") as f:
            for n, line in enumerate(f, 1):
                w = line.split()
                if not w or w[0].startswith("#"):
                    continue
                if not UUID.match(w[0]):
                    raise ValueError(f"{path}:{n}: {w[0]!r} isn't a session id")
                if w[0] in seen:
                    continue
                seen.add(w[0])
                role, _, job = (w[1] if len(w) > 1 else "").partition("-")
                out.append((w[0], role, w[2] if len(w) > 2 else (job or None)))
    return out


def entries_of(session, bases):
    """The session's entries that this tool reads (its subagents' too), sorted by time."""
    seen, out = set(), []
    for b in bases:
        for path in [os.path.join(b, session + ".jsonl")] + sorted(glob.glob(os.path.join(b, session, "subagents", "*.jsonl"))):
            if not os.path.isfile(path):
                continue
            with open(path, encoding="utf-8", errors="replace") as f:
                lines = f.readlines()
            for n, line in enumerate(lines, 1):
                if not line.strip():
                    continue
                try:
                    e = json.loads(line)
                except ValueError as err:
                    if n == len(lines) and not line.endswith("\n"):
                        continue  # the last line of a live session, still being written
                    raise Unknown(f"{path}:{n}: not JSON ({err})") from err
                kind = e.get("type") if isinstance(e, dict) else None
                if kind not in KNOWN:
                    raise Unknown(f"{path}:{n}: an entry of type {kind!r} this tool doesn't know")
                if kind not in READ:
                    continue
                if not isinstance(e.get("timestamp"), str) or (kind != "system" and not isinstance(e.get("message"), dict)):
                    raise Unknown(f"{path}:{n}: a {kind} entry without a timestamp or a message")
                key = e.get("uuid") or (kind, e["timestamp"], line[:80])
                if key not in seen:
                    seen.add(key)
                    out.append(e)
    out.sort(key=lambda e: e["timestamp"])
    return out


def prompt_text(e):
    """The prompt's text when the entry starts a turn, else None."""
    if e.get("type") != "user" or e.get("isSidechain") or e.get("isMeta"):
        return None
    c = e["message"].get("content")
    if isinstance(c, str):
        return None if c.startswith(("<local-command", "<command-")) else c
    if isinstance(c, list):
        if any(isinstance(b, dict) and b.get("type") == "tool_result" for b in c):
            return None
        return " ".join(b.get("text", "") for b in c if isinstance(b, dict))
    return None


def heading(text):
    """The prompt's first line, lowercased; empty for a compaction's summary or a notification."""
    first = text.strip().splitlines()[0].lower() if text.strip() else ""
    return "" if first.startswith(("this session is being continued", "<")) else first


def bare(h):
    return re.sub(r"\([^)]*\)", "", h)


def job_from_prompt(role, h):
    if role == "lead":
        if "review" in h:
            return "fix-review" if "fix" in bare(h) else "gate-review" if "whole" in h else "group-review"
        return "gate" if "gate" in h else "coordinator"
    return usage_default(role)


def usage_default(role):
    return {"build": "builder", "tests": "tests", "rules": "rules-step", "spec": "spec-writer", "lead": "coordinator"}.get(
        role, role or "unknown")


def builder_kind(h, last, first):
    """A builder turn's kind from its prompt's first line (`first`: the session's first prompt)."""
    if not h:
        return last or "other"
    b = bare(h)
    if "plan only" in h:
        return "plan"
    if "gate" in b:
        return "gate"
    if any(w in b for w in ("last fix", "third turn", "fix review")):
        return "last fix"
    if any(w in b for w in ("fix", "second turn", "finding")):
        return "fix"
    if first or "fresh session" in h or "builder group" in b or re.search(r"\bturn \d", b):
        return "build"
    return "other"


def turns_of(entries, seen_calls):
    """The session's turns: start, end, heading, cost, check calls [(mode, seconds, red)]."""
    turns, cur, uses = [], None, {}
    for e in entries:
        t = ts(e["timestamp"])
        text = prompt_text(e)
        if text is not None:
            cur = {"start": t, "end": t, "heading": heading(text), "weighted": 0.0, "checks": []}
            turns.append(cur)
            continue
        if cur is None:
            continue
        cur["end"] = max(cur["end"], t)
        m = e.get("message") or {}
        if e["type"] == "assistant":
            key = m.get("id") or e.get("uuid")
            if m.get("usage") and key not in seen_calls:
                seen_calls.add(key)
                cur["weighted"] += usage.weighted({**m["usage"], "model": m.get("model")})
            for b in m.get("content") or []:
                if isinstance(b, dict) and b.get("type") == "tool_use" and b.get("name") == "Bash":
                    uses[b.get("id")] = (t, (b.get("input") or {}))
        elif e["type"] == "user" and isinstance(m.get("content"), list):
            for b in m["content"]:
                if isinstance(b, dict) and b.get("type") == "tool_result" and b.get("tool_use_id") in uses:
                    t0, inp = uses.pop(b["tool_use_id"])
                    mode = next((name for name, rx in CHECKS if rx.search(inp.get("command") or "")), None)
                    if mode and not inp.get("run_in_background"):
                        text = b.get("content")
                        if isinstance(text, list):
                            text = " ".join(x.get("text", "") for x in text if isinstance(x, dict))
                        red = bool(b.get("is_error")) or bool(RED.search(text or ""))
                        cur["checks"].append((mode, t - t0, red))
    return turns


def table(head, rows):
    out = ["| " + " | ".join(head) + " |", "|" + "|".join(" --- " for _ in head) + "|"]
    out += ["| " + " | ".join(str(c) for c in r) + " |" for r in rows]
    return out


def pct(w):
    return f"{w / usage.PER_PERCENT:.2f}"


def hours(s):
    return f"{s / 3600:.1f}"


def gaps(span, busy):
    """The stretches of `span` no interval of `busy` covers."""
    free, at = [], span[0]
    for s, e in sorted(busy):
        if s > at:
            free.append((at, s))
        at = max(at, e)
    if at < span[1]:
        free.append((at, span[1]))
    return free


def overlap(a, b):
    return max(0.0, min(a[1], b[1]) - max(a[0], b[0]))


def report(ids_files, bases):
    sessions = sessions_of(ids_files)
    seen_calls, runs, missing = set(), [], []
    for sid, role, job in sessions:
        entries = entries_of(sid, bases)
        turns = turns_of(entries, seen_calls)
        if not turns:
            missing.append(sid)
            continue
        runs.append({"id": sid, "role": role, "job": job or job_from_prompt(role, turns[0]["heading"]), "turns": turns})
    every = [t for r in runs for t in r["turns"]]
    if not every:
        return ["timeline: no transcript found for any session"]
    span = (min(t["start"] for t in every), max(t["end"] for t in every))
    gate_from = min((r["turns"][0]["start"] for r in runs if r["job"] == "gate"), default=None)
    for r in runs:
        last = None
        for n, t in enumerate(r["turns"]):
            kind = builder_kind(t["heading"], last, n == 0) if r["job"] == "builder" else None
            if kind and kind != "plan" and gate_from is not None and t["start"] >= gate_from:
                kind = "gate"
            t["kind"], last = kind, kind
    builder_turns = [t for r in runs if r["job"] == "builder" for t in r["turns"]]
    out = [f"timeline: {len(runs)} sessions ({len(missing)} with no transcript) from {', '.join(ids_files)}; "
           f"{clock(span[0])} to {clock(span[1])}, {hours(span[1] - span[0])} h, local time", ""]
    out += ["**The builders' turns, by kind**", ""]
    rows = []
    for kind in KINDS:
        ts_ = [t for t in builder_turns if t["kind"] == kind]
        if not ts_:
            continue
        wall = sum(t["end"] - t["start"] for t in ts_)
        inside = sum(s for t in ts_ for _, s, _ in t["checks"])
        rows.append([kind, len(ts_), hours(wall), f"{hours(inside)} ({inside / wall:.0%})" if wall else "0.0",
                     pct(sum(t["weighted"] for t in ts_))])
    wall = sum(t["end"] - t["start"] for t in builder_turns)
    inside = sum(s for t in builder_turns for _, s, _ in t["checks"])
    rows.append(["all", len(builder_turns), hours(wall), f"{hours(inside)} ({inside / wall:.0%})" if wall else "0.0",
                 pct(sum(t["weighted"] for t in builder_turns))])
    out += table(["Turn", "Turns", "Wall, h", "In checks, h", "Cost, %"], rows) + [""]
    out += ["**The checks inside builder turns**", ""]
    rows = []
    for mode, _ in CHECKS:
        calls = [(s, red) for t in builder_turns for m, s, red in t["checks"] if m == mode]
        if calls:
            reds = [s for s, red in calls if red]
            rows.append([f"`check {mode}`", len(calls), f"{statistics.median(s for s, _ in calls):.0f}", len(reds),
                         f"{sum(reds) / 60:.0f}"])
    out += table(["Check", "Runs", "Median, s", "Red runs", "Red, min"], rows) + [""]
    busy = [(t["start"], t["end"]) for t in builder_turns]
    tail = (max(e for _, e in busy), span[1]) if busy else span
    holes = [g for g in gaps((span[0], tail[0]), busy) if g[1] - g[0] > GAP_MIN * 60]
    out += [f"**The build worktree's gaps over {GAP_MIN} minutes** (no builder turn running)", ""]
    rows, alone = [], 0.0
    others = [(r["job"], (t["start"], t["end"])) for r in runs if r["job"] not in ("builder", *LEADS) for t in r["turns"]]
    for g in holes:
        by_job = {}
        for job, iv in others:
            by_job[job] = by_job.get(job, 0.0) + overlap(g, iv)
        ran = ", ".join(f"{j} {m / 60:.0f}" for j, m in sorted(by_job.items(), key=lambda x: -x[1]) if m >= 60)
        covered = sum(overlap(g, iv) for iv in merge([iv for _, iv in others]))
        alone += (g[1] - g[0]) - covered
        rows.append([clock(g[0]), clock(g[1], day=False), f"{(g[1] - g[0]) / 60:.0f}", ran or "none but the coordinator"])
    out += table(["From", "To", "Min", "Other jobs' turns in it, min"], rows)
    idle = sum(g[1] - g[0] for g in holes)
    out += ["", f"{len(holes)} gaps, {idle / 60:.0f} min; {alone / 60:.0f} min of them with no session but the "
                f"coordinator's running (a gate-8 run, CI, the user); after the last builder turn, "
                f"{(tail[1] - tail[0]) / 60:.0f} min to the run's last entry", ""]
    out += ["**The cost by job**", ""]
    rows, total = [], sum(t["weighted"] for t in every)
    jobs = {}
    for r in runs:
        if r["job"] == "builder":
            for t in r["turns"]:
                key = f"builder: {t['kind']}"
                n, w = jobs.get(key, (set(), 0.0))
                jobs[key] = (n | {r["id"]}, w + t["weighted"])
        else:
            n, w = jobs.get(r["job"], (set(), 0.0))
            jobs[r["job"]] = (n | {r["id"]}, w + sum(t["weighted"] for t in r["turns"]))
    for job, (n, w) in sorted(jobs.items(), key=lambda x: -x[1][1]):
        rows.append([job, len(n), pct(w), f"{w / total:.1%}" if total else "0 %"])
    rows.append(["all", len(runs), pct(total), "100 %"])
    out += table(["Job", "Sessions", "Cost, %", "Share"], rows)
    if missing:
        out += ["", "no transcript: " + " ".join(missing)]
    return out


def merge(ivs):
    out = []
    for s, e in sorted(ivs):
        if out and s <= out[-1][1]:
            out[-1] = (out[-1][0], max(out[-1][1], e))
        else:
            out.append((s, e))
    return out


def main(argv):
    projects = None
    if "--projects" in argv:
        k = argv.index("--projects")
        projects, argv = argv[k + 1] if k + 1 < len(argv) else "", argv[:k] + argv[k + 2 :]
    if not argv:
        print("usage: engine/tools/timeline IDS-FILE ... [--projects DIR]", file=sys.stderr)
        return 64
    roots = [projects] if projects else [os.path.expanduser(b + "/projects") for b in ("~/.claude", "~/.claude-personal")]
    bases = sorted(d for r in roots for d in glob.glob(os.path.join(r, "*")) if os.path.isdir(d))
    try:
        lines = report(argv, bases)
    except (OSError, ValueError) as e:
        print(f"timeline: {e}", file=sys.stderr)
        return 1
    except Unknown as e:
        print(f"timeline: refused: {e}. Claude Code's transcript format changed; read the entry, and add its type to "
              "KNOWN in tools/timeline.py in a rules step if it carries nothing this tool reads.", file=sys.stderr)
        return 1
    print("\n".join(lines))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
