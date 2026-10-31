"""The engine's guard: a Claude Code PreToolUse hook, and its ConfigChange hook.

    engine/tools/guard                  PreToolUse: reads the hook's JSON on stdin
    engine/tools/guard --config-change  ConfigChange: blocks settings changes in `build`

It refuses an edit by exiting 2 with the reason on stderr, and it fails closed: the
shim beside it turns any other exit into 2, so a crash refuses the call (R-79).

The role comes from TALLY_ROLE (R-79):
  build  the build session: writes only its allow-list (the engine's code, the Cargo
         files, its log, the survivors file, the exception lock)
  rules  the clean-side lead: everything but the spec, the oracle and engine/tests/
  tests  the test owner (T): engine/tests/ only
  spec   the clean-side lead patching the spec: engine/spec/ and the oracle only
  unset  app work: anything but the spec, the oracle, the tests and the rule files

The oracle is the spec's reference interpreter: engine/tools/refcore/, refcore_fast/
and engine/tools/SHA256SUMS (refcore's checksums). It is frozen with the spec.

The Bash check is a tripwire, not a wall: it refuses a command that names a protected
path together with a form that writes. The checksums (check-frozen) and a clean CI
runner are the wall.
"""

import datetime
import fnmatch
import json
import os
import re
import shlex
import sys

TOOLS = os.path.dirname(os.path.realpath(__file__))
ENGINE = os.path.dirname(TOOLS)
ROOT = os.path.dirname(ENGINE)
HOME = os.path.realpath(os.path.expanduser("~"))
ROLES = ("build", "rules", "tests", "spec", "unset")

REPORT = (
    "Report it instead: a STOP-REPORT in engine/BUILD-LOG.md (R-83); a test gap is T's, "
    "in engine/tests/GAPS.md; a rule change goes through R-112's procedure."
)

# Rule files: refused to every role but `rules` (C §5, R-80, R-90, R-91)
RULE_GLOBS = [
    "engine/clippy.toml",
    "engine/deny*.toml",
    "engine/sgconfig.yml",
    "engine/rust-toolchain.toml",
    "engine/rust-toolchain",
    "engine/.cargo/*",
    "engine/.config/*",
    "engine/tools/*",
    "engine/fuzz/*",
    "engine/crates/*/benches/*",
    ".github/workflows/engine.yml",
    ".claude/settings.json",
]
CONFIG_NAMES = {"clippy.toml", ".clippy.toml", "rustfmt.toml", ".rustfmt.toml"}
# The oracle: the spec's reference interpreter, frozen with it and written only in the
# `spec` role (R-80); inside engine/tools/ but not a rule file
ORACLE_GLOBS = [
    "engine/tools/refcore",
    "engine/tools/refcore/*",
    "engine/tools/refcore_fast",
    "engine/tools/refcore_fast/*",
    "engine/tools/sha256sums",
]
# The build role's allow-list (C §5)
BUILD_FILES = {
    "engine/cargo.toml",
    "engine/cargo.lock",
    "engine/build-log.md",
    "engine/mutants-survivors.md",
    "engine/exceptions.lock",
}

WRITERS = {
    "cp", "mv", "rm", "rmdir", "touch", "ln", "install", "rsync", "dd", "truncate",
    "chmod", "chown", "chgrp", "unlink", "mkdir", "tee", "shred", "patch", "cargo-fmt",
}
INTERPRETERS = {
    "python", "python3", "perl", "ruby", "node", "bun", "deno", "awk", "gawk", "php",
    "lua", "osascript", "tclsh", "pwsh",
}
SHELLS = {"bash", "sh", "zsh", "dash", "ksh", "fish"}
PREFIXES = {"sudo", "env", "command", "exec", "nohup", "time", "nice", "xargs", "builtin", "doas"}
GIT_WRITES = {
    "checkout", "restore", "reset", "rm", "mv", "apply", "stash", "clean", "switch", "am",
    "cherry-pick", "revert", "merge", "rebase", "pull", "update-index", "read-tree",
    "checkout-index", "filter-branch", "filter-repo", "worktree", "commit",
}
FIND_WRITES = {"-delete", "-exec", "-execdir", "-ok", "-okdir", "-fprint", "-fprintf", "-fls"}
ATTRIBUTION = ("co-authored-by", "noreply@anthropic.com", "generated with [claude", "generated with claude")


class Refused(Exception):
    """A refusal, with the message the session sees."""


def rel(path):
    """The path relative to the repo root, casefolded ('' for the root), or None if outside."""
    p = os.path.realpath(path)
    folded, root = p.casefold(), ROOT.casefold()
    # casefolded: macOS's file system ignores case, so engine/SPEC is engine/spec
    if folded == root:
        return ""
    if folded.startswith(root + os.sep):
        return folded[len(root) + 1 :].replace(os.sep, "/")
    return None


def matches(r, globs):
    return any(fnmatch.fnmatchcase(r, g.casefold()) for g in globs)


def is_oracle(r):
    return matches(r, ORACLE_GLOBS)


def is_rule(r):
    if is_oracle(r):
        return False
    if matches(r, RULE_GLOBS):
        return True
    if not r.startswith("engine/"):
        return False
    name = os.path.basename(r)
    # a config file anywhere in the engine changes how it is built or linted (R-80)
    return name in CONFIG_NAMES or name.startswith("rust-toolchain") or "/.cargo/" in "/" + r


def build_allowed(r):
    if r in BUILD_FILES or r.startswith("engine/target/"):
        return True
    if not r.startswith("engine/crates/"):
        return False
    parts = r.split("/")
    name = parts[-1]
    if name == "build.rs" or name in CONFIG_NAMES or name.startswith("rust-toolchain"):
        return False
    if ".cargo" in parts:
        return False
    return not (len(parts) > 3 and parts[3] == "benches")


def verdict(path, role):
    """None if `role` may write `path`, else the reason it may not."""
    r = rel(path)
    spec = r is not None and (r == "engine/spec" or r.startswith("engine/spec/"))
    tests = r is not None and (r == "engine/tests" or r.startswith("engine/tests/"))
    oracle = r is not None and is_oracle(r)
    rule = r is not None and is_rule(r)
    if role == "build":
        if r is not None and build_allowed(r):
            return None
        if spec:
            return "the spec is frozen and checksummed (R-80)"
        if tests:
            return "the tests are T's and checksummed (R-80, R-84)"
        if oracle:
            return "the reference interpreter is frozen with the spec and checksummed (R-80)"
        if rule:
            return "a rule file, protected and checksummed (R-80, R-81, R-91)"
        if r is None:
            return "outside the repo, which the build role never writes (R-79)"
        return "not on the build role's allow-list (R-79)"
    if role == "rules":
        if spec:
            return "the spec changes only in the spec role (R-80)"
        if tests:
            return "the tests change only in the tests role (R-80)"
        if oracle:
            return "the reference interpreter changes only in the spec role (R-80)"
        return None
    if role == "tests":
        return None if tests else "the tests role writes engine/tests/ only (R-79)"
    if role == "spec":
        return None if spec or oracle else "the spec role writes engine/spec/ and the reference interpreter only (R-79)"
    # unset: app work
    if spec:
        return "the spec is frozen and checksummed (R-80)"
    if tests:
        return "the tests are T's and checksummed (R-80)"
    if oracle:
        return "the reference interpreter is frozen with the spec and checksummed (R-80)"
    if rule:
        return "a rule file, protected and checksummed (R-80, R-81)"
    return None


def watched(path, role):
    """Whether a Bash command naming `path` is checked at all: the repo, and in the
    build role also ~/.claude and ~/.cargo (R-81). Scratch space elsewhere is free."""
    r = rel(path)
    if r == "":
        return True
    if r is None:
        if role != "build":
            return False
        p = os.path.realpath(path)
        return p.startswith(os.path.join(HOME, ".claude")) or p.startswith(os.path.join(HOME, ".cargo"))
    if role != "build":
        return True
    return r.startswith(("engine", ".claude", ".github")) or os.path.basename(r) in CONFIG_NAMES or "/.cargo/" in "/" + r + "/"


# One path inside each protected area, to catch a command on a folder that holds one
ANCHORS = [
    "engine/spec/DONE.md",
    "engine/tests/x",
    "engine/tools/refcore/x",
    "engine/tools/refcore_fast/x",
    "engine/tools/SHA256SUMS",
    "engine/tools/guard",
    "engine/clippy.toml",
    "engine/fuzz/Cargo.toml",
    "engine/.cargo/mutants.toml",
    "engine/.config/nextest.toml",
    ".github/workflows/engine.yml",
    ".claude/settings.json",
    ".claude/settings.local.json",
]


def bash_protected(path, role):
    """The reason a Bash command may not write `path` (or a folder holding a protected
    file), or None."""
    if not watched(path, role):
        return None
    reason = verdict(path, role)
    if reason:
        return reason
    r = rel(path)
    if r is None:
        return None
    anchors = list(ANCHORS)
    crates = os.path.join(ENGINE, "crates")
    if os.path.isdir(crates):
        anchors += [f"engine/crates/{c.casefold()}/benches/x" for c in sorted(os.listdir(crates))]
    for anchor in anchors:
        if r == "" or anchor.startswith(r.rstrip("/") + "/"):
            inner = verdict(os.path.join(ROOT, anchor), role)
            if inner:
                return f"it holds {anchor}, and {inner}"
    return None


def say(role, what, target, reason):
    shown = rel(target)
    shown = target if shown is None else (shown or ".")
    return f"engine guard (role {role}): refused {what} of {shown}: {reason}. {REPORT}"


# --- Bash --------------------------------------------------------------------------


def split_lines(command):
    """Replace newlines outside quotes with ';' so each line is its own command."""
    out, quote, escape = [], None, False
    for ch in command:
        if escape:
            out.append(ch)
            escape = False
            continue
        if ch == "\\" and quote != "'":
            out.append(ch)
            escape = True
            continue
        if quote:
            if ch == quote:
                quote = None
            out.append(ch)
            continue
        if ch in ("'", '"'):
            quote = ch
            out.append(ch)
            continue
        out.append(";" if ch == "\n" else ch)
    return "".join(out)


def strip_heredocs(command):
    """Split off here-document bodies: they are data to the command line, and an
    apostrophe in one would unbalance the quotes. The redirect that writes them stays.
    Returns the command without them, and the bodies (checked when a shell or an
    interpreter reads them)."""
    out, ends, bodies = [], [], []
    for line in command.split("\n"):
        if ends:
            if line.strip() == ends[0] or (ends[0].startswith("-") and line.strip() == ends[0][1:]):
                ends.pop(0)
            else:
                bodies.append(line)
            continue
        out.append(line)
        for dash, word in re.findall(r"(?<!<)<<(?!<)(-?)\s*['\"]?([A-Za-z_][A-Za-z0-9_]*)['\"]?", line):
            ends.append(("-" if dash else "") + word)
    return "\n".join(out), "\n".join(bodies)


def tokens_of(command, role):
    lexer = shlex.shlex(split_lines(command), posix=True, punctuation_chars=";&|()<>")
    lexer.whitespace_split = True
    lexer.commenters = ""
    try:
        return list(lexer)
    except ValueError:
        if role == "build":
            raise Refused(f"engine guard (role build): refused a Bash command it can't parse (unbalanced quotes). {REPORT}")
        return command.split()


SEPARATORS = {";", "&&", "||", "|", "&", "(", ")", "|&", ";;"}
REDIRECTS = {">", ">>", ">|", "&>", "&>>", ">&", "<>"}


def resolve(token, cwd):
    t = os.path.expanduser(os.path.expandvars(token))
    return t if os.path.isabs(t) else os.path.join(cwd, t)


def dynamic(token):
    return "$" in token or "`" in token


def check_target(token, cwd, role, what):
    if not token:
        return
    if dynamic(os.path.expandvars(token)):
        # a computed path: the build role can't be trusted with one; for the others it
        # names nothing the guard can check
        if role == "build":
            raise Refused(f"engine guard (role build): refused {what} to a computed path ({token}). {REPORT}")
        return
    path = resolve(token, cwd)
    reason = bash_protected(path, role)
    if reason:
        raise Refused(say(role, what, path, reason))


PROTECTED_NAMES = {"spec", "tests", "tools", "fuzz", "benches", ".cargo", ".config", ".claude", ".github", "engine"}
QUOTED = re.compile(r"'([^'\n]*)'|\"([^\"\n]*)\"")


def path_ish(s):
    return bool(s) and not s.startswith("-") and (
        "/" in s or s.casefold() in PROTECTED_NAMES or re.fullmatch(r"[\w.+-]*\w\.\w+", s) is not None
    )


def path_like(code):
    """The strings in an interpreter's code that look like paths: quoted ones, and bare
    words that start at a protected folder or a path root."""
    found = [q.rstrip(".") for pair in QUOTED.findall(code) for q in pair if path_ish(q.rstrip("."))]
    for w in re.findall(r"[\w.~/$+-]+", QUOTED.sub(" ", code)):
        w = w.rstrip(".")
        head = w.split("/", 1)[0]
        if "/" in w and (head in ("", "~", ".", "..") or head.casefold() in PROTECTED_NAMES):
            found.append(w)
    return found


def code_like(arg):
    return re.search(r"[\s()'\";{}=,]", arg) is not None


def interpreter_paths(args):
    """The paths an interpreter's arguments name: inside inline code, the strings that
    look like paths; as a plain argument, one with a slash or an extension. The first
    plain operand is the script it runs, which it only reads (a tripwire can't see what
    a script then writes; check-frozen does)."""
    found, script_seen = [], False
    for a in args:
        if a.startswith("-"):
            continue
        if code_like(a):
            found += path_like(a)
        elif not script_seen:
            script_seen = True
        elif path_ish(a):
            found.append(a)
    return found


# options whose value is the next word, so the value isn't taken for a path
VALUED = {
    "truncate": {"-s", "-r", "--size", "--reference"},
    "touch": {"-t", "-d", "-r", "-A"},
    "mkdir": {"-m", "--mode"},
    "install": {"-m", "-o", "-g", "-S", "--mode", "--owner", "--group", "--suffix"},
    "cp": {"-S", "--suffix"},
    "mv": {"-S", "--suffix"},
    "ln": {"-S", "--suffix"},
    "rsync": {"-e", "--rsh", "--exclude", "--include", "--filter", "-f", "--files-from", "-T", "--temp-dir"},
    "patch": {"-p", "-d", "-D", "-F", "-B", "-z", "-V", "-Y", "-i", "-o", "-r"},
    "shred": {"-n", "-s", "--iterations", "--size"},
}
# options whose value is the place written
DEST = {"-t", "--target-directory"}
TARGET_OPTIONS = {"cp": DEST, "mv": DEST, "ln": DEST, "install": DEST, "patch": {"-o", "--output"}}
# git commands that change the working tree, refused to the build role unless they
# name only paths it may write (R-79): a ref or a patch can carry any file
GIT_WHOLE_TREE = {
    "apply", "am", "merge", "rebase", "pull", "cherry-pick", "revert", "stash", "clean",
    "switch", "worktree", "filter-branch", "filter-repo", "read-tree", "update-index", "checkout-index",
}


def operands(name, args):
    """The words of `args` that are operands, and the values of target options."""
    valued, dest = VALUED.get(name, set()), TARGET_OPTIONS.get(name, set())
    found, targets, skip = [], [], None
    for a in args:
        if skip is not None:
            if skip in dest:
                targets.append(a)
            skip = None
            continue
        if a in valued or a in dest:
            skip = a
            continue
        if "=" in a and a.split("=", 1)[0] in dest:
            targets.append(a.split("=", 1)[1])
            continue
        if a.startswith("-"):
            continue
        found.append(a)
    if name in ("chown", "chgrp") or (name == "chmod" and not any(re.fullmatch(r"-[rwxXst]+", a) for a in args)):
        found = found[1:]  # the owner or the mode
    return found, targets


def check_git(args, cwd, role):
    sub, rest = None, list(args)
    while rest:
        a = rest.pop(0)
        if a in ("-C", "-c", "--git-dir", "--work-tree"):
            if rest:
                v = rest.pop(0)
                if a == "-C":
                    cwd = os.path.realpath(resolve(v, cwd))
            continue
        if a.startswith("-"):
            continue
        sub = a
        break
    if sub not in GIT_WRITES or sub == "commit":
        return
    if "--" in rest:
        paths = [a for a in rest[rest.index("--") + 1 :] if a]
    else:
        # without `--`, a word is a path if it exists or points into the engine or the
        # settings; otherwise it's a ref or a message
        paths = []
        for a in rest:
            if a.startswith("-"):
                continue
            r = rel(resolve(a, cwd))
            if os.path.lexists(resolve(a, cwd)) or (r is not None and r.startswith(("engine", ".claude", ".github"))):
                paths.append(a)
    if role == "build":
        hard = sub == "reset" and any(a in ("--hard", "--merge", "--keep") for a in rest)
        if sub in GIT_WHOLE_TREE or hard or (sub in ("checkout", "restore") and not paths):
            raise Refused(f"engine guard (role build): refused `git {sub}`, which can change any file in the tree (R-79). {REPORT}")
    for a in paths:
        check_target(a, cwd, role, f"`git {sub}`")


def check_segment(argv, cwd, role, depth, state):
    """Checks one simple command; returns the cwd after it (cd)."""
    # redirections first, wherever they sit
    words = []
    i = 0
    while i < len(argv):
        tok = argv[i]
        if tok in REDIRECTS or re.fullmatch(r"\d+(>|>>|>&|>\|)", tok):
            target = argv[i + 1] if i + 1 < len(argv) else ""
            i += 2
            if tok.endswith(">&") or tok == ">&":
                if re.fullmatch(r"\d+|-", target):
                    continue
            if target in ("/dev/null", "/dev/stdout", "/dev/stderr") or re.fullmatch(r"&\d+|&-", target):
                continue
            check_target(target, cwd, role, "a redirect")
            continue
        if tok in ("<", "<<", "<<<"):
            i += 2
            continue
        if re.fullmatch(r"\d+", tok) and i + 1 < len(argv) and argv[i + 1] in REDIRECTS:
            i += 1
            continue
        words.append(tok)
        i += 1
    # leading assignments and prefixes
    while words and re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*=.*", words[0]):
        words = words[1:]
    while words and os.path.basename(words[0]) in PREFIXES:
        name = os.path.basename(words[0])
        if name == "xargs":
            state["xargs"] = True
        words = words[1:]
        while words and (words[0].startswith("-") or re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*=.*", words[0])):
            if name == "command" and words[0] == "-v":
                return cwd
            words = words[1:]
    if not words:
        return cwd
    name = os.path.basename(words[0])
    args = words[1:]
    if name in ("cd", "pushd"):
        target = args[-1] if args else HOME
        if target != "-":
            cwd = os.path.realpath(resolve(target, cwd))
        return cwd
    if role == "build" and (name == "claude" or "claude-code" in " ".join(words)):
        raise Refused(f"engine guard (role build): refused a nested claude run (R-81). {REPORT}")
    if name in SHELLS:
        if "-c" in args:
            k = args.index("-c")
            if k + 1 < len(args):
                check_command(args[k + 1], cwd, role, depth + 1)
        else:
            state["shell"] = cwd
        return cwd
    if name == "eval":
        check_command(" ".join(args), cwd, role, depth + 1)
        return cwd
    if name == "timeout":
        rest = [a for a in args if not a.startswith("-")]
        return check_segment(rest[1:], cwd, role, depth, state) if len(rest) > 1 else cwd
    writes = name in WRITERS or name in INTERPRETERS or re.fullmatch(r"python3?(\.\d+)?|perl\d*|ruby\d*|node\d*", name)
    if state.get("xargs") and writes:
        state["blind"] = name
    if name == "dd":
        for a in args:
            if a.startswith("of="):
                check_target(a[3:], cwd, role, "`dd`")
        return cwd
    if name in WRITERS:
        found, targets = operands(name, args)
        if name in ("cp", "install", "ln", "rsync"):
            # these write only their destination
            found = targets or found[-1:]
        else:
            found += targets
        for a in found:
            check_target(a, cwd, role, f"`{name}`")
        return cwd
    if name in ("sed", "gsed") and any(a.startswith("--in-place") or re.fullmatch(r"-[a-zA-Z]*i.*", a) for a in args):
        files = [a for a in args if a and not a.startswith("-")]
        if not any(a in ("-e", "-f", "--expression", "--file") for a in args):
            files = files[1:]  # the first operand is the script
        for a in files:
            check_target(a, cwd, role, "`sed -i`")
        return cwd
    if writes:
        state["interpreter"] = (name, cwd)
        for s in interpreter_paths(args):
            check_target(s, cwd, role, f"an interpreter (`{name}`) naming it")
        return cwd
    if name == "git":
        check_git(args, cwd, role)
        return cwd
    if name == "find" and any(a in FIND_WRITES for a in args):
        roots = []
        for a in args:
            if a.startswith(("-", "(", "!", ")")):
                break
            roots.append(a)
        for a in roots or ["."]:
            check_target(a, cwd, role, "`find` with a writing action")
        return cwd
    return cwd


def check_command(command, cwd, role, depth=0):
    if depth > 8:
        raise Refused(f"engine guard (role {role}): refused a command nested too deeply to check. {REPORT}")
    if role == "build":
        folded = command.casefold()
        if "disableallhooks" in folded:
            raise Refused(f"engine guard (role build): refused a command naming disableAllHooks (R-81). {REPORT}")
        if "tally_role" in folded:
            raise Refused(f"engine guard (role build): refused a command naming TALLY_ROLE (R-79). {REPORT}")
    if role != "unset" and re.search(r"\bgit\b", command) and re.search(r"\bcommit\b", command):
        if any(m in command.casefold() for m in ATTRIBUTION):
            raise Refused(f"engine guard (role {role}): refused a commit with an AI co-author or attribution line (R-100).")
    state = {}
    start = cwd
    command, bodies = strip_heredocs(command)
    tokens = tokens_of(command, role)
    segment = []
    for tok in tokens + [";"]:
        if tok in SEPARATORS:
            if segment:
                cwd = check_segment(segment, cwd, role, depth, state)
            segment = []
        else:
            segment.append(tok)
    if state.get("blind"):
        # a writer fed by xargs: its targets are unknown, so any protected path refuses
        for tok in tokens:
            if "/" in tok or tok.casefold() in PROTECTED_NAMES:
                check_target(tok, start, role, f"`xargs {state['blind']}`")
    if bodies and "shell" in state:
        # a here-document fed to a shell is a script
        check_command(bodies, state["shell"], role, depth + 1)
    if bodies and "interpreter" in state:
        name, where = state["interpreter"]
        for s in path_like(bodies):
            check_target(s, where, role, f"a here-document fed to `{name}` naming it")


# --- entry points ------------------------------------------------------------------


def role_of():
    role = os.environ.get("TALLY_ROLE", "") or "unset"
    if role not in ROLES:
        raise Refused(f"engine guard: TALLY_ROLE={role!r} is not a role ({', '.join(ROLES[:-1])}, or unset); refusing everything (R-79).")
    return role


def pre_tool_use(event, role):
    tool = event.get("tool_name")
    data = event.get("tool_input")
    if not isinstance(data, dict):
        raise Refused("engine guard: the hook input has no tool_input; refusing (R-79).")
    cwd = event.get("cwd") or os.getcwd()
    if tool in ("Edit", "Write", "MultiEdit", "NotebookEdit"):
        path = data.get("notebook_path") if tool == "NotebookEdit" else data.get("file_path")
        if not isinstance(path, str) or not path:
            raise Refused(f"engine guard: {tool} without a path; refusing (R-79).")
        path = resolve(path, cwd)
        reason = verdict(path, role)
        if reason:
            raise Refused(say(role, tool, path, reason))
        return
    if tool == "Bash":
        command = data.get("command")
        if not isinstance(command, str):
            raise Refused("engine guard: Bash without a command; refusing (R-79).")
        check_command(command, os.path.realpath(cwd), role)
        return
    raise Refused(f"engine guard: unknown tool {tool!r}; refusing (R-79).")


def config_change(event, role):
    if role != "build":
        return
    source = event.get("source") or event.get("matcher") or "unknown"
    path = event.get("file_path") or ""
    stamp = datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    line = f"\n- {stamp} CONFIG-CHANGE BLOCKED: source {source} {path} (R-81)\n"
    with open(os.path.join(ENGINE, "BUILD-LOG.md"), "a", encoding="utf-8") as log:
        log.write(line)
    raise Refused(f"engine guard (role build): refused a settings change ({source}) (R-81).")


def main():
    if os.environ.get("TALLY_GUARD_SELFTEST_CRASH") == "1":
        raise RuntimeError("forced internal failure (guard-test)")
    try:
        role = role_of()
        raw = sys.stdin.read()
        event = json.loads(raw) if raw.strip() else {}
        if not isinstance(event, dict):
            raise Refused("engine guard: the hook input isn't a JSON object; refusing (R-79).")
        if sys.argv[1:] == ["--config-change"]:
            config_change(event, role)
        elif sys.argv[1:] == []:
            pre_tool_use(event, role)
        else:
            raise Refused("usage: guard [--config-change]")
    except Refused as refused:
        print(str(refused), file=sys.stderr)
        sys.exit(2)
    except json.JSONDecodeError:
        print("engine guard: the hook input isn't JSON; refusing (R-79).", file=sys.stderr)
        sys.exit(2)


if __name__ == "__main__":
    main()
