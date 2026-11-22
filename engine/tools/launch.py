"""launch: start one role session for a coordinator, headless, in the background (the
retro after M0, decision 4; the skill engine-coordinator).

    engine/tools/launch ROLE PROMPT-FILE WORKTREE --ids FILE [--job J] [--effort E]
                        [--model M] [--max-usd N] [--resume ID] [--dry-run]
    engine/tools/launch --jobs

It runs, from WORKTREE, with this environment:

    CARGO_BUILD_JOBS=6 NEXTEST_TEST_THREADS=6 TALLY_ROLE=ROLE \\
      nice -n 10 claude -p "<the prompt file's text>" --model M --effort E \\
      --output-format json --session-id <a new id> --add-dir <main>/tmp/active < /dev/null

detached, its JSON answer in <ids file's folder>/out/<id>.json and its errors beside it.
CARGO_INCREMENTAL is left out, the caller's too, so `check` chooses: incremental for
`quick`, `commit` and `fences`, not for `full` (R-82). M0's launches set it to 0, which kept
a builder's `check quick` at about 104 s after an edit against 37 s incremental (the process
step after tally 2.3, 2026-11-22; about 2.7 GB of caches a worktree).
The session id is fixed in advance (`--session-id`), so it is in the ids file before the
session starts; `--resume ID` answers that session instead, under the same id.

The model and the effort come from the session's job (`--job J`; `--jobs` prints the
table), which engine/PLAYBOOK.md decides (D2, D3) and guard-test holds equal to its table.
With no `--job` a role runs its default job, on Opus: build the builder, tests T, rules a
rules step, spec the spec writer, lead a coordinator. `--model` and `--effort` override
the job, for a pick the coordinator logs; `--max-usd N` passes Claude Code's
`--max-budget-usd N`, a stop on a runaway session (its own estimate, at list price). The
ids file's line is `<id> <role> <job> <model>`; `usage` reads the id.

It refuses:
  - outside the lead role and app work (the master), which are the coordinators;
  - a role the guard doesn't know, a job that isn't in the table or isn't that role's, a
    model id that isn't in MODELS (the master once sent a stale one, 2026-11-15), a
    missing prompt file, a worktree that isn't a checkout of this repo;
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
# The ids the installed Claude Code knows (2.1.325, checked 2026-11-15); a new model is a
# rules step that adds its id here, its prices to `usage`, and its jobs below.
OPUS, FABLE = "claude-opus-5-5", "claude-fable-5-2"
MODELS = (OPUS, FABLE)
EFFORTS = ("low", "medium", "high", "xhigh", "max")
# job: (role, model, effort), as engine/PLAYBOOK.md's table (D2, D3) has it: Fable for a
# plan's critic, the gate's whole-diff review and a study, where it found what Opus had
# passed (M2's and M3's gate reviews); Opus for the rest, a round's lead, planner, group
# reviews and paper checks included (since 2026-11-22: no gain measured on Fable, and its
# cost doubled the week's). Fable stays at high: Anthropic's guidance starts there, and at
# xhigh it reviews its own work with subagents.
JOBS = {
    "builder": ("build", OPUS, "xhigh"),
    "tests": ("tests", OPUS, "xhigh"),
    "rules-step": ("rules", OPUS, "high"),
    "study": ("rules", FABLE, "high"),
    "spec-writer": ("spec", OPUS, "high"),
    "planner": ("spec", OPUS, "xhigh"),
    "critic": ("spec", FABLE, "high"),
    "paper-check": ("spec", OPUS, "high"),
    "coordinator": ("lead", OPUS, "high"),
    "round-lead": ("lead", OPUS, "high"),
    "group-review": ("lead", OPUS, "high"),
    "fix-review": ("lead", OPUS, "high"),
    "gate": ("lead", OPUS, "xhigh"),
    "gate-review": ("lead", FABLE, "high"),
}
DEFAULT_JOB = {"build": "builder", "tests": "tests", "rules": "rules-step", "spec": "spec-writer", "lead": "coordinator"}
MIN_FREE_GB = 30
ENV = {"CARGO_BUILD_JOBS": "6", "NEXTEST_TEST_THREADS": "6"}
LEFT_TO_CHECK = ("TALLY_ROLE", "CARGO_INCREMENTAL")  # the role is set here; check picks incremental
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
    opts = {"--effort": None, "--model": None, "--resume": None, "--ids": None, "--job": None, "--max-usd": None}
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
        raise Refused("usage: engine/tools/launch ROLE PROMPT-FILE WORKTREE --ids FILE [--job J] [--effort E] [--model M] "
                      "[--max-usd N] [--resume ID] [--dry-run]")
    return words, opts, dry


def settings(role, opts):
    """The job, the model and the effort of this launch, and what it overrides of its job."""
    job = opts["--job"] or DEFAULT_JOB[role]
    if job not in JOBS:
        raise Refused(f"{job!r} isn't a job ({', '.join(JOBS)})")
    job_role, job_model, job_effort = JOBS[job]
    if job_role != role:
        raise Refused(f"the job {job} runs in the {job_role} role, not {role}")
    model, effort = opts["--model"] or job_model, opts["--effort"] or job_effort
    if model not in MODELS:
        raise Refused(f"{model!r} isn't a model id this tool knows ({', '.join(MODELS)}): check the installed Claude Code "
                      "before naming another, and add it in a rules step")
    if effort not in EFFORTS:
        raise Refused(f"{effort!r} isn't an effort ({', '.join(EFFORTS)})")
    over = [f"{what} {mine} for {its}" for what, mine, its in (("model", model, job_model), ("effort", effort, job_effort))
            if mine != its]
    return job, model, effort, over


def max_usd(text):
    try:
        usd = float(text)
    except ValueError:
        usd = 0.0
    if not 0 < usd < 100000:
        raise Refused(f"--max-usd {text!r} isn't an amount in dollars")
    return f"{usd:g}"


def launch(argv, env_in):
    if argv == ["--jobs"]:
        for job, (job_role, model, effort) in JOBS.items():
            print(f"{job} {job_role} {model} {effort}")
        return 0
    words, opts, dry = parse(argv)
    role, prompt_file, worktree = words
    me = env_in.get("TALLY_ROLE") or "unset"
    if me not in ("lead", "unset"):
        raise Refused(f"only a coordinator (the lead role, or app work) launches sessions, and this is {me}")
    if role not in ROLES:
        raise Refused(f"{role!r} isn't a role ({', '.join(ROLES)})")
    job, model, effort, over = settings(role, opts)
    budget = ["--max-budget-usd", max_usd(opts["--max-usd"])] if opts["--max-usd"] else []
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
    ids = os.path.realpath(opts["--ids"])
    out_dir = os.path.join(os.path.dirname(ids), "out")
    claude = ["claude", "-p", prompt, "--model", model, "--effort", effort, "--output-format", "json", *budget,
              "--add-dir", os.path.join(main, "tmp", "active")]
    claude += ["--resume", sid] if opts["--resume"] else ["--session-id", sid]
    argv_ = ["nice", "-n", "10", *claude]
    env = {k: v for k, v in env_in.items() if k not in LEFT_TO_CHECK} | ENV | {"TALLY_ROLE": role}
    jobs = heavy_jobs()
    shown = " ".join(f"{k}={v}" for k, v in {**ENV, "TALLY_ROLE": role}.items())
    print(f"launch: {role} session {sid} in {worktree} (job {job}, {model}, effort {effort}, {free_gb:.0f} GB free)")
    print(f"launch: {shown} nice -n 10 claude -p <{prompt_file}, {len(prompt)} chars> --model {model} "
          f"--effort {effort} --output-format json {' '.join(budget) + ' ' if budget else ''}"
          f"{'--resume' if opts['--resume'] else '--session-id'} {sid} "
          f"--add-dir {os.path.join(main, 'tmp', 'active')} < /dev/null")
    if over:
        print(f"launch: note: {'; '.join(over)}, the job {job}'s in engine/PLAYBOOK.md (D2, D3): log it as a pick")
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
            f.write(f"{sid} {role} {job} {model}\n")
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
