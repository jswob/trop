"""advisory: the report-only measures (R-74, R-75). It never fails the build.

    engine/tools/advisory

It reports, for the review to read:
  - function length and nesting depth: clippy's too_many_lines, cognitive_complexity and
    excessive_nesting as warnings, in a pass of their own (the strict passes allow the
    first and leave the last off);
  - clone density: `.clone()` calls per 100 lines of code, per library file;
  - kernel functions whose doc comment cites no spec section (`/// L §7.2`), once the
    lead has set the kernel at S (tools/kernel.toml).

It writes into engine/target/advisory/ and always exits 0.
"""

import fnmatch
import json
import os
import re
import shutil
import subprocess
import sys
import tomllib

TOOLS = os.path.dirname(os.path.realpath(__file__))
ENGINE = os.path.dirname(TOOLS)
sys.path.insert(0, TOOLS)
import check  # noqa: E402  the cleaned environment
import check_rules  # noqa: E402  the lexer and the file walk

OUT = os.path.join(ENGINE, "target", "advisory")
LINTS = ["clippy::too_many_lines", "clippy::cognitive_complexity", "clippy::excessive_nesting"]
# the advisory pass's thresholds, in its own copy of clippy.toml, so the strict passes don't see them
THRESHOLDS = "\n# advisory pass only (R-74)\ntoo-many-lines-threshold = 60\ncognitive-complexity-threshold = 20\nexcessive-nesting-threshold = 4\n"
CITATION = re.compile(r"\b[A-Z][A-Z-]*(?:\.md)? §[0-9]+(?:\.[0-9]+)*")
FN = re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:const\s+)?(?:unsafe\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)")


def clippy_report():
    os.makedirs(OUT, exist_ok=True)
    conf = os.path.join(OUT, "clippy-conf")
    os.makedirs(conf, exist_ok=True)
    shutil.copyfile(os.path.join(ENGINE, "clippy.toml"), os.path.join(conf, "clippy.toml"))
    with open(os.path.join(conf, "clippy.toml"), "a", encoding="utf-8") as f:
        f.write(THRESHOLDS)
    env = check.clean_env()
    env["CLIPPY_CONF_DIR"] = conf
    env["CARGO_TARGET_DIR"] = os.path.join(OUT, "cargo")
    argv = ["cargo", "clippy", "--workspace", "--all-features", "--lib", "--bins", "--locked",
            "--message-format=json", "--", "--cap-lints", "warn"]
    for lint in LINTS:
        argv += ["-W", lint]
    r = subprocess.run(argv, cwd=ENGINE, env=env, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True, check=False)
    hits = set()
    for line in r.stdout.splitlines():
        try:
            msg = json.loads(line)
        except json.JSONDecodeError:
            continue
        m = msg.get("message") or {}
        code = (m.get("code") or {}).get("code")
        if msg.get("reason") != "compiler-message" or code not in LINTS:
            continue
        span = next((s for s in m.get("spans", []) if s.get("is_primary")), None)
        where = f"engine/{span['file_name']}:{span['line_start']}" if span else "?"
        hits.add(f"{where}: {code.removeprefix('clippy::')}: {m.get('message')}")
    print(f"advisory: function length and nesting ({', '.join(l.removeprefix('clippy::') for l in LINTS)}):")
    if r.returncode != 0 and not hits:
        print("  the clippy pass failed; see the strict passes")
    for h in sorted(hits):
        print(f"  {h}")
    if not hits:
        print("  none over the thresholds")


def library_files():
    return sorted(p for p in check_rules.rust_files(os.path.join(ENGINE, "crates")) if "/src/" in p)


def clone_density():
    print("advisory: clone density (.clone() per 100 lines of code):")
    rows = []
    for path in library_files():
        code, _ = check_rules.lex(open(path, encoding="utf-8").read())
        lines = sum(1 for line in code.splitlines() if line.strip())
        clones = len(re.findall(r"\.clone\(\)", code))
        if lines:
            rows.append((100 * clones / lines, clones, lines, check_rules.rel(path)))
    for density, clones, lines, r in sorted(rows, reverse=True):
        print(f"  {r}: {density:.1f} ({clones} in {lines} lines)")


def uncited():
    print("advisory: kernel functions without a spec citation (R-75):")
    try:
        with open(check_rules.KERNEL, "rb") as f:
            kernel = tomllib.load(f)
    except OSError:
        kernel = {}
    if kernel.get("set_at_s") is not True:
        print("  notice: the kernel isn't set yet (the lead sets it at S, R-85); skipped")
        return
    missing = 0
    for path in library_files():
        r = os.path.relpath(path, ENGINE).replace(os.sep, "/")
        if not any(fnmatch.fnmatchcase(r, g) for g in kernel.get("kernel", [])):
            continue
        doc = []
        for n, line in enumerate(open(path, encoding="utf-8").read().splitlines(), 1):
            stripped = line.strip()
            if stripped.startswith("///"):
                doc.append(stripped)
                continue
            m = FN.match(line)
            if m and not any(CITATION.search(d) for d in doc):
                print(f"  engine/{r}:{n}: {m.group(1)}")
                missing += 1
            if not stripped.startswith("#["):
                doc = []
    if not missing:
        print("  none")


def main():
    clippy_report()
    clone_density()
    uncited()
    return 0


if __name__ == "__main__":
    sys.exit(main())
