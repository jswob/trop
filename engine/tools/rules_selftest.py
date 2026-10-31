"""rules-selftest: every tool-enforced rule fires on a planted violation (R-78).

    engine/tools/rules-selftest/run              everything
    engine/tools/rules-selftest/run R-5 R-62     the clean run, the ban coverage and these rules' cases
    engine/tools/rules-selftest/run --list       what proves each rule

It copies the engine into a scratch repository, engine/target/rules-selftest/repo/ (a
git repository of its own, holding the spec's tag), so every tool runs there as it runs
here. Then:

  1. the clean run: the engine as it is passes every tool; the clippy and cargo output
     holds none of the messages that mean a rule silently does nothing; ast-grep has
     every rule on; the CI workflow runs the full check;
  2. the ban coverage: one planted use of every entry in clippy.toml's three ban lists,
     each of which must fail with its own lint on its own line;
  3. the cases, tools/rules-selftest/cases/R-<n>.case: each plants one violation, runs
     the tool that enforces the rule and checks that it fails with that rule's name
     (the lint code, the ast-grep rule id, the cargo-deny code or the script's message).
     The scratch is put back after each.

Cases that only add Rust files are linted together, one cargo run per package and pass;
a case that misses there runs again alone before it counts as a failure. Nothing needs
the network after the first `cargo fetch`: the crates the cases plant are fakes, built
in engine/target/rules-selftest/fakes/.

A case file holds one or more cases:

    case: <name>
    tool: clippy | clippy-tests | rustdoc | ast-grep | deny | deny-tally | run
    expect: <lint code, ast-grep id, cargo-deny code, or text of the run's output>
    text: <text the matching diagnostic must hold>       (optional)
    at: <file the diagnostic must be in>                 (optional; default the case's files)
    solo: yes                                            (optional; never batched)
    exit: any                                            (run: the command may pass)
    fetch: yes                                           (fetch a planted git dependency first)
    --- module [package]        a new module of the package (default tally)
    --- write <path>            a file, relative to engine/ (`../` reaches the repository root;
                                `@FAKES@` is the fakes folder)
    --- append <path>
    --- replace <path>          the text to replace, then `--- with` and its replacement
    --- delete <path>
    --- fake <name> <version>   a fake crate; the body is extra manifest text
    --- fakefile <crate-dir>/<path> [exec]
    --- fakebinary <crate-dir>/<path>   a copy of a native program
    --- gitfake <name> <version>        a fake crate in a git repository of its own
    --- run                     the shell commands of a `run` case, from engine/

`expect: pass` asks for no diagnostic at all in the case's files: a rule's exemption.
"""

import dataclasses
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import time
import tomllib

TOOLS = os.path.dirname(os.path.realpath(__file__))
ENGINE = os.path.dirname(TOOLS)
ROOT = os.path.dirname(ENGINE)
sys.path.insert(0, TOOLS)
import check  # noqa: E402  the same flags and environment as engine/tools/check

CASES = os.path.join(TOOLS, "rules-selftest", "cases")
SCRATCH = os.path.join(ENGINE, "target", "rules-selftest")
REPO = os.path.join(SCRATCH, "repo")
WS = os.path.join(REPO, "engine")
FAKES = os.path.join(SCRATCH, "fakes")
TARGET = os.path.join(SCRATCH, "target")
GIT_HOME = os.path.join(SCRATCH, "cargo-home")
LOGS = os.path.join(SCRATCH, "logs")
COPY_SKIP = {"target", "__pycache__", ".DS_Store", "corpus", "artifacts", "coverage", "mutants.out"}
OUTSIDE = ["package.json", ".github/workflows/engine.yml", ".claude/settings.json"]
COVERAGE_FILE = "crates/tally/src/zz_ban_coverage.rs"
TOOL_NAMES = {"clippy", "clippy-tests", "rustdoc", "ast-grep", "deny", "deny-tally", "run"}
BLOCKS = {"module", "write", "append", "replace", "delete", "fake", "fakefile", "fakebinary", "gitfake", "run"}

# What proves each rule that has no case (engine/SCAFFOLD.md has the full map)
ELSEWHERE = {
    "R-53": "cargo-deny's advisories, daily in CI: a planted advisory needs the RustSec database, so there is no offline case",
    "R-74": "the advisory report (engine/tools/advisory), which never fails",
    "R-75": "the advisory report (engine/tools/advisory), which never fails",
    "R-77": "the clean run: the workflow runs `engine/tools/check full`, whose steps are R-77's list",
    "R-78": "this script",
    "R-79": "engine/tools/guard-test",
    "R-81": "engine/tools/guard-test",
    "R-83": "engine/tools/guard-test (the Stop hook)",
    "R-84": "the protected mutants config (test_package); checksummed",
    "R-88": "the protected mutants and nextest configs; checksummed",
    "R-91": "engine/tools/guard-test",
    "R-92": "check full's fuzz step and the workflow's long fuzz job",
    "R-94": "the special-casing report (engine/tools/special-casing), which never fails",
    "R-95": "the workflow's nightly jobs",
    "R-96": "the workflow's nightly coverage job",
    "R-97": "the workflow's nightly benchmark job",
    "R-98": "the workflow's daily advisories and weekly beta-clippy jobs",
    "R-100": "engine/tools/guard-test (the commit tripwire)",
    "R-112": "a procedure: the guard (guard-test), RULES.sha256 (R-80's cases) and this script",
}
REVIEW = {"R-28", "R-31", "R-40", "R-47", "R-63", "R-76", "R-99", "R-113"}
DROPPED = {"R-17", "R-32", "R-33", "R-34", "R-35", "R-101", "R-102"}
# rules the ban coverage proves on its own
BY_COVERAGE = {"R-25", "R-26", "R-46"}
# the steps R-77 asks of every pull request
PR_STEPS = [
    "fmt", "clippy-lib", "clippy-tests", "doc", "build", "test-dev", "test-release", "deny", "deny-tally",
    "check-frozen", "spec-loadcheck", "guard-test", "rules-selftest", "check-rules", "exceptions",
    "ast-grep-test", "ast-grep-scan", "fuzz", "fault-injection-probe",
]


class CaseError(Exception):
    pass


@dataclasses.dataclass
class Case:
    rule: str
    name: str
    line: int
    tool: str = ""
    expect: list = dataclasses.field(default_factory=list)
    text: str = ""
    at: list = dataclasses.field(default_factory=list)
    solo: bool = False
    exit_any: bool = False
    fetch: bool = False
    blocks: list = dataclasses.field(default_factory=list)

    @property
    def id(self):
        return f"{self.rule}/{self.name}"


@dataclasses.dataclass
class Diag:
    code: str
    level: str
    file: str
    line: int
    text: str


# --- the cases ---------------------------------------------------------------------


def parse_cases(path):
    m = re.fullmatch(r"R-(\d+)\.case", os.path.basename(path))
    if not m:
        raise CaseError(f"{os.path.basename(path)}: a case file is named R-<n>.case")
    rule, where = f"R-{int(m.group(1))}", os.path.relpath(path, ENGINE)
    cases, case, block = [], None, None
    with open(path, encoding="utf-8") as f:
        lines = f.read().split("\n")
    for n, line in enumerate(lines, 1):
        if line.startswith("case: "):
            case, block = Case(rule, line[6:].strip(), n), None
            cases.append(case)
        elif case is None:
            if line.strip() and not line.startswith("#"):
                raise CaseError(f"{where}:{n}: text before the first `case:`")
        elif line.startswith("--- "):
            words = line[4:].split()
            if words == ["with"]:
                if block is None or block[0] != "replace" or block[3] is not None:
                    raise CaseError(f"{where}:{n}: `--- with` belongs after `--- replace`")
                block[3] = []
            elif words and words[0] in BLOCKS:
                block = [words[0], words[1:], [], None]
                case.blocks.append(block)
            else:
                raise CaseError(f"{where}:{n}: unknown block {line!r}")
        elif block is None:
            if not line.strip() or line.startswith("# "):
                continue
            key, sep, value = line.partition(": ")
            if not sep:
                raise CaseError(f"{where}:{n}: a header is `key: value`")
            if key == "expect":
                case.expect.append(value)
            elif key == "at":
                case.at.append(value)
            elif key in ("tool", "text"):
                setattr(case, key, value)
            elif (key, value) in (("solo", "yes"), ("fetch", "yes"), ("exit", "any")):
                setattr(case, {"exit": "exit_any"}.get(key, key), True)
            else:
                raise CaseError(f"{where}:{n}: unknown header {line!r}")
        else:
            (block[3] if block[3] is not None else block[2]).append(line)
    for c in cases:
        if c.tool not in TOOL_NAMES:
            raise CaseError(f"{where}:{c.line}: case {c.name!r} has no known tool")
        if not c.expect:
            raise CaseError(f"{where}:{c.line}: case {c.name!r} expects nothing")
        if c.tool == "run" and not any(b[0] == "run" for b in c.blocks):
            raise CaseError(f"{where}:{c.line}: case {c.name!r} is a run case with no `--- run`")
        for b in c.blocks:
            if b[0] == "replace" and b[3] is None:
                raise CaseError(f"{where}:{c.line}: case {c.name!r}: a replace without `--- with`")
    names = [c.name for c in cases]
    if len(set(names)) != len(names):
        raise CaseError(f"{where}: two cases share a name")
    return cases


def load_cases():
    cases = []
    for name in sorted(os.listdir(CASES), key=lambda n: (len(n), n)):
        if name.endswith(".case"):
            cases += parse_cases(os.path.join(CASES, name))
    return cases


def body(lines):
    """A block's text: its lines without the blank ones at either end."""
    while lines and not lines[0].strip():
        lines = lines[1:]
    while lines and not lines[-1].strip():
        lines = lines[:-1]
    return sub("\n".join(lines))


def sub(text):
    return text.replace("@FAKES@", FAKES).replace("@WS@", WS)


# --- the scratch repository ----------------------------------------------------------


def git_env():
    env = dict(os.environ)
    env.update({"GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": os.devnull})
    return env


def git(*args, cwd=REPO):
    argv = ["git", "-c", "user.name=rules-selftest", "-c", "user.email=rules-selftest@invalid",
            "-c", "commit.gpgsign=false", "-c", "core.hooksPath=/dev/null", *args]
    r = subprocess.run(argv, cwd=cwd, env=git_env(), capture_output=True, text=True, check=False)
    if r.returncode != 0:
        raise RuntimeError(f"git {' '.join(args)}: {r.stderr.strip()}")
    return r.stdout


def copy_ignore(_dir, names):
    return [n for n in names if n in COPY_SKIP]


def pin_the_tag():
    """The scratch's one commit carries the spec's tag, but tools/frozen.toml may pin
    a later commit only the real history has: the scratch pins the tag instead, and its
    RULES.sha256 line for frozen.toml follows."""
    frozen = os.path.join(WS, "tools", "frozen.toml")
    with open(frozen, encoding="utf-8") as f:
        lines = f.readlines()
    with open(frozen, "w", encoding="utf-8") as f:
        f.writelines(line for line in lines if not line.startswith("pin = "))
    with open(frozen, "rb") as f:
        digest = hashlib.sha256(f.read()).hexdigest()
    sums = os.path.join(WS, "tools", "RULES.sha256")
    with open(sums, encoding="utf-8") as f:
        lines = f.readlines()
    with open(sums, "w", encoding="utf-8") as f:
        for line in lines:
            f.write(f"{digest}  engine/tools/frozen.toml\n" if line.endswith("  engine/tools/frozen.toml\n") else line)


def build_scratch():
    shutil.rmtree(REPO, ignore_errors=True)
    shutil.rmtree(FAKES, ignore_errors=True)
    shutil.copytree(ENGINE, WS, ignore=copy_ignore, symlinks=True)
    pin_the_tag()
    for p in OUTSIDE:
        if os.path.isfile(os.path.join(ROOT, p)):
            os.makedirs(os.path.dirname(os.path.join(REPO, p)) or REPO, exist_ok=True)
            shutil.copy2(os.path.join(ROOT, p), os.path.join(REPO, p))
    git("init", "-q")
    git("add", "-A")
    git("commit", "-qm", "the engine, copied for rules-selftest")
    with open(os.path.join(WS, "spec", "VERSION"), encoding="utf-8") as f:
        git("tag", f.read().strip())
    os.makedirs(GIT_HOME, exist_ok=True)
    registry = os.path.join(os.environ.get("CARGO_HOME") or os.path.join(os.path.expanduser("~"), ".cargo"), "registry")
    link = os.path.join(GIT_HOME, "registry")
    if not os.path.lexists(link) and os.path.isdir(registry):
        os.symlink(registry, link)


def reset():
    git("reset", "-q", "--hard")
    git("clean", "-qfdx")
    shutil.rmtree(FAKES, ignore_errors=True)


def ws_path(rel):
    path = os.path.normpath(os.path.join(WS, sub(rel)))
    if not (path.startswith(REPO + os.sep) or path.startswith(FAKES + os.sep)):
        raise CaseError(f"{rel}: outside the scratch")
    return path


def write(rel, text, mode="w"):
    path = ws_path(rel)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, mode, encoding="utf-8") as f:
        f.write(text if text.endswith("\n") or not text else text + "\n")


def make_fake(name, version, extra, git_repo=False):
    d = os.path.join(FAKES, f"{name}-{version}")
    os.makedirs(os.path.join(d, "src"), exist_ok=True)
    manifest = f'[package]\nname = "{name}"\nversion = "{version}"\nedition = "2024"\n'
    if not re.search(r"^license\s*=", extra, re.M):
        manifest += 'license = "MIT"\n'
    with open(os.path.join(d, "Cargo.toml"), "w", encoding="utf-8") as f:
        f.write(manifest + (extra + "\n" if extra else ""))
    lib = os.path.join(d, "src", "lib.rs")
    if not os.path.exists(lib):
        with open(lib, "w", encoding="utf-8") as f:
            f.write("//! A rules-selftest fake.\n")
    if git_repo:
        git("init", "-q", cwd=d)
        git("add", "-A", cwd=d)
        git("commit", "-qm", "a rules-selftest fake", cwd=d)
    return d


def plant(case):
    """Writes the case into the scratch; returns the engine-relative files it touched."""
    touched, modules = [], 0
    for kind, args, lines, with_lines in case.blocks:
        text = body(lines)
        if kind == "module":
            pkg = args[0] if args else "tally"
            modules += 1
            slug = "zz_" + re.sub(r"[^a-z0-9]+", "_", case.id.lower()).strip("_") + (f"_{modules}" if modules > 1 else "")
            rel = f"crates/{pkg}/src/{slug}.rs"
            write(rel, text)
            write(f"crates/{pkg}/src/lib.rs", f"\n/// A planted rules-selftest case.\npub mod {slug};\n", "a")
            touched.append(rel)
        elif kind in ("write", "append"):
            write(args[0], text, "w" if kind == "write" else "a")
            touched.append(sub(args[0]))
        elif kind == "replace":
            path = ws_path(args[0])
            with open(path, encoding="utf-8") as f:
                old_text = f.read()
            old, new = text, body(with_lines)
            if old_text.count(old) != 1:
                raise CaseError(f"{case.id}: the text to replace is in {args[0]} {old_text.count(old)} times, not once")
            with open(path, "w", encoding="utf-8") as f:
                f.write(old_text.replace(old, new))
            touched.append(args[0])
        elif kind == "delete":
            os.remove(ws_path(args[0]))
            touched.append(args[0])
        elif kind in ("fake", "gitfake"):
            make_fake(args[0], args[1], text, git_repo=kind == "gitfake")
        elif kind == "fakefile":
            path = os.path.join(FAKES, args[0])
            os.makedirs(os.path.dirname(path), exist_ok=True)
            with open(path, "w", encoding="utf-8") as f:
                f.write(text + "\n")
            if "exec" in args[1:]:
                os.chmod(path, 0o755)
        elif kind == "fakebinary":
            path = os.path.join(FAKES, args[0])
            os.makedirs(os.path.dirname(path), exist_ok=True)
            shutil.copy(os.path.realpath(shutil.which("true") or "/usr/bin/true"), path)
            os.chmod(path, 0o755)
    return touched


# --- running the tools ---------------------------------------------------------------


def env(fetching=False):
    e = check.clean_env()
    e["CARGO_TARGET_DIR"] = TARGET
    if fetching:
        e["CARGO_HOME"] = GIT_HOME
    else:
        e["CARGO_NET_OFFLINE"] = "true"
    return e


def run(argv, log, e=None, shell=False):
    began = time.monotonic()
    r = subprocess.run(argv, cwd=WS, env=e or env(), capture_output=True, text=True, check=False, shell=shell,
                       executable="/bin/bash" if shell else None)
    shown = argv if isinstance(argv, str) else " ".join(argv)
    log.write(f"$ {shown}\n--- stdout\n{r.stdout}\n--- stderr\n{r.stderr}\n[exit {r.returncode}, {time.monotonic() - began:.0f}s]\n\n")
    return r


def rel_file(name):
    if os.path.isabs(name):
        name = os.path.relpath(name, WS)
    return name.replace(os.sep, "/")


def cargo_diags(stdout):
    diags = []
    for line in stdout.splitlines():
        if not line.startswith("{"):
            continue
        try:
            m = json.loads(line)
        except ValueError:
            continue
        if m.get("reason") != "compiler-message":
            continue
        msg = m["message"]
        code = (msg.get("code") or {}).get("code") or ""
        primary = [s for s in msg.get("spans", []) if s.get("is_primary")]
        file = rel_file(primary[0]["file_name"]) if primary else ""
        diags.append(Diag(code, msg.get("level", ""), file, primary[0]["line_start"] if primary else 0,
                          msg.get("rendered") or msg.get("message", "")))
    return diags


def deny_diags(stderr):
    diags = []
    for line in stderr.splitlines():
        try:
            m = json.loads(line)
        except ValueError:
            continue
        if m.get("type") != "diagnostic":
            continue
        f = m.get("fields", {})
        labels = " ".join(f"{x.get('span', '')} {x.get('message', '')}" for x in f.get("labels", []))
        diags.append(Diag(f.get("code") or "", f.get("severity", ""), "", 0, f"{f.get('message', '')} {labels}"))
    return diags


def astgrep_diags(stdout):
    diags = []
    for line in stdout.splitlines():
        try:
            m = json.loads(line)
        except ValueError:
            continue
        diags.append(Diag(m.get("ruleId", ""), m.get("severity", ""), rel_file(m.get("file", "")),
                          m.get("range", {}).get("start", {}).get("line", -1) + 1, m.get("message", "")))
    return diags


def json_cargo(argv):
    """`argv` with JSON messages, without --locked (the cases change the lock), offline."""
    argv = [a for a in argv if a != "--locked"]
    cut = argv.index("--") if "--" in argv else len(argv)
    return argv[:cut] + ["--offline", "--message-format=json", "--keep-going"] + argv[cut:]


def run_tool(tool, log, files=(), script=None, fetching=False):
    """Runs one tool in the scratch: (diagnostics, exit code, all output)."""
    e = env(fetching)
    if fetching:
        run(["cargo", "fetch"], log, e)
    if tool in ("clippy", "clippy-tests"):
        r = run(json_cargo(check.CLIPPY_TESTS if tool == "clippy-tests" else check.CLIPPY_LIB), log, e)
        return cargo_diags(r.stdout), r.returncode, r.stdout + r.stderr
    if tool == "rustdoc":
        e["RUSTDOCFLAGS"] = "-D warnings"
        r = run(json_cargo(["cargo", "doc", "--workspace", "--no-deps", "--all-features"]), log, e)
        return cargo_diags(r.stdout), r.returncode, r.stdout + r.stderr
    if tool == "ast-grep":
        r = run(["ast-grep", "scan", "--json=stream", *files], log, e)
        return astgrep_diags(r.stdout), r.returncode, r.stdout + r.stderr
    if tool == "deny":
        r = run(["cargo", "deny", "--offline", "--format", "json", "check", "bans", "licenses", "sources"], log, e)
        return deny_diags(r.stderr), r.returncode, r.stderr
    if tool == "deny-tally":
        r = run(["cargo", "deny", "--offline", "--format", "json", "--manifest-path", "crates/tally/Cargo.toml",
                 "--config", "deny-tally.toml", "check", "bans"], log, e)
        return deny_diags(r.stderr), r.returncode, r.stderr
    if tool == "run":
        r = run(script, log, e, shell=True)
        return [], r.returncode, r.stdout + r.stderr
    raise CaseError(f"unknown tool {tool}")


def judge(case, diags, code, out, touched):
    """What is wrong with the case's outcome; empty when the rule fired as it should."""
    problems = []
    if case.tool == "run":
        for e in case.expect:
            if e not in out:
                problems.append(f"the output lacks {e!r}")
        if not case.exit_any and code == 0:
            problems.append("the command passed")
        return problems
    files = case.at or [t for t in touched if t.endswith(".rs")]
    near = diags if case.tool in ("deny", "deny-tally") or not files or "*" in files else [d for d in diags if d.file in files]
    for e in case.expect:
        if e == "pass":
            if near:
                problems.append(f"expected no diagnostic, got {near[0].code or near[0].text[:100]!r}")
            continue
        if not [d for d in near if d.code == e and (not case.text or case.text in d.text)]:
            got = sorted({d.code or "(no code)" for d in near})
            problems.append(f"no {e}" + (f" naming {case.text!r}" if case.text else "") + f"; got {', '.join(got) or 'nothing'}")
    return problems


# --- the ban coverage ----------------------------------------------------------------

BUILTIN = {"std", "core", "alloc", "slice", "str"} | {"i8", "i16", "i32", "i64", "i128", "isize",
                                                      "u8", "u16", "u32", "u64", "u128", "usize", "f32", "f64"}
INTS = {"i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize"}
# how to name an entry's function without calling it: {m} is the method
OWNERS = {
    "std::vec::Vec": "Vec::<u8>::{m}",
    "std::string::String": "String::{m}",
    "slice": "<[u8]>::{m}",
    "str": "str::{m}",
    "std::iter::Iterator": "<core::ops::Range<u8> as Iterator>::{m}",
    "std::cmp::Ord": "<u8 as Ord>::{m}",
    "std::cell::RefCell": "core::cell::RefCell::<u8>::{m}",
    "num_integer::Integer": "<i64 as num_integer::Integer>::{m}",
    "num_bigint::BigInt": "num_bigint::BigInt::{m}",
    "num_bigint::BigUint": "num_bigint::BigUint::{m}",
    "std::panic": "std::panic::{m}",
    "std::process": "std::process::{m}",
    "std::env": "std::env::{m}",
    "std::time::Duration": "core::time::Duration::{m}",
    "num_traits::ToPrimitive": "<u8 as num_traits::ToPrimitive>::{m}",
    "num_traits::FromPrimitive": "<u8 as num_traits::FromPrimitive>::{m}",
    "num_traits::AsPrimitive": "<u8 as num_traits::AsPrimitive<u16>>::{m}",
    "num_traits::Pow": "<u8 as num_traits::Pow<u32>>::{m}",
    "num_traits": "num_traits::{m}",
    "std::option::Option": "Option::<u8>::{m}",
    "std::result::Result": "Result::<u8, u8>::{m}",
    "std::time::SystemTime": "std::time::SystemTime::{m}",
    "std::time::Instant": "std::time::Instant::{m}",
    "std::thread": "std::thread::{m}",
    "std::thread::Builder": "std::thread::Builder::{m}",
    "std::sync::mpsc": "std::sync::mpsc::{m}",
    "std::fs": "std::fs::{m}",
    "std::fs::File": "std::fs::File::{m}",
    "std::path::Path": "std::path::Path::{m}",
    "std::io": "std::io::{m}",
    "std::ffi::OsStr": "std::ffi::OsStr::{m}",
}
OPS_RHS = {"Shl": "<u32>", "Shr": "<u32>", "ShlAssign": "<u32>", "ShrAssign": "<u32>"}
SORT_BY = "::<fn(&u8, &u8) -> core::cmp::Ordering>"
SORT_BY_KEY = "::<u8, fn(&u8) -> u8>"
TURBOFISH = {
    "slice::sort_unstable_by": SORT_BY, "slice::select_nth_unstable_by": SORT_BY,
    "slice::sort_unstable_by_key": SORT_BY_KEY, "slice::select_nth_unstable_by_key": SORT_BY_KEY,
    "std::vec::Vec::drain": "::<core::ops::RangeFull>", "std::string::String::drain": "::<core::ops::RangeFull>",
    "std::string::String::replace_range": "::<core::ops::RangeFull>", "slice::copy_within": "::<core::ops::RangeFull>",
    "std::iter::Iterator::sum": "::<u8>", "std::iter::Iterator::product": "::<u8>",
    "std::panic::catch_unwind": "::<fn() -> u8, u8>",
    "std::env::var": "::<&str>", "std::env::var_os": "::<&str>", "std::env::set_current_dir": "::<&str>",
    "std::thread::spawn": "::<fn() -> u8, u8>", "std::thread::Builder::spawn": "::<fn() -> u8, u8>",
    "std::fs::write": "::<&str, &[u8]>", "std::fs::copy": "::<&str, &str>", "std::fs::rename": "::<&str, &str>",
    "num_traits::pow": "::<u8>",
    "std::sync::mpsc::channel": "::<u8>", "std::sync::mpsc::sync_channel": "::<u8>",
    **{f"std::fs::{m}": "::<&str>" for m in (
        "read", "read_to_string", "read_dir", "metadata", "symlink_metadata", "canonicalize", "create_dir",
        "create_dir_all", "remove_file", "remove_dir", "remove_dir_all", "read_link", "exists")},
    **{f"std::fs::File::{m}": "::<&str>" for m in ("open", "create", "create_new")},
}
# entries a path can't name without a closure's type: planted as a call
CALLS = {
    "std::thread::scope": "pub fn planted_scope() { std::thread::scope(|_scope| {}); }",
    "std::thread::Builder::spawn_scoped":
        "pub fn planted_spawn_scoped<'scope>(scope: &'scope std::thread::Scope<'scope, '_>) { let _handle = std::thread::Builder::new().spawn_scoped(scope, || {}); }",
    "std::iter::Sum::sum": "pub fn planted_sum() -> u8 { <u8 as std::iter::Sum>::sum(core::iter::empty::<u8>()) }",
    "std::iter::Product::product": "pub fn planted_product() -> u8 { <u8 as std::iter::Product>::product(core::iter::empty::<u8>()) }",
    "std::ops::Index::index": "pub fn planted_index(v: &[u8]) -> &u8 { std::ops::Index::index(v, 0) }",
    "std::ops::IndexMut::index_mut": "pub fn planted_index_mut(v: &mut [u8]) -> &mut u8 { std::ops::IndexMut::index_mut(v, 0) }",
    "std::fmt::Debug::fmt":
        "pub fn planted_debug_fmt(v: &u8, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { std::fmt::Debug::fmt(v, f) }",
    "std::thread::Scope::spawn":
        "pub fn planted_scope_spawn<'scope>(scope: &'scope std::thread::Scope<'scope, '_>) { let _handle = scope.spawn(|| {}); }",
}
TYPE_ARGS = {
    "std::collections::HashMap": "<u8, u8>", "std::collections::HashSet": "<u8>",
    "std::num::Wrapping": "<u8>", "std::num::Saturating": "<u8>",
    "serde_json::Map": "<String, serde_json::Value>",
    "std::sync::Mutex": "<u8>", "std::sync::RwLock": "<u8>", "std::sync::mpsc::Sender": "<u8>",
    "std::sync::mpsc::SyncSender": "<u8>", "std::sync::mpsc::Receiver": "<u8>", "std::sync::atomic::AtomicPtr": "<u8>",
}
MACRO_ARGS = {"assert": '(core::hint::black_box(true), "planted")', "debug_assert": '(core::hint::black_box(true), "planted")'}
EQ_ARGS = '(core::hint::black_box(1_u8), 1, "planted")'


def crate_of(path):
    return path.split("::", 1)[0]


def method_expr(path, fakes):
    owner, _, m = path.rpartition("::")
    if crate_of(path) in fakes:
        return path
    if owner in INTS:
        return f"{owner}::{m}"
    if owner.startswith("std::ops::"):
        trait = owner.rsplit("::", 1)[1]
        return f"<{'i8' if trait == 'Neg' else 'u8'} as {owner}{OPS_RHS.get(trait, '')}>::{m}"
    if owner not in OWNERS:
        return None
    return OWNERS[owner].format(m=m) + TURBOFISH.get(path, "")


def fake_source(paths):
    """A fake crate's lib.rs that holds every item the ban paths name."""
    tree = {}
    for kind, path in paths:
        node = tree
        segs = path.split("::")[1:]
        for i, seg in enumerate(segs):
            last = i == len(segs) - 1
            if seg[:1].isupper() and not last:
                node = node.setdefault(("type", seg), {})
            elif last:
                node[("fn" if kind == "method" else "type", seg)] = {}
            else:
                node = node.setdefault(("mod", seg), {})

    def emit(node, owner=None, indent=""):
        out = []
        for (kind, name), child in sorted(node.items()):
            if kind == "mod":
                out += [f"{indent}pub mod {name} {{", *emit(child, None, indent + "    "), f"{indent}}}"]
            elif kind == "type":
                out.append(f"{indent}pub struct {name};")
                fns = [n for (k, n) in child if k == "fn"]
                if fns:
                    out += [f"{indent}impl {name} {{", *[f"{indent}    pub fn {f}() {{}}" for f in sorted(fns)], f"{indent}}}"]
            elif kind == "fn" and owner is None:
                out.append(f"{indent}pub fn {name}() {{}}")
        return out

    return "//! A rules-selftest fake: the items engine/clippy.toml bans, empty.\n" + "\n".join(emit(tree)) + "\n"


def plant_coverage():
    """Writes the coverage module; returns [(line, lint, entry, rule)] and the problems."""
    cfg = tomllib.load(open(os.path.join(WS, "clippy.toml"), "rb"))
    lock = tomllib.load(open(os.path.join(WS, "Cargo.lock"), "rb"))
    manifest_path = os.path.join(WS, "crates", "tally", "Cargo.toml")
    deps = {k.replace("-", "_") for k in tomllib.load(open(manifest_path, "rb")).get("dependencies", {})}
    locked = {}
    for p in lock.get("package", []):
        locked.setdefault(p["name"].replace("-", "_"), (p["name"], p["version"]))
    entries = [(kind, e if isinstance(e, str) else e["path"], "" if isinstance(e, str) else e.get("reason", ""))
               for kind, key in (("method", "disallowed-methods"), ("type", "disallowed-types"), ("macro", "disallowed-macros"))
               for e in cfg.get(key, [])]
    problems, fakes, extra_deps = [], {}, []
    for crate in sorted({crate_of(p) for _, p, _ in entries} - BUILTIN - deps):
        if crate in locked:
            name, version = locked[crate]
            extra_deps.append(f'{name} = "={version}"')
        else:
            fakes[crate] = [(k, p) for k, p, _ in entries if crate_of(p) == crate]
    for crate, paths in fakes.items():
        d = make_fake(crate, "0.0.1", "")
        with open(os.path.join(d, "src", "lib.rs"), "w", encoding="utf-8") as f:
            f.write(fake_source(paths))
        extra_deps.append(f'{crate} = {{ path = "{d}", version = "0.0.1" }}')
    if extra_deps:
        with open(manifest_path, encoding="utf-8") as f:
            text = f.read()
        with open(manifest_path, "w", encoding="utf-8") as f:
            f.write(text.replace("[dependencies]\n", "[dependencies]\n" + "\n".join(extra_deps) + "\n", 1))

    lines = ["//! The ban coverage: one planted use of every clippy.toml ban entry, each on its own line.", ""]
    planted = []

    def add(text, kind, path, reason):
        lines.append(text)
        planted.append((len(lines), {"method": "clippy::disallowed_methods", "type": "clippy::disallowed_types",
                                     "macro": "clippy::disallowed_macros"}[kind], path, reason))

    lines.append("pub fn planted_methods() {")
    calls = []
    for kind, path, reason in entries:
        if kind != "method":
            continue
        if path in CALLS:
            calls.append((path, reason))
            continue
        expr = method_expr(path, fakes)
        if expr is None:
            problems.append(f"ban coverage: no template names {path}; add its owner to rules_selftest.py's OWNERS")
            continue
        add(f"    core::hint::black_box({expr});", kind, path, reason)
    lines.append("}")
    for path, reason in calls:
        add(CALLS[path], "method", path, reason)
    lines.append("pub struct PlantedTypes {")
    for i, (kind, path, reason) in enumerate(e for e in entries if e[0] == "type"):
        args = "" if crate_of(path) in fakes else TYPE_ARGS.get(path, "")
        add(f"    pub t{i}: Option<{path if '::' in path else path}{args}>,", kind, path, reason)
    lines.append("}")
    lines.append("pub fn planted_macros() {")
    for kind, path, reason in entries:
        if kind != "macro":
            continue
        name = path.rsplit("::", 1)[1]
        args = MACRO_ARGS.get(name) or (EQ_ARGS if name.endswith(("_eq", "_ne")) else None)
        if args is None or crate_of(path) not in BUILTIN:
            problems.append(f"ban coverage: no way to plant the macro {path}; add it to rules_selftest.py's MACRO_ARGS")
            continue
        add(f"    {name}!{args};", kind, path, reason)
    lines.append("}")
    write(COVERAGE_FILE, "\n".join(lines))
    slug = os.path.basename(COVERAGE_FILE)[:-3]
    write("crates/tally/src/lib.rs", f"\n/// The ban coverage.\npub mod {slug};\n", "a")
    return planted, problems, sorted(fakes)


def judge_coverage(planted, diags):
    problems = []
    hits = {(d.code, d.line) for d in diags if d.file == COVERAGE_FILE}
    # a compile error, not a lint (borrowck's have no code), stops every lint in the batch
    broken = [d for d in diags if d.level == "error" and (not d.code or re.fullmatch(r"E\d{4}", d.code))]
    for d in broken:
        what = d.text.splitlines()[0] if d.text else d.code
        if d.file == COVERAGE_FILE:
            problems.append(f"ban coverage: line {d.line} doesn't compile ({what}): "
                            "the ban names something the pinned toolchain or crate doesn't have")
        else:
            problems.append(f"ban coverage: {d.file}:{d.line} doesn't compile ({what}), so no lint ran in the batch")
    for line, lint, path, reason in planted:
        if (lint, line) not in hits:
            problems.append(f"ban coverage: {path} ({reason.split(':')[0] or 'no reason'}) didn't fire {lint} on line {line}")
    return problems


# --- the clean run -------------------------------------------------------------------


def clean_run(log):
    """The engine as it is passes every tool, and nothing is silently off."""
    problems = []
    for tool in ("clippy", "clippy-tests", "rustdoc"):
        diags, code, out = run_tool(tool, log)
        bad = [d for d in diags if d.level in ("error", "warning")]
        if code != 0 or bad:
            problems.append(f"clean {tool}: {bad[0].text.splitlines()[0] if bad else f'exit {code}'}")
        for pattern in check.SILENT:
            if pattern.search(out):
                problems.append(f"clean {tool}: the output says \"{pattern.pattern}\": a rule silently does nothing")
    for name, argv in (("fmt", ["cargo", "fmt", "--all", "--check"]), ("ast-grep", ["ast-grep", "scan"]),
                       ("check-rules", ["tools/check-rules"]), ("check-frozen", ["tools/check-frozen"]),
                       ("exceptions", ["tools/exceptions", "--check"]),
                       ("fault-injection-probe", ["tools/check", "step", "fault-injection-probe"]),
                       ("tree-probe", ["tools/check", "step", "tree-probe"])):
        r = run(argv, log)
        if r.returncode != 0:
            problems.append(f"clean {name}: {check.first_error(r.stdout + r.stderr)}")
    for tool in ("deny", "deny-tally"):
        diags, code, _ = run_tool(tool, log)
        bad = [d for d in diags if d.level == "error"]
        if code != 0 or bad:
            problems.append(f"clean {tool}: {bad[0].code + ' ' + bad[0].text[:100] if bad else f'exit {code}'}")
    r = run(["ast-grep", "scan", "--inspect", "entity"], log)
    severity = dict(re.findall(r"entity\|rule\|([\w-]+): finalSeverity=(\w+)", r.stdout + r.stderr))
    rules_dir = os.path.join(WS, "tools", "ast-grep", "rules")
    for name in sorted(os.listdir(rules_dir)):
        rule_id = name.removesuffix(".yml")
        if severity.get(rule_id) in (None, "Off"):
            problems.append(f"clean ast-grep: the rule {rule_id} isn't on (--inspect entity says {severity.get(rule_id)})")
    problems += ci_problems()
    return problems


def ci_problems():
    """R-77: every pull request runs the full check, whose steps are R-77's list."""
    problems = []
    names = [s[0] for s in check.steps("full")]
    for step in PR_STEPS:
        if step not in names:
            problems.append(f"R-77: `check full` has no {step} step")
    workflow = os.path.join(ROOT, ".github", "workflows", "engine.yml")
    if not os.path.isfile(workflow):
        problems.append("R-77: .github/workflows/engine.yml is missing")
    else:
        text = open(workflow, encoding="utf-8").read()
        if "engine/tools/check full" not in text:
            problems.append("R-77: .github/workflows/engine.yml doesn't run `engine/tools/check full`")
        if not re.search(r"\n  pull_request:\n    branches: \[master, engine\]\n", text):
            problems.append(
                "R-77: .github/workflows/engine.yml doesn't run on pull requests into master and engine"
            )
    return problems


# --- the whole run -------------------------------------------------------------------


def batch_key(case):
    if case.solo or case.tool not in ("clippy", "clippy-tests", "ast-grep"):
        return None
    pkgs = set()
    for kind, args, _, _ in case.blocks:
        if kind == "module":
            pkgs.add(args[0] if args else "tally")
        elif kind == "write" and args[0].endswith(".rs") and not os.path.exists(os.path.join(ENGINE, args[0])):
            parts = args[0].split("/")
            pkgs.add("tally-runner" if parts[0] == "tests" else parts[1])
        else:
            return None
    if case.tool != "clippy":
        return (case.tool, "")
    return (case.tool, pkgs.pop()) if len(pkgs) == 1 else None


def script_of(case):
    return "\n".join(body(b[2]) for b in case.blocks if b[0] == "run")


def run_solo(case):
    with open(os.path.join(LOGS, re.sub(r"[^\w.-]+", "-", case.id) + ".log"), "w", encoding="utf-8") as log:
        reset()
        try:
            touched = plant(case)
            diags, code, out = run_tool(case.tool, log, [t for t in touched if t.endswith(".rs")], script_of(case), case.fetch)
            problems = judge(case, diags, code, out, touched)
        except (CaseError, OSError, RuntimeError) as e:
            problems = [str(e)]
        log.write(f"verdict: {problems or 'fired'}\n")
        reset()
    return problems


def run_batch(key, group, with_coverage):
    name = "-".join(x for x in key if x)
    with open(os.path.join(LOGS, f"batch-{name}.log"), "w", encoding="utf-8") as log:
        reset()
        touched, results, cov_problems, fakes = {}, {}, [], []
        for c in group:
            try:
                touched[c.id] = plant(c)
            except (CaseError, OSError) as e:
                results[c.id] = [str(e)]
        planted = []
        if with_coverage:
            planted, cov_problems, fakes = plant_coverage()
        files = sorted({t for ts in touched.values() for t in ts if t.endswith(".rs")})
        diags, code, out = run_tool(key[0], log, files)
        for c in group:
            if c.id not in results:
                results[c.id] = judge(c, diags, code, out, touched[c.id])
        if with_coverage:
            cov_problems += judge_coverage(planted, diags)
            for pattern in check.SILENT:
                if pattern.search(out):
                    cov_problems.append(f"ban coverage: the output says \"{pattern.pattern}\"")
        log.write(f"verdicts: {results}\ncoverage: {cov_problems}\n")
        reset()
    return results, (len(planted), cov_problems, fakes)


def completeness(cases):
    have = {c.rule for c in cases}
    missing = [f"R-{n}" for n in range(1, 114)
               if f"R-{n}" not in have | set(ELSEWHERE) | REVIEW | DROPPED | BY_COVERAGE]
    return [f"{r} has no case in tools/rules-selftest/cases/ and isn't mapped elsewhere" for r in missing]


def list_rules(cases):
    by_rule = {}
    for c in cases:
        by_rule.setdefault(c.rule, []).append(f"{c.name} ({c.tool})")
    for n in range(1, 114):
        r = f"R-{n}"
        if r in by_rule:
            print(f"{r}: {', '.join(by_rule[r])}")
        elif r in BY_COVERAGE:
            print(f"{r}: the ban coverage")
        elif r in ELSEWHERE:
            print(f"{r}: {ELSEWHERE[r]}")
        elif r in REVIEW:
            print(f"{r}: review only")
        elif r in DROPPED:
            print(f"{r}: dropped at gate 1")
        else:
            print(f"{r}: NOTHING")


def main():
    args = sys.argv[1:]
    try:
        cases = load_cases()
    except (CaseError, OSError) as e:
        print(f"FAIL {e}")
        return 1
    if args == ["--list"]:
        list_rules(cases)
        return 0
    if any(not re.fullmatch(r"R-\d+", a) for a in args):
        print("usage: engine/tools/rules-selftest/run [--list | R-<n> ...]", file=sys.stderr)
        return 64
    began = time.monotonic()
    problems = completeness(cases)
    if args:
        cases = [c for c in cases if c.rule in set(args)]
    for tool in ("cargo", "ast-grep", "cargo-deny", "git"):
        if shutil.which(tool) is None:
            print(f"FAIL setup: {tool} is not on PATH")
            return 1
    os.makedirs(LOGS, exist_ok=True)
    try:
        build_scratch()
    except (OSError, RuntimeError) as e:
        print(f"FAIL setup: the scratch repository: {e}")
        return 1
    with open(os.path.join(LOGS, "clean.log"), "w", encoding="utf-8") as log:
        fetched = run(["cargo", "fetch", "--locked"], log, env(fetching=False) | {"CARGO_NET_OFFLINE": "false"})
        if fetched.returncode != 0:
            print("FAIL setup: cargo fetch --locked failed (see target/rules-selftest/logs/clean.log)")
            return 1
        clean = clean_run(log)
    if clean:
        for p in clean:
            print(f"FAIL {p}")
        print("rules-selftest: red: the clean run fails, so no case can be judged (target/rules-selftest/logs/clean.log)")
        return 1
    print("rules-selftest: the clean run passes every tool, nothing is silently off")

    groups, solo = {("clippy", "tally"): []}, []
    for c in cases:
        key = batch_key(c)
        if key:
            groups.setdefault(key, []).append(c)
        else:
            solo.append(c)
    verdicts, coverage = {}, None
    for key, group in groups.items():
        results, cov = run_batch(key, group, key == ("clippy", "tally"))
        if key == ("clippy", "tally"):
            coverage = cov
        for c in group:
            # a miss in a batch is checked alone before it counts
            verdicts[c.id] = run_solo(c) if results[c.id] else []
    for c in solo:
        verdicts[c.id] = run_solo(c)

    count, cov_problems, fakes = coverage
    failed = 0
    for p in problems + cov_problems:
        print(f"FAIL {p}")
    for c in cases:
        for p in verdicts[c.id]:
            print(f"FAIL {c.id}: {p}  (target/rules-selftest/logs/)")
        failed += bool(verdicts[c.id])
    fake_note = f"; fakes for {', '.join(fakes)}" if fakes else ""
    print(f"rules-selftest: ban coverage {count - len(cov_problems)}/{count} entries fire{fake_note}")
    took = time.monotonic() - began
    if problems or cov_problems or failed:
        print(f"rules-selftest: red, {failed} of {len(cases)} cases failed ({took:.0f}s)")
        return 1
    print(f"rules-selftest: green, {len(cases)} cases over {len({c.rule for c in cases})} rules ({took:.0f}s)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
