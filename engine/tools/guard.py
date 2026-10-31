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
  lead   the coordinator (issue #18): writes like app work, and alone runs
         engine/tools/merge-pr and engine/tools/worktree-done

Every role also writes tmp/, the local scratch nothing tracks (issue #11); build, rules,
tests and spec don't write the sealed tmp/holdout/ and tmp/cases-holdout/, or tmp/active/.

The oracle is the spec's reference interpreter: engine/tools/refcore/, refcore_fast/
and engine/tools/SHA256SUMS (refcore's checksums). It is frozen with the spec.

Every checkout of the repo is judged alike: the main checkout and each linked worktree,
found from the git files, so a session in one can't write another's spec (F-1).

The Bash check is a tripwire, not a wall: it refuses a command that names a protected
path together with a form that writes. The checksums (check-frozen) and a clean CI
runner are the wall.
"""

import datetime
import fnmatch
import glob
import json
import os
import re
import shlex
import sys
import tomllib
import urllib.parse

TOOLS = os.path.dirname(os.path.realpath(__file__))
ENGINE = os.path.dirname(TOOLS)
ROOT = os.path.dirname(ENGINE)
HOME = os.path.realpath(os.path.expanduser("~"))
ROLES = ("build", "rules", "tests", "spec", "lead", "unset")
# the roles that write like app work: anything but the spec, the oracle, the tests and
# the rule files, and all of tmp/ (the lead, issue #18)
APP_WRITERS = ("unset", "lead")

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
    # the builder's instructions (D6-3, F-23); the rest of .claude/ is the toolkit's
    "engine/CLAUDE.md",
    ".claude/skills/engine-rust/*",
    ".claude/agents/engine-reviewer.md",
    # the rules' full texts, which the skill and the reviewer cite (issue #3)
    "engine/RULEBOOK.md",
    "engine/BUILDER.md",
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
# tmp/ is local scratch, never tracked, which every role writes (issue #11), but these
# and tmp/active/ (the master's briefs); app work keeps all of tmp/
SEALED_SCRATCH = ("tmp/holdout", "tmp/cases-holdout")

WRITERS = {
    "cp", "mv", "rm", "rmdir", "touch", "ln", "install", "rsync", "dd", "truncate",
    "chmod", "chown", "chgrp", "unlink", "mkdir", "tee", "shred", "patch", "cargo-fmt",
    "ditto", "rustfmt", "vim", "vi", "nvim", "ex", "ed", "uniq",
}
# writers whose only target is their destination (the last operand, or -t)
COPIERS = {"cp", "install", "ln", "rsync", "ditto"}
EDITORS = {"vim", "vi", "nvim", "ex", "ed"}
INTERPRETERS = {
    "python", "python3", "perl", "ruby", "node", "bun", "deno", "awk", "gawk", "php",
    "lua", "osascript", "tclsh", "pwsh",
}
SHELLS = {"bash", "sh", "zsh", "dash", "ksh", "fish"}
# commands that run the rest of their words as a command: (their options that take a
# value, the operands they take before the command) (F-2)
PREFIXES = {
    "sudo": ({"-u", "-g", "-U", "-C", "-D", "-h", "-p", "-r", "-t", "-T", "--user", "--group",
              "--other-user", "--chdir", "--prompt", "--role", "--type", "--host", "--close-from"}, 0),
    "doas": ({"-u", "-C"}, 0),
    "env": ({"-u", "-C", "-S", "-P", "--unset", "--chdir", "--split-string"}, 0),
    "command": (set(), 0),
    "exec": ({"-a"}, 0),
    "nohup": (set(), 0),
    "time": ({"-f", "-o", "--format", "--output"}, 0),
    "nice": ({"-n", "--adjustment"}, 0),
    "ionice": ({"-c", "-n", "-p", "-P", "-u", "--class", "--classdata", "--pid", "--pgid", "--uid"}, 0),
    "xargs": ({"-n", "-I", "-L", "-P", "-s", "-E", "-d", "-a", "-J", "-R", "-S", "--max-args",
               "--replace", "--max-lines", "--max-procs", "--max-chars", "--eof", "--delimiter",
               "--arg-file", "--process-slot-var"}, 0),
    "builtin": (set(), 0),
    "timeout": ({"-s", "-k", "--signal", "--kill-after"}, 1),  # the duration
    "stdbuf": ({"-i", "-o", "-e", "--input", "--output", "--error"}, 0),
    "flock": ({"-w", "-E", "--timeout", "--conflict-exit-code"}, 1),  # the lock file
    "caffeinate": ({"-t", "-w"}, 0),
    "unbuffer": (set(), 0),
}
# shell words that open or close a compound command; the command follows them (F-2)
RESERVED = {"if", "then", "else", "elif", "fi", "do", "done", "while", "until", "{", "}", "!", "esac"}
ASSIGNMENT = re.compile(r"[A-Za-z_][A-Za-z0-9_]*=.*")
GIT_WRITES = {
    "checkout", "restore", "reset", "rm", "mv", "apply", "stash", "clean", "switch", "am",
    "cherry-pick", "revert", "merge", "rebase", "pull", "update-index", "read-tree",
    "checkout-index", "filter-branch", "filter-repo", "worktree", "commit",
}
# git commands that take a message (R-100)
GIT_MESSAGES = {"commit", "merge", "tag", "notes", "commit-tree"}
FIND_WRITES = {"-delete", "-exec", "-execdir", "-ok", "-okdir", "-fprint", "-fprintf", "-fls"}
ATTRIBUTION = ("co-authored-by", "noreply@anthropic.com", "generated with [claude", "generated with claude")
# cargo commands that write sources or manifests: judged by the manifests they act on
CARGO_WRITES = {"fmt", "fix", "add", "remove", "rm"}


class Refused(Exception):
    """A refusal, with the message the session sees."""


def read_line(path):
    with open(path, encoding="utf-8") as f:
        return f.read().strip()


def find_roots():
    """Every checkout of the repo, realpaths, longest first: this one, the main checkout
    and each linked worktree. Read from the git files, without running git (F-1):
    ROOT/.git is a folder (the main checkout) or a `gitdir:` file (a linked worktree),
    whose `commondir` names the common folder; its worktrees/*/gitdir name the others."""
    dotgit = os.path.join(ROOT, ".git")
    try:
        if os.path.isdir(dotgit):
            gitdir = dotgit
        else:
            line = read_line(dotgit)
            if not line.startswith("gitdir:"):
                raise ValueError(f"{dotgit} has no gitdir line")
            gitdir = os.path.join(ROOT, line[len("gitdir:") :].strip())
        if not os.path.isdir(gitdir):
            raise ValueError(f"{gitdir} isn't a folder")
        common = gitdir
        if os.path.lexists(os.path.join(gitdir, "commondir")):
            common = os.path.join(gitdir, read_line(os.path.join(gitdir, "commondir")))
        common = os.path.realpath(common)
        roots = {ROOT}
        if os.path.basename(common) == ".git":
            roots.add(os.path.dirname(common))
        trees = os.path.join(common, "worktrees")
        if os.path.isdir(trees):
            for name in sorted(os.listdir(trees)):
                entry = os.path.join(trees, name)
                if os.path.isdir(entry):
                    roots.add(os.path.dirname(os.path.join(entry, read_line(os.path.join(entry, "gitdir")))))
    except (OSError, ValueError) as e:
        raise Refused(f"engine guard: can't read the repo's git files to find its checkouts ({e}); refusing (fails closed, R-79).") from e
    return sorted({os.path.realpath(r) for r in roots}, key=len, reverse=True)


_ROOTS = []


def roots():
    if not _ROOTS:
        _ROOTS.extend(find_roots())
    return _ROOTS


def locate(path):
    """(the checkout holding `path`, the path relative to it casefolded, '' for its root),
    or (None, None) outside every checkout. The longest root holding it wins, so a
    worktree inside another checkout is judged as itself."""
    p = os.path.realpath(path).casefold()
    # casefolded: macOS's file system ignores case, so engine/SPEC is engine/spec
    for root in roots():
        folded = root.casefold()
        if p == folded:
            return root, ""
        if p.startswith(folded + os.sep):
            return root, p[len(folded) + 1 :].replace(os.sep, "/")
    return None, None


def rel(path):
    """The path relative to the checkout holding it, casefolded ('' for its root), or None
    if outside every checkout."""
    return locate(path)[1]


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


def under(r, folder):
    """Whether the checkout-relative path `r` is `folder` or inside it."""
    return r is not None and (r == folder or r.startswith(folder + "/"))


def scratch_verdict(r):
    """In every role but app work: None for a path under tmp/, which is scratch (issue
    #11), or the reason for one of its sealed folders. `r` is the real path, casefolded,
    so neither a symlink nor a case form reaches them."""
    if any(under(r, f) for f in SEALED_SCRATCH):
        return "the sealed exam and its record, which no session writes (R-79)"
    if under(r, "tmp/active"):
        return "the master's briefs, which a session doesn't rewrite (R-79)"
    return None


def verdict(path, role):
    """None if `role` may write `path`, else the reason it may not."""
    root, r = locate(path)
    if role not in APP_WRITERS and under(r, "tmp"):
        return scratch_verdict(r)
    spec = r is not None and (r == "engine/spec" or r.startswith("engine/spec/"))
    tests = r is not None and (r == "engine/tests" or r.startswith("engine/tests/"))
    oracle = r is not None and is_oracle(r)
    rule = r is not None and is_rule(r)
    if role == "build":
        if r is not None and root == ROOT and build_allowed(r):
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
        if root != ROOT:
            return "in another checkout of the repo, which the build role never writes (R-79)"
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
    # unset (app work) and lead
    if spec:
        return "the spec is frozen and checksummed (R-80)"
    if tests:
        return "the tests are T's and checksummed (R-80)"
    if oracle:
        return "the reference interpreter is frozen with the spec and checksummed (R-80)"
    if rule:
        return "a rule file, protected and checksummed (R-80, R-81)"
    return None


def outside_watched():
    """The places outside the checkouts the build role never writes, realpaths casefolded:
    Claude Code's settings by name (~/.claude, ~/.claude.json, $CLAUDE_CONFIG_DIR; R-81,
    F-22), and every cargo config folder cargo reads: each `.cargo/` from a checkout up
    to `/`, ~/.cargo and $CARGO_HOME (F-9)."""
    found = [os.path.join(HOME, ".claude"), os.path.join(HOME, ".claude.json"), os.path.join(HOME, ".cargo")]
    found += [os.environ[k] for k in ("CLAUDE_CONFIG_DIR", "CARGO_HOME") if os.environ.get(k)]
    for root in roots():
        d = os.path.dirname(root)
        while True:
            found.append(os.path.join(d, ".cargo"))
            if os.path.dirname(d) == d:
                break
            d = os.path.dirname(d)
    return {os.path.realpath(os.path.expanduser(p)).casefold() for p in found}


def watched(path, role):
    """Whether a Bash command naming `path` is checked at all: every checkout of the
    repo, and in the build role also the places outside_watched() names. Scratch space
    elsewhere is free."""
    r = rel(path)
    if r == "":
        return True
    if r is None:
        if role != "build":
            return False
        p = os.path.realpath(path).casefold()
        return any(p == d or p.startswith(d + os.sep) for d in outside_watched())
    if role != "build":
        return True
    return r.startswith(("engine", ".claude", ".github")) or under(r, "tmp") or os.path.basename(r) in CONFIG_NAMES or "/.cargo/" in "/" + r + "/"


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
    ".claude/skills/engine-rust/SKILL.md",
    ".claude/agents/engine-reviewer.md",
    "tmp/holdout/x",
    "tmp/cases-holdout/x",
    "tmp/active/x",
]


def bash_protected(path, role):
    """The reason a Bash command may not write `path` (or a folder holding a protected
    file), or None."""
    if not watched(path, role):
        return None
    reason = verdict(path, role)
    if reason:
        return reason
    root, r = locate(path)
    if r is None:
        return None
    anchors = list(ANCHORS)
    crates = os.path.join(root, "engine", "crates")
    if os.path.isdir(crates):
        anchors += [f"engine/crates/{c.casefold()}/benches/x" for c in sorted(os.listdir(crates))]
    for anchor in anchors:
        if r == "" or anchor.startswith(r.rstrip("/") + "/"):
            inner = verdict(os.path.join(root, anchor), role)
            if inner:
                return f"it holds {anchor}, and {inner}"
    return None


def say(role, what, target, reason):
    root, r = locate(target)
    # another checkout's path is shown whole
    shown = target if r is None else os.path.realpath(target) if root != ROOT else (r or ".")
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
        found = list(lexer)
    except ValueError:
        if role == "build":
            raise Refused(f"engine guard (role build): refused a Bash command it can't parse (unbalanced quotes). {REPORT}") from None
        return command.split()
    return [part for tok in found for part in split_operators(tok)]


SEPARATORS = {";", "&&", "||", "|", "&", "(", ")", "|&", ";;", ">(", "<("}
REDIRECTS = {">", ">>", ">|", "&>", "&>>", ">&", "<>"}
OPERATORS = sorted(SEPARATORS | REDIRECTS | {"<", "<<", "<<<"}, key=len, reverse=True)


def split_operators(tok):
    """A run of operator characters the lexer kept as one token (`);`, `>(`), split into
    its operators, so a process substitution or a closing paren still separates (F-2)."""
    if not tok or any(c not in ";&|()<>" for c in tok):
        return [tok]
    out, i = [], 0
    while i < len(tok):
        op = next((o for o in OPERATORS if tok.startswith(o, i)), tok[i])
        out.append(op)
        i += len(op)
    return out


def substitutions(command):
    """The commands inside `$(…)` and backticks, outside single quotes and inside double
    ones: they run as commands of their own (F-2)."""
    found, i, n, quote = [], 0, len(command), None
    while i < n:
        c = command[i]
        if c == "\\" and quote != "'":
            i += 2
        elif quote == "'":
            quote = None if c == "'" else quote
            i += 1
        elif c == "'" and quote is None:
            quote = "'"
            i += 1
        elif c == '"':
            quote = None if quote == '"' else '"'
            i += 1
        elif c == "`":
            j = command.find("`", i + 1)
            j = n if j < 0 else j
            found.append(command[i + 1 : j])
            i = j + 1
        elif command.startswith("$(", i) and not command.startswith("$((", i):
            j, depth, inner = i + 2, 1, None
            while j < n and depth:
                d = command[j]
                if d == "\\" and inner != "'":
                    j += 1
                elif inner:
                    inner = None if d == inner else inner
                elif d in ("'", '"'):
                    inner = d
                elif d == "(":
                    depth += 1
                elif d == ")":
                    depth -= 1
                j += 1
            found.append(command[i + 2 : j - 1 if not depth else j])
            i = j
        else:
            i += 1
    return found


def expands(command):
    """Whether the shell expands anything in `command`: a `$` or a backtick outside
    single quotes."""
    quote, escape = None, False
    for c in command:
        if escape:
            escape = False
        elif c == "\\" and quote != "'":
            escape = True
        elif quote == "'":
            quote = None if c == "'" else quote
        elif c == "'" and quote is None:
            quote = "'"
        elif c == '"':
            quote = None if quote == '"' else '"'
        elif c in "$`":
            return True
    return False


def resolve(token, cwd):
    t = os.path.expanduser(os.path.expandvars(token))
    return t if os.path.isabs(t) else os.path.join(cwd, t)


def dynamic(token):
    return "$" in token or "`" in token


# the words of each `for NAME in WORDS` in the command being checked, by NAME (F-2)
LOOPS = {}


def check_target(token, cwd, role, what, loops=True):
    if not token:
        return
    for name, words in LOOPS.items() if loops else ():
        for form in (f"${name}", f"${{{name}}}"):
            if form in token:
                # a loop variable stands for each of its words (one level deep)
                for w in words:
                    check_target(token.replace(form, w), cwd, role, what, loops=False)
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


def code_letters(name):
    """The short options after which an interpreter takes inline code, and the long ones."""
    if name.startswith("python"):
        return "c", {"-c"}
    if name.startswith("perl"):
        return "eE", set()
    if name.startswith(("ruby", "lua", "osascript")):
        return "e", set()
    if name.startswith(("node", "bun", "deno")):
        return "ep", {"--eval", "--print"}
    if name == "php":
        return "r", set()
    if name == "pwsh":
        return "c", {"-command", "--command"}
    return "", set()


# python's options whose value is neither code nor the script
PYTHON_VALUED = {"-W", "-X"}


def interpreter_paths(name, args):
    """The paths an interpreter's arguments name: inside inline code, the strings that
    look like paths; as a plain argument, one with a slash or an extension. The script
    it runs, it only reads: inline code (`-e '…'`, `-c "…"`, `-es/a/b/`, or a word that
    looks like code), or else the first plain operand (a tripwire can't see what a
    script then writes; check-frozen does). Every operand after it is checked (F-3)."""
    letters, long_code = code_letters(name)
    found, script_seen, code_next, skip = [], False, False, False
    for a in args:
        if skip:
            skip = False
            continue
        if code_next:
            found += path_like(a)
            script_seen, code_next = True, False
            continue
        if a.startswith("-") and len(a) > 1:
            if a.casefold() in long_code:
                code_next = True
            elif a.startswith("--"):
                if "=" in a and a.split("=", 1)[0] in long_code:
                    found += path_like(a.split("=", 1)[1])
                    script_seen = True
            elif a in PYTHON_VALUED and name.startswith("python"):
                skip = True
            elif a[-1] in letters:
                code_next = True  # -e, -ne, -lne, -Bc: the code is the next word
            elif a[1] in letters:
                found += path_like(a[2:])  # -es/a/b/: the code is attached
                script_seen = True
            continue
        if code_like(a):
            found += path_like(a)
            script_seen = True
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
    "rustfmt": {"--edition", "--config", "--config-path", "--emit", "--color", "--style-edition", "--print-config"},
    "uniq": {"-f", "-s", "-w"},
    **{e: {"-c", "--cmd", "-S", "-u", "-U", "-i", "-T", "-w", "-W", "-t", "-q"} for e in EDITORS},
}
# options whose value is the place written
DEST = {"-t", "--target-directory"}
TARGET_OPTIONS = {"cp": DEST, "mv": DEST, "ln": DEST, "install": DEST, "patch": {"-o", "--output"}}


def short_value(opt, letters):
    """For a short-option word like `-sSo` or `-ofile`: whether one of `letters` (options
    that take a value) is in it, and the value attached after it ('' when it's the next
    word). None when it holds none of them."""
    if opt.startswith("--") or len(opt) < 2:
        return None
    for k, ch in enumerate(opt[1:], 1):
        if ch in letters:
            return ch, opt[k + 1 :]
    return None


def option_targets(args, short, long):
    """The values of the options that name the place written: short letters (`-o f`,
    `-of`, `-sSo f`) and long names (`--output f`, `--output=f`)."""
    found, it = [], iter(args)
    for a in it:
        if a.startswith("--"):
            key, eq, value = a.partition("=")
            if key in long:
                found.append(value if eq else next(it, ""))
            continue
        if a.startswith("-"):
            hit = short_value(a, short)
            if hit:
                found.append(hit[1] or next(it, ""))
    return found


def tar_targets(args, cwd):
    """Where tar writes: with -x, the -C folder (or the cwd); with -c, -r or -u, the
    archive (-f) (F-4)."""
    mode, first = "", args[0] if args else ""
    if first and not first.startswith("-") and re.fullmatch(r"[A-Za-z]+", first):
        mode += first  # the old bundled form: tar xzf a.tgz
        args = ["-" + first, *args[1:]]
    for a in args:
        if a in ("--extract", "--get"):
            mode += "x"
        elif a in ("--create", "--append", "--update"):
            mode += "c"
        elif re.fullmatch(r"-[A-Za-z]+", a):
            mode += a[1:]
    found = []
    if "x" in mode:
        found += option_targets(args, "C", {"--directory"}) or ["."]
    if any(m in mode for m in "cru"):
        found += [f for f in option_targets(args, "f", {"--file"}) if f != "-"]
    return found


def download_targets(name, args):
    """Where curl or wget writes: its output options, and with curl -O (or wget by
    default) the URL's last name, in the output folder or the cwd (F-4)."""
    urls = [a for a in args if "://" in a]
    if name == "curl":
        found = option_targets(args, "oDc", {"--output", "--dump-header", "--cookie-jar", "--trace",
                                             "--trace-ascii", "--stderr", "--libcurl", "--etag-save"})
        folder = (option_targets(args, "", {"--output-dir"}) or ["."])[-1]
        remote = any(a in ("-O", "--remote-name", "--remote-name-all", "-J", "--remote-header-name")
                     or (re.fullmatch(r"-[A-Za-z]+", a) and "O" in a) for a in args)
    else:
        found = [f for f in option_targets(args, "Ooa", {"--output-document", "--output-file", "--append-output"}) if f != "-"]
        folder = (option_targets(args, "P", {"--directory-prefix"}) or ["."])[-1]
        remote = not option_targets(args, "O", {"--output-document"})
    if remote:
        for u in urls:
            last = os.path.basename(urllib.parse.urlsplit(u).path) or "index.html"
            found.append(os.path.join(folder, last))
    return [f for f in found if f not in ("-", "/dev/null")]


def cargo_targets(args, cwd):
    """The manifests a cargo command that writes sources or manifests acts on: the
    -p packages', or --manifest-path, or the cwd's package; every member's for --all,
    --workspace or a workspace with no package of its own (F-4). Judging the manifest
    stands for the package's files: its benches go with it (the checksums cover them)."""
    manifest = (option_targets(args, "", {"--manifest-path"}) or [None])[-1]
    packages = option_targets(args, "p", {"--package"})
    if manifest is None:
        d = cwd
        while not os.path.isfile(os.path.join(d, "Cargo.toml")):
            if os.path.dirname(d) == d or rel(d) == "":
                return [os.path.join(cwd, "Cargo.toml")]
            d = os.path.dirname(d)
        manifest = os.path.join(d, "Cargo.toml")
    start = manifest if os.path.isabs(manifest) else os.path.join(cwd, manifest)

    def load(path):
        try:
            with open(path, "rb") as f:
                return tomllib.load(f)
        except (OSError, tomllib.TOMLDecodeError):
            return {}

    top, data = start, load(start)
    if not data:
        return [start]
    if "workspace" not in data:
        d = os.path.dirname(os.path.dirname(start))
        while rel(d) is not None:
            if "workspace" in load(os.path.join(d, "Cargo.toml")):
                top = os.path.join(d, "Cargo.toml")
                break
            if rel(d) == "":
                break
            d = os.path.dirname(d)
    ws = load(top).get("workspace", {})
    members = {}
    for pattern in ws.get("members", []) if isinstance(ws, dict) else []:
        for m in sorted(glob.glob(os.path.join(os.path.dirname(top), str(pattern)))):
            name = load(os.path.join(m, "Cargo.toml")).get("package", {}).get("name")
            members[name or m] = os.path.join(m, "Cargo.toml")
    if packages:
        return [members.get(p, start) for p in packages]
    if "--all" in args or "--workspace" in args or ("package" not in data and members):
        return sorted(set(members.values()))
    return [start]
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


def check_message(sub, rest, cwd, role, state):
    """R-100 beyond the command's text: the file `-F` names is read and scanned; a
    message computed by the shell (`-m "$(…)"`, `-m "$M"`) or piped in (`-F -` without a
    here-document) can't be checked, so it is refused (F-8)."""
    it = iter(rest)
    for a in it:
        kind = value = None
        if a in ("-m", "--message", "-F", "--file"):
            kind, value = a.lstrip("-")[0].casefold(), next(it, None)
        elif a.startswith(("--message=", "--file=")):
            kind, value = a[2], a.split("=", 1)[1]
        elif not a.startswith("--") and a.startswith("-"):
            hit = short_value(a, "mF")
            if hit:
                kind, value = hit[0].casefold(), hit[1] or next(it, None)
        if kind is None:
            continue
        what = f"engine guard (role {role}): refused `git {sub}`"
        if kind == "m":
            if value is not None and state.get("stdin_text") and re.fullmatch(r"\$\(\s*cat\s+<<-?\s*['\"]?\w+['\"]?\s*\)", value):
                continue  # -m "$(cat <<'EOF' … EOF)": the here-document is in the text, scanned already
            # a `$` or backtick the shell expands (not one inside single quotes)
            if value is not None and dynamic(value) and state.get("expands"):
                raise Refused(f"{what} with a computed message, which can't be checked for an AI co-author or attribution line (R-100); write the message to a file and use -F.")
            continue
        if value == "-":
            if not state.get("stdin_text"):
                raise Refused(f"{what} -F - without a here-document: a piped message can't be checked for an AI co-author or attribution line (R-100).")
            continue  # the here-document is in the command's text, scanned already
        if value is None or dynamic(value):
            raise Refused(f"{what} with a message file it can't read (R-100).")
        try:
            with open(resolve(value, cwd), encoding="utf-8", errors="replace") as f:
                text = f.read().casefold()
        except OSError:
            raise Refused(f"{what}: can't read the message file {value} to check it for an AI co-author or attribution line (R-100).") from None
        if any(m in text for m in ATTRIBUTION):
            raise Refused(f"{what}: the message file {value} has an AI co-author or attribution line (R-100).")


# --- R-132: changes land on master only by a pull request the user merges, or the --
# --- lead merges through engine/tools/merge-pr (issue #18) --------------------------


MASTER = "master"
LANDING = (
    "changes land on master only by a pull request the user merges, or the lead merges "
    "through engine/tools/merge-pr (R-132). Push your own branch and open a PR with "
    "`gh pr create`"
)
# the lead's tools, which only the lead runs (issue #18): by name, with or without .py
LEAD_TOOLS = ("merge-pr", "merge_pr", "worktree-done", "worktree_done")
# git push's options whose value is the next word
PUSH_VALUED = {"-o", "--push-option", "--repo", "--receive-pack", "--exec"}
# git push's options that push every branch, master included
PUSH_EVERY = {"--all", "--mirror", "--branches"}
# gh api endpoints and GraphQL mutations that merge, move or unprotect master
GH_API_MERGES = re.compile(
    r"pulls/[^/\s]+/merge(?![\w-])|/merges(?![\w-])|branches/master/protection|git/refs/heads/master(?![\w-])"
    r"|mergePullRequest|enablePullRequestAutoMerge|mergeBranch|updateRef|deleteRef",
    re.IGNORECASE,
)


def current_branch(cwd):
    """The branch checked out where `cwd` is, from the git files without running git:
    the nearest `.git` above it (a folder, or a linked worktree's `gitdir:` file) and
    its HEAD. None when no repo holds `cwd` or HEAD is detached."""
    d = cwd
    while not os.path.lexists(os.path.join(d, ".git")):
        if os.path.dirname(d) == d:
            return None
        d = os.path.dirname(d)
    dotgit = os.path.join(d, ".git")
    try:
        gitdir = dotgit
        if not os.path.isdir(dotgit):
            line = read_line(dotgit)
            if not line.startswith("gitdir:"):
                raise ValueError(f"{dotgit} has no gitdir line")
            gitdir = os.path.join(d, line[len("gitdir:") :].strip())
        head = read_line(os.path.join(gitdir, "HEAD"))
    except (OSError, ValueError) as e:
        raise Refused(f"engine guard: can't read which branch {d} has checked out ({e}), so a `git push` can't be checked against master; refusing (R-132).") from e
    return head[len("ref: refs/heads/") :] if head.startswith("ref: refs/heads/") else None


def is_master(ref):
    """Whether a ref names master: `master`, `heads/master`, `refs/heads/master`, in any case."""
    return ref is not None and ref.removeprefix("refs/").removeprefix("heads/").casefold() == MASTER


def check_push(rest, cwd, role, state):
    """R-132: no `git push` whose destination is master, in any form: a refspec naming
    it (`master`, `HEAD:master`, `+master`, `:master`, `--delete master`, forced or
    not), `--all`, `--mirror`, or no refspec at all while master is checked out."""
    what = f"engine guard (role {role}): refused `git push`"
    words, repo_given, tags, it = [], False, False, iter(rest)
    for a in it:
        if a == "--":
            words += list(it)
            break
        key = a.split("=", 1)[0]
        if key in PUSH_EVERY:
            raise Refused(f"{what} {key}, which pushes master too: {LANDING}.")
        if key == "--repo":
            repo_given = True
        if a in PUSH_VALUED:
            next(it, None)
        elif a == "--tags":
            tags = True
        elif not a.startswith("--") and a.startswith("-"):
            hit = short_value(a, "o")
            if hit and not hit[1]:
                next(it, None)  # -o VALUE, -fo VALUE
        if not a.startswith("-"):
            words.append(a)
    refspecs = words if repo_given else words[1:]
    if not refspecs:
        if tags:
            return  # only the tags
        if is_master(current_branch(cwd)):
            raise Refused(f"{what} on branch master, which pushes master: {LANDING}.")
        return
    for spec in refspecs:
        if dynamic(spec) and state.get("expands"):
            raise Refused(f"{what} to a computed ref ({spec}), which can't be checked against master (R-132); name the branch.")
        src, colon, dest = spec.lstrip("+").partition(":")
        dest = dest if colon and dest else src
        if dest in ("HEAD", "@"):
            dest = current_branch(cwd)
        if is_master(dest):
            raise Refused(f"{what} to master ({spec}): {LANDING}.")


def check_gh(args, role):
    """R-132: no `gh pr merge`, and no `gh api` call that merges a pull request or a
    branch, moves or deletes master's ref, or changes master's protection, in every
    role, the lead included: the lead merges only through engine/tools/merge-pr."""
    words, it = [], iter(args)
    for a in it:
        if a in ("-R", "--repo"):
            next(it, None)
        elif not a.startswith("-"):
            words.append(a)
    what = f"engine guard (role {role}): refused"
    if words[:2] == ["pr", "merge"]:
        raise Refused(f"{what} `gh pr merge`: {LANDING}.")
    if words[:1] == ["api"] and any(GH_API_MERGES.search(a) for a in args):
        raise Refused(f"{what} a `gh api` call that merges, moves master or changes its protection: {LANDING}.")


def lead_tool(name, args):
    """The lead's tool a command runs, or None (issue #18): the command word itself
    (`merge-pr`, `merge_pr.py`, any case), a script given to a shell, `source` or an
    interpreter, or an interpreter's inline code naming one (`python3 -c "…merge_pr…"`).
    A shell's `-c` string is checked as a command of its own."""
    def stem(word):
        return os.path.basename(word).casefold().removesuffix(".py")

    if stem(name) in LEAD_TOOLS:
        return stem(name)
    interpreter = name in INTERPRETERS or re.fullmatch(r"python3?(\.\d+)?|perl\d*|ruby\d*|node\d*", name)
    if not (interpreter or name in SHELLS or name in ("source", ".")):
        return None
    for a in args:
        if stem(a) in LEAD_TOOLS:
            return stem(a)
        if interpreter:
            hit = next((t for t in LEAD_TOOLS if t in a.casefold()), None)
            if hit:
                return hit
    return None


def check_git(args, cwd, role, state):
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
    if role != "unset" and sub in GIT_MESSAGES:
        check_message(sub, rest, cwd, role, state)
    if sub == "push":
        check_push(rest, cwd, role, state)
        return
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


def prefix_option(opt, valued):
    """(the option, its attached value or None) when `opt` is one of `valued`, including a
    short one in a bundle (`-n5`, `-iu`) or a long one with `=`; (None, None) otherwise."""
    if opt in valued:
        return opt, None
    if opt.startswith("--"):
        key, eq, value = opt.partition("=")
        return (key, value) if eq and key in valued else (None, None)
    hit = short_value(opt, "".join(v[1] for v in valued if len(v) == 2))
    return ("-" + hit[0], hit[1] or None) if hit else (None, None)


def strip_prefixes(words, cwd, role, depth, state):
    """Drops the words before the command: reserved words, assignments and the prefixes
    with their options and operands. Returns (words, cwd), or (None, cwd) when the
    command is done with (a `flock -c` checked as its own command, `command -v`)."""
    while words:
        w = words[0]
        if w in RESERVED or ASSIGNMENT.fullmatch(w):
            words = words[1:]
            continue
        name = os.path.basename(w)
        if name not in PREFIXES:
            break
        if name == "xargs":
            state["xargs"] = True
        valued, operands_before = PREFIXES[name]
        words = words[1:]
        while words and words[0].startswith("-") and words[0] != "--":
            opt = words.pop(0)
            if name == "command" and opt in ("-v", "-V"):
                return None, cwd
            key, value = prefix_option(opt, valued)
            if key is None:
                continue
            if value is None:
                value = words.pop(0) if words else ""
            if name == "env" and key in ("-S", "--split-string"):
                words = tokens_of(value, role) + words  # the string is the command
            elif name == "env" and key in ("-C", "--chdir"):
                cwd = os.path.realpath(resolve(value, cwd))
        if words and words[0] == "--":
            words = words[1:]
        words = words[operands_before:]
        if name == "flock" and words and words[0] in ("-c", "--command"):
            if len(words) > 1:
                check_command(words[1], cwd, role, depth + 1)
            return None, cwd
    return words, cwd


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
    if len(words) >= 3 and words[0] == "for" and words[2] == "in":
        LOOPS[words[1]] = words[3:]  # `for f in a b`: $f stands for a and b
        return cwd
    # reserved words, leading assignments and prefixes
    words, cwd = strip_prefixes(words, cwd, role, depth, state)
    if not words:
        return cwd
    name = os.path.basename(words[0])
    args = words[1:]
    if name in ("cd", "pushd"):
        target = args[-1] if args else HOME
        if target != "-":
            cwd = os.path.realpath(resolve(target, cwd))
        return cwd
    if role != "lead":
        tool = lead_tool(name, args)
        if tool:
            raise Refused(f"engine guard (role {role}): refused `{tool}`: only the lead role (TALLY_ROLE=lead) runs it (R-132, issue #18).")
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
        if name == "rustfmt" and ("--check" in args or "stdout" in args or "--emit=stdout" in args):
            return cwd
        if name in EDITORS:
            # the files it opens, and the paths its commands (-c, --cmd, +cmd) name
            code = [args[k + 1] for k, a in enumerate(args[:-1]) if a in ("-c", "--cmd")]
            code += [a[1:] for a in args if a.startswith("+")]
            found = [a for a in found if not a.startswith("+")] + [p for c in code for p in path_like(c)]
        elif name == "uniq":
            found = found[1:2]  # uniq IN OUT writes OUT
        elif name in COPIERS:
            linking = (name == "ln" and not any(a == "--symbolic" or re.fullmatch(r"-[A-Za-z]*s[A-Za-z]*", a) for a in args)) or (
                name == "cp" and any(a == "--link" or re.fullmatch(r"-[A-Za-z]*l[A-Za-z]*", a) for a in args)
            )
            # a hard link is a second name for its source, so the source counts too (F-5)
            found = found + targets if linking else (targets or found[-1:])
        else:
            found += targets
        if name == "patch" and role == "build" and not found:
            raise Refused(f"engine guard (role build): refused `patch` without a file, which can change any file in the tree (R-79). {REPORT}")
        for a in found:
            check_target(a, cwd, role, f"`{name}`")
        return cwd
    if name in ("sed", "gsed") and any(a.startswith("--in-place") or re.fullmatch(r"-[a-zA-Z]*i.*", a) for a in args):
        files = [a for a in args if a and not a.startswith("-")]
        # the script is given by an option (-e, -f, -es/a/b/, --expression=…), or it is
        # the first operand (F-3)
        if not any(a in ("-e", "-f", "--expression", "--file") or a.startswith(("--expression=", "--file="))
                   or re.fullmatch(r"-[a-zA-Z]*[ef].*", a) for a in args):
            files = files[1:]
        for a in files:
            check_target(a, cwd, role, "`sed -i`")
        return cwd
    if writes:
        state["interpreter"] = (name, cwd)
        for s in interpreter_paths(name, args):
            check_target(s, cwd, role, f"an interpreter (`{name}`) naming it")
        return cwd
    if name in ("tar", "bsdtar", "gtar"):
        for a in tar_targets(args, cwd):
            check_target(a, cwd, role, "`tar`")
        return cwd
    if name == "unzip":
        if not any(re.fullmatch(r"-[A-Za-z]*[lptvzZc][A-Za-z]*", a) for a in args):
            for a in option_targets(args, "d", set()) or ["."]:
                check_target(a, cwd, role, "`unzip`")
        return cwd
    if name in ("curl", "wget"):
        for a in download_targets(name, args):
            check_target(a, cwd, role, f"`{name}`")
        return cwd
    if name in ("sort", "gsort"):
        for a in option_targets(args, "o", {"--output"}):
            check_target(a, cwd, role, "`sort -o`")
        return cwd
    if name == "cargo":
        k = next((k for k, a in enumerate(args) if not a.startswith(("-", "+"))), len(args))
        sub, rest = (args[k], args[k + 1 :]) if k < len(args) else (None, [])
        own = rest[: rest.index("--")] if "--" in rest else rest
        if sub == "clippy" and "--fix" in own or sub in CARGO_WRITES and "--check" not in rest:
            for m in cargo_targets(own, cwd):
                check_target(m, cwd, role, f"`cargo {sub}` (it acts on each package's files: name yours with -p)")
        return cwd
    if name == "gh":
        check_gh(args, role)
        return cwd
    if name == "git":
        check_git(args, cwd, role, state)
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
    if role != "unset" and re.search(r"\bgit\b", command) and re.search(r"\b(commit|merge|tag|notes|commit-tree)\b", command):
        if any(m in command.casefold() for m in ATTRIBUTION):
            raise Refused(f"engine guard (role {role}): refused a commit with an AI co-author or attribution line (R-100).")
    command, bodies = strip_heredocs(command)
    state = {"stdin_text": bool(bodies) or "<<<" in command, "expands": expands(command)}
    start = cwd
    # `$(…)` and backticks run as commands of their own, quoted or not (F-2)
    for inner in substitutions(command):
        check_command(inner, cwd, role, depth + 1)
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
        LOOPS.clear()
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
