"""launch: start one role session for a coordinator, headless, in the background (the
retro after M0, decision 4; the skill engine-coordinator).

    engine/tools/launch ROLE PROMPT-FILE WORKTREE --ids FILE [--effort E] [--model M]
                        [--resume ID] [--dry-run]

It runs, from WORKTREE, with the environment every M0 launch used:

    CARGO_BUILD_JOBS=6 NEXTEST_TEST_THREADS=6 CARGO_INCREMENTAL=0 TALLY_ROLE=ROLE \\
      nice -n 10 claude -p "<the prompt file's text>" --model M --effort E \\
      --output-format json --session-id <a new id> --add-dir <main>/tmp/active < /dev/null

detached, its JSON answer in <ids file's folder>/out/<id>.json and its errors beside it.
The session id is fixed in advance (`--session-id`), so it is in the ids file before the
session starts; `--resume ID` answers that session instead, under the same id. Effort
defaults by role: build and tests xhigh, the rest high.

It refuses:
  - outside the lead role and app work (the master), which are the coordinators;
  - a role the guard doesn't know, a missing prompt file, a worktree that isn't a
    checkout of this repo;
  - when fewer than 30 GB are free on the worktree's disk (a full disk lost M0 a builder
    turn).
It warns when two heavy jobs already run (cargo builds and tests, or launched sessions):
the Mac takes two at most. `--dry-run` prints the command and the checks, and starts
nothing.
"""

import json
import os
import shutil
import subprocess
import sys
import uuid

TOOLS = os.path.dirname(os.path.realpath(__file__))
ROLES = ("build", "tests", "rules", "spec", "lead")
EFFORT = {"build": "xhigh", "tests": "xhigh", "rules": "high", "spec": "high", "lead": "high"}
MODEL = "claude-opus-5-5"
MIN_FREE_GB = 30
ENV = {"CARGO_BUILD_JOBS": "6", "NEXTEST_TEST_THREADS": "6", "CARGO_INCREMENTAL": "0"}
HEAVY = ("cargo build", "cargo test", "cargo nextest", "cargo clippy", "cargo fuzz", "cargo mutants", "cargo-nextest",
         "cargo-mutants", "cargo-fuzz", "claude -p")


class Refused(Exception):
    """A refusal, with its reason."""


def common_dir(folder):
    r = subprocess.run(["git", "-C", folder, "rev-parse", "--path-format=absolute", "--git-common-dir"],
                       capture_output=True, text=True, check=False)
    if r.returncode != 0:
        raise Refused(f"{folder} isn't a git checkout")
    return os.path.realpath(r.stdout.strip())


def main_checkout(worktree):
    """The main checkout of the repo `worktree` belongs to, which must be this tool's."""
    common = common_dir(worktree)
    if common != common_dir(TOOLS):
        raise Refused(f"{worktree} is a checkout of another repo")
    if os.path.basename(common) != ".git":
        raise Refused(f"{worktree}'s git folder {common} isn't a main checkout's .git")
    return os.path.dirname(common)


def heavy_jobs():
    r = subprocess.run(["ps", "-axo", "command"], capture_output=True, text=True, check=False)
    return [line.strip()[:120] for line in r.stdout.splitlines()[1:] if any(h in line for h in HEAVY)]


def parse(argv):
    opts = {"--effort": None, "--model": MODEL, "--resume": None, "--ids": None}
    dry, words = False, []
    it = iter(argv)
    for a in it:
        if a == "--dry-run":
            dry = True
        elif a in opts:
            opts[a] = next(it, None)
            if opts[a] is None:
                raise Refused(f"{a} needs a value")
        elif a.startswith("--") and a.partition("=")[0] in opts:
            opts[a.partition("=")[0]] = a.partition("=")[2]
        elif a.startswith("-"):
            raise Refused(f"unknown option {a}")
        else:
            words.append(a)
    if len(words) != 3 or not opts["--ids"]:
        raise Refused("usage: engine/tools/launch ROLE PROMPT-FILE WORKTREE --ids FILE [--effort E] [--model M] [--resume ID] [--dry-run]")
    return words, opts, dry


def launch(argv, env_in):
    words, opts, dry = parse(argv)
    role, prompt_file, worktree = words
    me = env_in.get("TALLY_ROLE") or "unset"
    if me not in ("lead", "unset"):
        raise Refused(f"only a coordinator (the lead role, or app work) launches sessions, and this is {me}")
    if role not in ROLES:
        raise Refused(f"{role!r} isn't a role ({', '.join(ROLES)})")
    if not os.path.isfile(prompt_file):
        raise Refused(f"{prompt_file} isn't a file")
    worktree = os.path.realpath(worktree)
    if not os.path.isdir(worktree):
        raise Refused(f"{worktree} isn't a folder")
    main = main_checkout(worktree)
    if opts["--resume"]:
        try:
            sid = str(uuid.UUID(opts["--resume"]))
        except ValueError as e:
            raise Refused(f"--resume {opts['--resume']!r} isn't a session id") from e
    else:
        sid = str(uuid.uuid4())
    free_gb = shutil.disk_usage(worktree).free / 2**30
    floor = float(env_in.get("TALLY_LAUNCH_MIN_FREE_GB") or MIN_FREE_GB)
    if free_gb < floor:
        raise Refused(f"{free_gb:.1f} GB free on {worktree}'s disk, under {floor:g} GB: free a worktree's "
                      "target first (`engine/tools/worktree-done --target-only`)")
    with open(prompt_file, encoding="utf-8") as f:
        prompt = f.read()
    effort = opts["--effort"] or EFFORT[role]
    ids = os.path.realpath(opts["--ids"])
    out_dir = os.path.join(os.path.dirname(ids), "out")
    claude = ["claude", "-p", prompt, "--model", opts["--model"], "--effort", effort, "--output-format", "json",
              "--add-dir", os.path.join(main, "tmp", "active")]
    claude += ["--resume", sid] if opts["--resume"] else ["--session-id", sid]
    argv_ = ["nice", "-n", "10", *claude]
    env = {k: v for k, v in env_in.items() if k != "TALLY_ROLE"} | ENV | {"TALLY_ROLE": role}
    jobs = heavy_jobs()
    shown = " ".join(f"{k}={v}" for k, v in {**ENV, "TALLY_ROLE": role}.items())
    print(f"launch: {role} session {sid} in {worktree} (effort {effort}, {free_gb:.0f} GB free)")
    print(f"launch: {shown} nice -n 10 claude -p <{prompt_file}, {len(prompt)} chars> --model {opts['--model']} "
          f"--effort {effort} --output-format json {'--resume' if opts['--resume'] else '--session-id'} {sid} "
          f"--add-dir {os.path.join(main, 'tmp', 'active')} < /dev/null")
    if len(jobs) >= 2:
        print(f"launch: warning: {len(jobs)} heavy jobs already run (the Mac takes two at most): " + "; ".join(jobs[:4]))
    if dry:
        print("launch: dry run: nothing started, the ids file unchanged")
        return 0
    os.makedirs(out_dir, exist_ok=True)
    known = set()
    if os.path.isfile(ids):
        with open(ids, encoding="utf-8") as f:
            known = {line.split()[0] for line in f if line.strip()}
    if sid not in known:
        with open(ids, "a", encoding="utf-8") as f:
            f.write(f"{sid} {role}\n")
    stamp = len([n for n in os.listdir(out_dir) if n.startswith(sid)])
    out = os.path.join(out_dir, f"{sid}.{stamp}.json")
    with open(out, "wb") as o, open(out[:-5] + ".err", "wb") as e, open(os.devnull, "rb") as i:
        p = subprocess.Popen(argv_, cwd=worktree, env=env, stdin=i, stdout=o, stderr=e, start_new_session=True)
    print(json.dumps({"session_id": sid, "pid": p.pid, "out": out}))
    return 0


def main(argv):
    try:
        return launch(argv, dict(os.environ))
    except (Refused, OSError) as r:
        print(f"launch: refused: {r}.", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
