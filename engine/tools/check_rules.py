"""check-rules: the files the builder edits keep the rules (R-80, R-85, R-86, R-93).

    engine/tools/check-rules                  check
    engine/tools/check-rules --write-frozen   rewrite engine/tools/frozen-lints.toml from
                                              the current files (the `rules` role, R-112)

The builder edits the Cargo manifests, the lock and the engine's code, so the guard
can't protect them. This checks what in them is a rule: the lint and profile tables,
the pins, the root package.json's engine:* scripts, the members and their lints, no
patches, build scripts or stray config files, and the text the code may not hold.
"""

import fnmatch
import json
import os
import re
import subprocess
import sys
import tomllib

TOOLS = os.path.dirname(os.path.realpath(__file__))
ENGINE = os.path.dirname(TOOLS)
ROOT = os.path.dirname(ENGINE)
FROZEN = os.path.join(TOOLS, "frozen-lints.toml")
KERNEL = os.path.join(TOOLS, "kernel.toml")
SKIP_DIRS = {"target", "__pycache__", ".git", "corpus", "artifacts", "coverage"}
PINNED = ("num-bigint", "num-integer", "num-traits", "sha2")
DEP_TABLES = ("dependencies", "dev-dependencies", "build-dependencies")
# the assert family and the arguments each needs before its message (R-71)
ASSERT = re.compile(r"\b(debug_assert_eq|debug_assert_ne|debug_assert|assert_eq|assert_ne|assert)\s*!\s*([(\[{])")
ASSERT_ARGS = {"assert": 1, "debug_assert": 1}
# R-51: the features that exist only in builds with debug assertions, each with its
# compile_error! guard in tally's lib.rs: gate 5's fault injection, the counters behind
# --stats (BUILDER §13.2) and gate 6's reversed completion order (D3-6)
TEST_ONLY = ("fault-injection", "counters", "reverse-order")


def guard_text(feature):
    return (f'#[cfg(all(feature = "{feature}", not(debug_assertions)))]\n'
            f'compile_error!("the {feature} feature exists only in builds with debug assertions (R-51)");')


def declared_test_only():
    """The test-only features tally's manifest declares; fault-injection always counts."""
    try:
        features = load(os.path.join(ENGINE, "crates", "tally", "Cargo.toml")).get("features", {})
    except (OSError, ValueError):
        features = {}
    return [f for f in TEST_ONLY if f in features or f == "fault-injection"]
# the only config files that may change how the engine is built or linted
EXPECTED_CONFIG = {"engine/clippy.toml", "engine/.cargo/mutants.toml", "engine/rust-toolchain.toml"}
CONFIG_NAMES = {"clippy.toml", ".clippy.toml", "rustfmt.toml", ".rustfmt.toml", "rust-toolchain", "rust-toolchain.toml"}
PRIMITIVES = {
    "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize",
    "f32", "f64", "bool", "char", "str", "slice", "array", "pointer", "tuple", "unit",
}
# text the library may not hold (R-93, R-39, R-86); `cfg(debug_assertions)` is allowed
# once, in the fault-injection guard
FORBIDDEN_TEXT = [
    (re.compile(r"\bcfg!\s*\(\s*test\s*\)"), "cfg!(test): the engine can't tell it is tested (R-93)"),
    (re.compile(r"\bcfg!\s*\(\s*debug_assertions\s*\)"), "cfg!(debug_assertions): no checks on debug assertions (R-93)"),
    (re.compile(r"\bdebug_assertions\b"), "debug_assertions: no checks on debug assertions (R-93)"),
    (re.compile(r"\binclude_str!"), "include_str!: the library reads no files (R-39, R-93)"),
    (re.compile(r"\binclude_bytes!"), "include_bytes!: the library reads no files (R-39, R-93)"),
    (re.compile(r"\bstd\s*::\s*fs\b"), "std::fs: the library reads no files (R-39)"),
]
PATH_STRING = re.compile(r"(^|[^\w])(tests/|expected/|spec/|\.\./)")


def load(path):
    with open(path, "rb") as f:
        return tomllib.load(f)


def walk(top):
    for d, dirs, names in os.walk(top):
        dirs[:] = sorted(x for x in dirs if x not in SKIP_DIRS)
        for n in sorted(names):
            yield os.path.join(d, n)


def rel(path):
    return os.path.relpath(path, ROOT).replace(os.sep, "/")


# --- the frozen copy ---------------------------------------------------------------


def current_frozen():
    ws = load(os.path.join(ENGINE, "Cargo.toml"))
    pkg = _package_json()
    deps = ws.get("workspace", {}).get("dependencies", {})
    return {
        "workspace-lints": ws.get("workspace", {}).get("lints", {}),
        "profile": ws.get("profile", {}),
        "pins": {name: deps.get(name) for name in PINNED},
        "root-scripts": {k: v for k, v in sorted(pkg.get("scripts", {}).items()) if k.startswith("engine:")},
    }


def _package_json():
    with open(os.path.join(ROOT, "package.json"), encoding="utf-8") as f:
        return json.load(f)


def toml_value(v):
    if isinstance(v, bool):
        return "true" if v else "false"
    if isinstance(v, (int, float)):
        return repr(v)
    if isinstance(v, str):
        return '"' + v.replace("\\", "\\\\").replace('"', '\\"') + '"'
    if isinstance(v, list):
        return "[" + ", ".join(toml_value(x) for x in v) + "]"
    if isinstance(v, dict):
        return "{ " + ", ".join(f"{toml_key(k)} = {toml_value(x)}" for k, x in v.items()) + " }"
    raise TypeError(f"can't write {type(v).__name__} as TOML")


def toml_key(k):
    return k if re.fullmatch(r"[A-Za-z0-9_-]+", k) else toml_value(k)


def toml_tables(prefix, table, out, depth=1):
    """Two levels of [tables]; anything deeper is written inline."""
    inline = {k: v for k, v in table.items() if not isinstance(v, dict) or depth >= 2}
    if inline or not table:
        out.append(f"[{prefix}]")
        out += [f"{toml_key(k)} = {toml_value(v)}" for k, v in inline.items()]
        out.append("")
    for k, v in table.items():
        if k not in inline:
            toml_tables(f"{prefix}.{toml_key(k)}", v, out, depth + 1)


def write_frozen():
    data = current_frozen()
    out = [
        "# The frozen copy check-rules compares the builder-edited files with (R-80). A rule",
        "# file: the `rules` role rewrites it with `engine/tools/check-rules --write-frozen`, in",
        "# a rule change's commit (R-112). Generated; don't edit by hand.",
        "",
    ]
    for top in ("workspace-lints", "profile", "pins", "root-scripts"):
        toml_tables(top, data[top], out)
    with open(FROZEN, "w", encoding="utf-8") as f:
        f.write("\n".join(out).rstrip("\n") + "\n")


# --- the checks --------------------------------------------------------------------


def check_frozen_tables(problems):
    try:
        frozen = load(FROZEN)
    except OSError:
        problems.append("engine/tools/frozen-lints.toml is missing")
        return
    now = current_frozen()
    names = {
        "workspace-lints": "engine/Cargo.toml [workspace.lints]",
        "profile": "engine/Cargo.toml [profile.*]",
        "pins": "the four pins in engine/Cargo.toml [workspace.dependencies] (R-30, R-60)",
        "root-scripts": "the root package.json's engine:* scripts",
    }
    for key, what in names.items():
        if frozen.get(key, {}) != now[key]:
            detail = diff_keys(frozen.get(key, {}), now[key])
            problems.append(f"{what} differs from the frozen copy (R-80): {detail}")


def diff_keys(a, b, path=""):
    for k in sorted(set(a) | set(b)):
        p = f"{path}.{k}" if path else k
        if k not in b:
            return f"{p} removed"
        if k not in a:
            return f"{p} added"
        if isinstance(a[k], dict) and isinstance(b[k], dict):
            if a[k] != b[k]:
                return diff_keys(a[k], b[k], p)
        elif a[k] != b[k]:
            return f"{p} is {b[k]!r}, frozen as {a[k]!r}"
    return "(differs)"


def check_manifests(problems):
    ws_path = os.path.join(ENGINE, "Cargo.toml")
    ws = load(ws_path)
    members = ws.get("workspace", {}).get("members", [])
    for m in members:
        if not re.fullmatch(r"(crates|tests)/[A-Za-z0-9_-]+", m):
            problems.append(f"engine/Cargo.toml: member {m!r} isn't a folder under crates/ or tests/")
    for key in ("patch", "replace"):
        if key in ws:
            problems.append(f"engine/Cargo.toml: a [{key}] section (R-80)")
    if "cargo-features" in ws:
        problems.append("engine/Cargo.toml: cargo-features (unstable cargo) (R-80)")
    crates = os.path.join(ENGINE, "crates")
    on_disk = sorted(f"crates/{d}" for d in os.listdir(crates) if os.path.isfile(os.path.join(crates, d, "Cargo.toml")))
    for d in on_disk:
        if d not in members:
            problems.append(f"engine/{d} isn't a workspace member, so the workspace lints don't reach it (R-80)")
    for path in walk(ENGINE):
        if os.path.basename(path) != "Cargo.toml" or path == ws_path:
            continue
        r = rel(path)
        # the fuzz and gungraun instruments are workspaces of their own, the lead's (R-91, R-92, R-97)
        if r.startswith(("engine/fuzz/", "engine/spec/", "engine/tools/gungraun/")):
            continue
        m = load(path)
        for key in ("patch", "replace", "profile", "workspace", "cargo-features"):
            if key in m:
                problems.append(f"{r}: a [{key}] section (R-80)")
        pkg = m.get("package", {})
        if "build" in pkg:
            problems.append(f"{r}: package.build names a build script (R-80)")
        lints = m.get("lints")
        runner = r.startswith("engine/tests/")
        if lints != {"workspace": True}:
            if runner and lints is None:
                print(f"check-rules: notice: {r} doesn't inherit the workspace lints (T's choice)")
            else:
                problems.append(f"{r}: [lints] must be exactly `workspace = true` (R-80)")
        for bench in m.get("bench", []):
            p = bench.get("path", f"benches/{bench.get('name', '')}.rs")
            if not os.path.normpath(p).replace(os.sep, "/").startswith("benches/"):
                problems.append(f"{r}: bench {bench.get('name')!r} lives outside benches/ ({p}), where the builder could edit it (R-91)")
        tables = [m.get(t, {}) for t in DEP_TABLES]
        tables += [t.get(k, {}) for t in m.get("target", {}).values() for k in DEP_TABLES]
        if r.startswith("engine/crates/"):
            for t, v in m.get("target", {}).items():
                for k in DEP_TABLES:
                    if isinstance(v, dict) and v.get(k):
                        problems.append(f"{r}: [target.{t}.{k}]: a dependency for some targets only, which "
                                        "cargo-deny may not see; declare it for every target (R-52)")
            for table in tables:
                for name, spec in table.items():
                    if not (isinstance(spec, dict) and spec.get("workspace") is True):
                        problems.append(f"{r}: {name} is declared here, not in engine/Cargo.toml "
                                        "[workspace.dependencies]: a crate is declared once, centrally (R-59)")
        for table in tables:
            for name, spec in table.items():
                if isinstance(spec, dict) and "path" in spec:
                    target = os.path.realpath(os.path.join(os.path.dirname(path), spec["path"]))
                    if not target.startswith(ENGINE + os.sep):
                        problems.append(f"{r}: {name} is a path dependency outside engine/ (R-55)")
        if m.get("build-dependencies"):
            problems.append(f"{r}: build-dependencies, though no crate has a build script (R-80)")


# cargo commands the checks, the hooks and CI run, and their short forms: an alias with one
# of these names in a cargo config above the engine would run something else (R-82)
CARGO_VERBS = {
    "build", "b", "check", "c", "test", "t", "run", "r", "bench", "doc", "d", "fmt", "clippy", "fix",
    "tree", "metadata", "fetch", "update", "rustc", "rustdoc", "install", "nextest", "deny", "mutants",
    "fuzz", "llvm-cov", "pgo",
}


def cargo_config_problems(path):
    """What in a cargo config file outside the engine would change its build."""
    try:
        cfg = load(path)
    except (OSError, ValueError):
        return [f"{path}: an unreadable cargo config"]
    build = cfg.get("build", {})
    bad = [f"build.{k}" for k in ("rustflags", "rustdocflags", "rustc", "rustc-wrapper", "rustc-workspace-wrapper", "target-dir") if k in build]
    bad += [f"target.{t}.{k}" for t, v in cfg.get("target", {}).items() if isinstance(v, dict) for k in ("rustflags", "linker", "runner") if k in v]
    bad += [k for k in ("profile", "patch", "source", "unstable", "env", "include") if k in cfg]
    bad += [f"alias.{a}" for a in cfg.get("alias", {}) if a in CARGO_VERBS]
    return [f"{path}: sets {', '.join(bad)}, which changes the engine's build (R-82)"] if bad else []


def check_build_scripts_and_config(problems):
    seen = set()
    for path in walk(ENGINE):
        r = rel(path)
        name = os.path.basename(path)
        if r.startswith("engine/spec/"):
            continue
        if name == "build.rs":
            problems.append(f"{r}: a build script (R-80)")
        if name in CONFIG_NAMES or "/.cargo/" in f"/{r}":
            seen.add(r)
            if r not in EXPECTED_CONFIG:
                problems.append(f"{r}: an unexpected config file (R-80)")
    for r in sorted(EXPECTED_CONFIG - seen):
        problems.append(f"{r}: missing (R-80)")
    # cargo reads .cargo/config.toml in every folder above engine/, and rustfmt the
    # nearest rustfmt.toml; engine/ has neither of its own
    home = os.environ.get("CARGO_HOME") or os.path.join(os.path.expanduser("~"), ".cargo")
    folders, d = [], ROOT
    while True:
        folders.append(d)
        parent = os.path.dirname(d)
        if parent == d:
            break
        d = parent
    configs = {os.path.join(f, ".cargo", n) for f in folders for n in ("config.toml", "config")}
    configs |= {os.path.join(home, n) for n in ("config.toml", "config")}
    for p in sorted(configs):
        if os.path.isfile(p):
            problems += cargo_config_problems(p)
    for f in folders:
        for n in ("rustfmt.toml", ".rustfmt.toml"):
            if os.path.isfile(os.path.join(f, n)):
                problems.append(f"{os.path.join(f, n)}: a rustfmt config above engine/ that cargo fmt would read (R-80)")
    if os.environ.get("CLIPPY_CONF_DIR"):
        problems.append("CLIPPY_CONF_DIR is set: clippy would read another clippy.toml (R-78)")


def lex(text):
    """Rust source split into its code (comments and literals blanked, lines kept) and
    its string literals as (line, text)."""
    code, strings, i, n = [], [], 0, len(text)
    while i < n:
        c = text[i]
        if text.startswith("//", i):
            j = text.find("\n", i)
            j = n if j < 0 else j
            i = j
            continue
        if text.startswith("/*", i):
            depth, j = 1, i + 2
            while j < n and depth:
                if text.startswith("/*", j):
                    depth, j = depth + 1, j + 2
                elif text.startswith("*/", j):
                    depth, j = depth - 1, j + 2
                else:
                    j += 1
            code.append("\n" * text.count("\n", i, j))
            i = j
            continue
        m = re.compile(r'(?:b|c)?r(#*)"').match(text, i)
        if m and (i == 0 or not (text[i - 1].isalnum() or text[i - 1] == "_")):
            end = text.find('"' + m.group(1), m.end())
            end = n if end < 0 else end
            strings.append((text.count("\n", 0, i) + 1, text[m.end() : end]))
            code.append('""' + "\n" * text.count("\n", i, end))
            i = end + 1 + len(m.group(1))
            continue
        if c == '"':
            j = i + 1
            while j < n and text[j] != '"':
                j += 2 if text[j] == "\\" else 1
            strings.append((text.count("\n", 0, i) + 1, text[i + 1 : j]))
            code.append('""' + "\n" * text.count("\n", i, j))
            i = j + 1
            continue
        m = re.compile(r"'(?:[^'\\\n]|\\(?:u\{[0-9a-fA-F]+\}|x[0-9a-fA-F]{2}|.))'").match(text, i)
        if m:
            code.append("' '")
            i = m.end()
            continue
        code.append(c)
        i += 1
    return "".join(code), strings


def assert_args(code, start):
    """The number of arguments of the macro call whose bracket opens at code[start - 1],
    counting top-level commas (strings are already blanked) and ignoring a trailing one."""
    pairs = {"(": ")", "[": "]", "{": "}"}
    stack, commas, last = [pairs[code[start - 1]]], 0, ""
    for c in code[start:]:
        if c in pairs:
            stack.append(pairs[c])
        elif stack and c == stack[-1]:
            stack.pop()
            if not stack:
                break
        elif c == "," and len(stack) == 1:
            commas += 1
            last = ","
            continue
        if not c.isspace():
            last = c
    return commas + (last != ",")


def rust_files(top):
    return [p for p in walk(top) if p.endswith(".rs")]


def check_code_text(problems):
    for path in rust_files(os.path.join(ENGINE, "crates")) + rust_files(os.path.join(ENGINE, "tests")):
        text = open(path, encoding="utf-8").read()
        r = rel(path)
        if "mutants::" in text:
            problems.append(f"{r}: `mutants::` text; no mutation skips (R-86)")
        if "ast-grep-ignore" in text:
            problems.append(f"{r}: an ast-grep-ignore comment; no exception outside #[expect] (R-109)")
    # R-71: asserts carry messages; clippy's missing_assert_message skips tests, the one
    # place asserts are allowed (R-4), so this scan stands in for it
    for path in rust_files(os.path.join(ENGINE, "crates")) + rust_files(os.path.join(ENGINE, "tests")):
        code, _ = lex(open(path, encoding="utf-8").read())
        for m in ASSERT.finditer(code):
            if assert_args(code, m.end()) <= ASSERT_ARGS.get(m.group(1), 2):
                line = code.count("\n", 0, m.start()) + 1
                problems.append(f"{rel(path)}:{line}: {m.group(1)}! without a message (R-71)")
    # R-1: the workspace lint forbids unsafe code; each crate root says so as well
    for path in rust_files(os.path.join(ENGINE, "crates")):
        parts = rel(path).split("/")
        root = parts[3:] in (["src", "lib.rs"], ["src", "main.rs"]) or parts[3:5] == ["src", "bin"]
        if root and "#![forbid(unsafe_code)]" not in open(path, encoding="utf-8").read():
            problems.append(f"{rel(path)}: no #![forbid(unsafe_code)] at the crate root (R-1)")
    lib = os.path.join(ENGINE, "crates", "tally", "src")
    for path in rust_files(lib):
        text = open(path, encoding="utf-8").read()
        r = rel(path)
        if r == "engine/crates/tally/src/lib.rs":
            for feature in TEST_ONLY:
                text = text.replace(guard_text(feature), "")
        code, strings = lex(text)
        for pattern, why in FORBIDDEN_TEXT:
            if pattern.search(code):
                problems.append(f"{r}: {why}")
        for line, literal in strings:
            if PATH_STRING.search(literal):
                problems.append(f"{r}:{line}: a path string into the tests, the answer keys or the spec (R-93)")
    lib_rs = os.path.join(lib, "lib.rs")
    lib_text = open(lib_rs, encoding="utf-8").read() if os.path.isfile(lib_rs) else ""
    for feature in declared_test_only():
        if guard_text(feature) not in lib_text:
            problems.append(f"engine/crates/tally/src/lib.rs: the {feature} compile_error! guard is missing or changed (R-51)")


CLI_IO = os.path.join(TOOLS, "cli-io.toml")
CLI_WATCHED = ("std::fs", "std::env", "std::io", "std::process", "std::net")


def use_paths(tree, prefix=""):
    """The leaf paths of a `use` tree's text, braces expanded; `x as y` gives x."""
    tree = tree.strip()
    brace = tree.find("{")
    if brace < 0:
        leaf = tree.split(" as ")[0].strip()
        return [prefix + leaf] if leaf else []
    head, body = tree[:brace].strip(), tree[brace + 1 : tree.rindex("}")]
    parts, depth, cur = [], 0, ""
    for ch in body:
        depth += (ch == "{") - (ch == "}")
        if ch == "," and depth == 0:
            parts.append(cur)
            cur = ""
        else:
            cur += ch
    parts.append(cur)
    out = []
    for p in parts:
        out += use_paths(p, prefix + head)
    return [x.replace("self", "").rstrip(":") if x.endswith("::self") else x for x in out]


def check_cli_io(problems):
    """R-39 (audit F-19): main.rs names std's I/O items only from tools/cli-io.toml."""
    main_rs = os.path.join(ENGINE, "crates", "tally-cli", "src", "main.rs")
    if not os.path.isfile(main_rs):
        return
    allowed = set(load(CLI_IO).get("allowed", []))
    for p in sorted(allowed):
        if p.startswith(("std::process", "std::net")) and p != "std::process::ExitCode":
            problems.append(f"engine/tools/cli-io.toml: {p}: std::process and std::net give only the exit code (R-39)")
    code, _ = lex(open(main_rs, encoding="utf-8").read())
    named = []
    for m in re.finditer(r"\buse\s+((?:::)?(?:std|core)\b[^;]*);", code):
        named += use_paths(m.group(1).lstrip(":"))
    named += [m.group(1) for m in re.finditer(r"(?<![\w:])(?:::)?(std(?:::\w+)+)", code)]
    if re.search(r"\b(?:std|core)\s+as\b", code) or re.search(r"\bextern\s+crate\b", code):
        problems.append("engine/crates/tally-cli/src/main.rs: std renamed (`as`, `extern crate`), which hides what it names (R-39)")
    for p in sorted(set(named)):
        if p.startswith(CLI_WATCHED) and p not in allowed:
            problems.append(f"engine/crates/tally-cli/src/main.rs: {p} isn't on tools/cli-io.toml's list (R-39)")
    for p in sorted(set(named)):
        if p.endswith("::*"):
            problems.append(f"engine/crates/tally-cli/src/main.rs: a glob import ({p}) hides what it names (R-39)")


def known_crates():
    """The crates in Cargo.lock, and those the cargo-deny configs name (a ban may
    anticipate a crate that isn't a dependency yet, such as chrono)."""
    lock = load(os.path.join(ENGINE, "Cargo.lock"))
    names = {p["name"] for p in lock.get("package", [])}
    for cfg in ("deny.toml", "deny-tally.toml"):
        text = open(os.path.join(ENGINE, cfg), encoding="utf-8").read()
        names |= set(re.findall(r'\b(?:crate|name)\s*=\s*"([A-Za-z0-9_-]+)(?:@[^"]*)?"', text))
    return {n.replace("-", "_") for n in names}


def check_ban_paths(problems):
    cfg = load(os.path.join(ENGINE, "clippy.toml"))
    known = known_crates() | {"std", "core", "alloc"} | PRIMITIVES
    for key in ("disallowed-methods", "disallowed-types", "disallowed-macros"):
        for entry in cfg.get(key, []):
            p = entry if isinstance(entry, str) else entry.get("path", "")
            if isinstance(entry, dict) and "allow-invalid" in entry:
                problems.append(f"engine/clippy.toml {key}: {p!r} carries allow-invalid, which hides a path that doesn't resolve (R-78)")
            head = p.split("::", 1)[0]
            if head not in known:
                problems.append(f"engine/clippy.toml {key}: {p!r} starts with {head!r}, which is neither std, a primitive nor a crate in Cargo.lock or the deny configs (R-78)")


def check_num_bigint(problems):
    """num-bigint's arithmetic is the kernel's; only tally depends on it (R-30)."""
    lock = load(os.path.join(ENGINE, "Cargo.lock"))
    users = sorted(
        p["name"] for p in lock.get("package", [])
        if any(d.split()[0] == "num-bigint" for d in p.get("dependencies", []))
    )
    if users != ["tally"]:
        problems.append(f"engine/Cargo.lock: num-bigint is a dependency of {users or 'nothing'}, not of tally alone (R-30)")


def check_generated(problems):
    r = subprocess.run([sys.executable, "-I", "-B", os.path.join(TOOLS, "gen-clippy-ints.py"), "--check"],
                       capture_output=True, text=True, check=False)
    if r.returncode != 0:
        problems.append((r.stderr or r.stdout).strip() or "gen-clippy-ints --check failed")


def check_kernel(problems):
    try:
        k = load(KERNEL)
    except OSError:
        problems.append("engine/tools/kernel.toml is missing (R-85)")
        return
    if not k.get("set_at_s", False):
        print("check-rules: notice: the kernel files aren't set yet (the lead sets them at S, R-85); skipped")
        return
    kernel, protected = k.get("kernel", []), k.get("protected_non_kernel", [])
    src = os.path.join(ENGINE, "crates", "tally", "src")
    for path in rust_files(src):
        r = os.path.relpath(path, ENGINE).replace(os.sep, "/")
        if not any(fnmatch.fnmatchcase(r, g) for g in kernel + protected):
            problems.append(f"engine/{r}: in neither the kernel nor the protected non-kernel list (R-85)")
    mutants = load(os.path.join(ENGINE, ".cargo", "mutants.toml"))
    if sorted(mutants.get("examine_globs", [])) != sorted(kernel):
        problems.append("engine/.cargo/mutants.toml examine_globs differs from tools/kernel.toml's kernel (R-85)")


def main():
    args = sys.argv[1:]
    if args == ["--write-frozen"]:
        write_frozen()
        print(f"check-rules: wrote {rel(FROZEN)}")
        return 0
    if args:
        print("usage: engine/tools/check-rules [--write-frozen]", file=sys.stderr)
        return 64
    problems = []
    for check in (check_frozen_tables, check_manifests, check_build_scripts_and_config, check_code_text,
                  check_ban_paths, check_num_bigint, check_generated, check_kernel, check_cli_io):
        try:
            check(problems)
        except (OSError, ValueError, KeyError, TypeError) as e:
            problems.append(f"{check.__name__}: {type(e).__name__}: {e}")
    for p in problems:
        print(f"FAIL {p}")
    if problems:
        print(f"check-rules: {len(problems)} problem{'' if len(problems) == 1 else 's'}")
        return 1
    print("check-rules: the builder-edited files keep the rules")
    return 0


if __name__ == "__main__":
    sys.exit(main())
