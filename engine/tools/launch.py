"""launch: start one role session for a coordinator, headless in the background or in a
titled Orca terminal (the retro after M0, decision 4; the Orca study, 2026-11-25; the skill
engine-coordinator).

    engine/tools/launch ROLE PROMPT-FILE WORKTREE --ids FILE [--job J] [--effort E]
                        [--model M] [--max-usd N] [--resume ID] [--orca | --plain] [--dry-run]
    engine/tools/launch --exec SPEC-FILE      the Orca terminal's half (launch writes SPEC-FILE)
    engine/tools/launch --jobs

The headless form (`--plain`, the default), from WORKTREE, with this environment:

    CARGO_BUILD_JOBS=6 NEXTEST_TEST_THREADS=6 TALLY_ROLE=ROLE TALLY_TURN_LOG=<turns> \\
      nice -n 10 claude -p "<the prompt file's text>" --model M --effort E \\
      --output-format json --session-id <a new id> --add-dir <main>/tmp/active < /dev/null

detached, its JSON answer in <ids file's folder>/out/<id>.<n>.json and its errors beside it.

The Orca form (`--orca`) opens a terminal tab titled `<run> · <job> · <n>` (the run is the
ids file's folder, n the session's place among that job's lines) in WORKTREE:

    orca terminal create --worktree path:WORKTREE --title "<title>" \\
      --command "<this tool> --exec <ids file's folder>/out/<id>.<n>.spec.json"

and `--exec` sets the same environment and replaces itself with an interactive
`nice -n 10 claude --model M --effort E --add-dir … -n "<title>" --session-id <id> "<prompt>"`.
The study measured 14.4 k more tokens a call for an interactive session (2026-11-25), so
role sessions stay headless unless a brief asks for the Orca form; when `orca status`
fails, `--orca` falls back to the headless form by itself and says so (`TALLY_ORCA` names
another binary, for guard-test). `--max-usd` needs
the headless form: Claude Code's budget stop works only with `-p`.

Either way the disk floor, the job table and the ids line come first, and the Stop hook
appends one line to <turns> (<ids file's folder>/turns.log) when a turn ends, which the
coordinator's Monitor watches (`engine/tools/stop-check`).

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
ids file's line is `<id> <role> <job> <model>`, and the title after them in the Orca form;
`usage` reads the id.

It refuses:
  - outside the lead role and app work (the master), which are the coordinators;
  - a role the guard doesn't know, a job that isn't in the table or isn't that role's, a
    model id that isn't in MODELS (the master once sent a stale one, 2026-11-15), a
    missing prompt file, a worktree that isn't a checkout of this repo;
  - when fewer than 30 GB are free on the worktree's disk (a full disk lost M0 a builder
    turn);
  - `--exec` of a file that isn't a spec this tool wrote: under a run's `out/` folder in
    the main checkout's `tmp/`, a `nice -n 10 claude` command, only the environment above.
It warns when two heavy jobs already run (cargo builds and tests, or launched sessions:
`claude -p`, a terminal titled with a job, or a session the ids file names but a
coordinator's): the Mac takes two at most. `--dry-run` prints the command and the checks,
and starts nothing.
"""

import json
import os
import re
import shlex
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
# the coordinators' own sessions, which aren't heavy jobs
COORDINATORS = ("coordinator", "round-lead")
MIN_FREE_GB = 30
ENV = {"CARGO_BUILD_JOBS": "6", "NEXTEST_TEST_THREADS": "6"}
LEFT_TO_CHECK = ("TALLY_ROLE", "CARGO_INCREMENTAL")  # the role is set here; check picks incremental
TURNS = "TALLY_TURN_LOG"
SPEC_ENV = (*ENV, "TALLY_ROLE", TURNS)
HEAVY = ("cargo build", "cargo test", "cargo nextest", "cargo clippy", "cargo fuzz", "cargo mutants", "cargo-nextest",
         "cargo-mutants", "cargo-fuzz", "claude -p")
# an interactive session launched here: its title `-n <run> · <job> · <n>`, or its id
CLAUDE = re.compile(r"(?:^|/)claude\s")
TITLE = re.compile(r"\s-n\s+\S+ · ([\w-]+) · \S+")
SESSION = re.compile(r"--(?:session-id|resume)\s+([0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12})")
ORCA_TIMEOUT = 30


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


def ids_lines(ids):
    if not os.path.isfile(ids):
        return []
    with open(ids, encoding="utf-8") as f:
        return [line.split() for line in f if line.strip()]


def heavy_jobs(lines=()):
    """The heavy jobs running now: a cargo build or test, a headless session, an interactive
    one by its title or by an id of the ids file that isn't a coordinator's."""
    known = {w[0] for w in lines if len(w) < 3 or w[2] not in COORDINATORS}
    r = subprocess.run(["ps", "-axo", "command"], capture_output=True, text=True, check=False)
    found = []
    for line in r.stdout.splitlines()[1:]:
        cmd = line.strip()
        title = TITLE.search(cmd)
        if any(h in cmd for h in HEAVY) or CLAUDE.search(cmd) and (
                title and title.group(1) in JOBS or any(s in known for s in SESSION.findall(cmd))):
            found.append(cmd[:120])
    return found


def parse(argv):
    opts = {"--effort": None, "--model": None, "--resume": None, "--ids": None, "--job": None, "--max-usd": None}
    flags, words = {"--dry-run": False, "--orca": False, "--plain": False}, []
    it = iter(argv)
    for a in it:
        if a in flags:
            flags[a] = True
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
                      "[--max-usd N] [--resume ID] [--orca | --plain] [--dry-run]")
    if flags["--orca"] and flags["--plain"]:
        raise Refused("--orca and --plain are the two forms: name one")
    if flags["--orca"] and opts["--max-usd"]:
        raise Refused("--max-usd needs the headless form (--plain): Claude Code's --max-budget-usd works only with -p")
    return words, opts, flags


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


def orca_answers(env):
    """Whether Orca's CLI answers (`orca status`), and why not when it doesn't."""
    try:
        r = subprocess.run([env.get("TALLY_ORCA") or "orca", "status", "--json"], capture_output=True, text=True, env=env,
                           timeout=ORCA_TIMEOUT, check=False)
    except (OSError, subprocess.TimeoutExpired) as e:
        return False, str(e)
    try:
        result = json.loads(r.stdout)["result"]
        up = bool(result["app"]["running"]) and bool(result["runtime"]["reachable"])
    except (ValueError, KeyError, TypeError):
        up = False
    return r.returncode == 0 and up, (r.stderr or r.stdout).strip()[:200]


def title_of(ids, lines, job, sid):
    """`<run> · <job> · <n>`: the run is the ids file's folder; n the session's place among
    that job's lines, a new session's the next."""
    run = os.path.basename(os.path.dirname(ids))
    mine = [w[0] for w in lines if len(w) > 2 and w[2] == job]
    n = mine.index(sid) + 1 if sid in mine else len(mine) + 1
    return f"{run} · {job} · {n}"


def launch(argv, env_in):
    if argv == ["--jobs"]:
        for job, (job_role, model, effort) in JOBS.items():
            print(f"{job} {job_role} {model} {effort}")
        return 0
    me = env_in.get("TALLY_ROLE") or "unset"
    if argv[:1] == ["--exec"]:
        if me not in ("lead", "unset"):
            raise Refused(f"only a coordinator (the lead role, or app work) launches sessions, and this is {me}")
        return run_spec(argv[1:], env_in)
    words, opts, flags = parse(argv)
    dry = flags["--dry-run"]
    role, prompt_file, worktree = words
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
    run_dir = os.path.dirname(ids)
    out_dir = os.path.join(run_dir, "out")
    turns = os.path.join(run_dir, "turns.log")
    active = os.path.join(main, "tmp", "active")
    orca = flags["--orca"]
    if orca:
        ok, why = orca_answers(env_in)
        if not ok:
            print(f"launch: `orca status` failed ({why or 'no answer'}): the headless form instead (--plain)")
            orca = False
    lines = ids_lines(ids)
    title = title_of(ids, lines, job, sid) if orca else None
    resume = ["--resume", sid] if opts["--resume"] else ["--session-id", sid]
    if orca:
        claude = ["claude", "--model", model, "--effort", effort, "--add-dir", active, "-n", title, *resume, prompt]
    else:
        claude = ["claude", "-p", prompt, "--model", model, "--effort", effort, "--output-format", "json", *budget,
                  "--add-dir", active, *resume]
    argv_ = ["nice", "-n", "10", *claude]
    set_env = {**ENV, "TALLY_ROLE": role, TURNS: turns}
    env = {k: v for k, v in env_in.items() if k not in LEFT_TO_CHECK} | set_env
    jobs = heavy_jobs(lines)
    shown = " ".join(f"{k}={v}" for k, v in set_env.items())
    print(f"launch: {role} session {sid} in {worktree} (job {job}, {model}, effort {effort}, {free_gb:.0f} GB free)")
    if orca:
        print(f"launch: orca terminal create --worktree path:{worktree} --title {shlex.quote(title)} --command "
              f"\"{os.path.join(TOOLS, 'launch')} --exec <spec>\"; the spec: {shown} nice -n 10 claude --model {model} "
              f"--effort {effort} --add-dir {active} -n {shlex.quote(title)} {' '.join(resume)} "
              f"<{prompt_file}, {len(prompt)} chars>")
    else:
        print(f"launch: {shown} nice -n 10 claude -p <{prompt_file}, {len(prompt)} chars> --model {model} "
              f"--effort {effort} --output-format json {' '.join(budget) + ' ' if budget else ''}"
              f"--add-dir {active} {' '.join(resume)} < /dev/null")
    if over:
        print(f"launch: note: {'; '.join(over)}, the job {job}'s in engine/PLAYBOOK.md (D2, D3): log it as a pick")
    if len(jobs) >= 2:
        print(f"launch: warning: {len(jobs)} heavy jobs already run (the Mac takes two at most): " + "; ".join(jobs[:4]))
    if dry:
        print("launch: dry run: nothing started, the ids file unchanged")
        return 0
    os.makedirs(out_dir, exist_ok=True)
    if sid not in {w[0] for w in lines}:
        with open(ids, "a", encoding="utf-8") as f:
            f.write(f"{sid} {role} {job} {model}" + (f" {title}" if title else "") + "\n")
    stamp = len([n for n in os.listdir(out_dir) if n.startswith(sid) and not n.endswith(".spec.json")])
    out = os.path.join(out_dir, f"{sid}.{stamp}.json")
    if orca:
        return open_terminal(out[:-5] + ".spec.json", worktree, set_env, argv_, title, sid, turns, env_in)
    with open(out, "wb") as o, open(out[:-5] + ".err", "wb") as e, open(os.devnull, "rb") as i:
        p = subprocess.Popen(argv_, cwd=worktree, env=env, stdin=i, stdout=o, stderr=e, start_new_session=True)
    print(json.dumps({"session_id": sid, "pid": p.pid, "out": out, "turns": turns}))
    return 0


def open_terminal(spec, worktree, set_env, argv_, title, sid, turns, env_in):
    with open(spec, "w", encoding="utf-8") as f:
        json.dump({"worktree": worktree, "env": set_env, "argv": argv_}, f, indent=1)
    command = f"{shlex.quote(os.path.join(TOOLS, 'launch'))} --exec {shlex.quote(spec)}"
    try:
        r = subprocess.run([env_in.get("TALLY_ORCA") or "orca", "terminal", "create", "--worktree", f"path:{worktree}", "--title", title,
                            "--command", command, "--json"], capture_output=True, text=True, env=env_in,
                           timeout=ORCA_TIMEOUT, check=False)
    except (OSError, subprocess.TimeoutExpired) as e:
        raise Refused(f"`orca terminal create` failed ({e}); the ids line stands, so `--resume {sid} --plain` starts it headless") from e
    try:
        handle = json.loads(r.stdout)["result"]["terminal"]["handle"]
    except (ValueError, KeyError, TypeError):
        handle = None
    if r.returncode != 0 or not handle:
        raise Refused(f"`orca terminal create` failed (exit {r.returncode}: {(r.stderr or r.stdout).strip()[:200]}); "
                      f"the ids line stands, so `--resume {sid} --plain` starts it headless")
    print(json.dumps({"session_id": sid, "terminal": handle, "title": title, "spec": spec, "turns": turns}))
    return 0


def run_spec(args, env_in):
    """The Orca terminal's half: check the spec, set its environment, become the session."""
    if len(args) != 1:
        raise Refused("usage: engine/tools/launch --exec SPEC-FILE")
    spec = os.path.realpath(args[0])
    parts = spec.split(os.sep)
    if not spec.endswith(".spec.json") or len(parts) < 5 or parts[-2] != "out" or parts[-4] != "tmp":
        raise Refused(f"{spec} isn't a spec this tool writes (tmp/<run>/out/<id>.<n>.spec.json)")
    main = os.sep.join(parts[:-4])
    if main_checkout(main) != main:
        raise Refused(f"{spec} isn't in the main checkout's tmp/")
    try:
        with open(spec, encoding="utf-8") as f:
            data = json.load(f)
        worktree, set_env, argv_ = data["worktree"], data["env"], data["argv"]
    except (OSError, ValueError, KeyError, TypeError) as e:
        raise Refused(f"{spec} can't be read as a spec ({e})") from e
    if not (isinstance(argv_, list) and all(isinstance(a, str) for a in argv_) and argv_[:4] == ["nice", "-n", "10", "claude"]
            and isinstance(set_env, dict) and set(set_env) == set(SPEC_ENV) and set_env.get("TALLY_ROLE") in ROLES
            and all(isinstance(v, str) for v in set_env.values())):
        raise Refused(f"{spec} isn't a spec this tool writes: a `nice -n 10 claude` command and {', '.join(SPEC_ENV)} only")
    worktree = os.path.realpath(worktree)
    if main_checkout(worktree) != main:
        raise Refused(f"{worktree} isn't a checkout of this repo")
    env = {k: v for k, v in env_in.items() if k not in LEFT_TO_CHECK} | set_env
    os.chdir(worktree)
    os.execvpe("nice", argv_, env)
    return 1  # not reached: exec replaces this process


def main(argv):
    try:
        return launch(argv, dict(os.environ))
    except (Refused, OSError) as r:
        print(f"launch: refused: {r}.", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
