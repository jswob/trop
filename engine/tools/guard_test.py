"""Tests for the guard and the Stop hook (C §5, R-79 to R-83).

    engine/tools/guard-test

It copies the guard and stop-check into a throwaway repo (a temporary git tree with
the engine's layout), feeds them hook inputs as JSON on stdin, and checks each answer:
every role against every class of protected path, each Bash form, the build role's
extra refusals, the ConfigChange and Stop hooks, malformed input and a forced crash.
"""

import json
import os
import shutil
import subprocess
import sys
import tempfile

TOOLS = os.path.dirname(os.path.realpath(__file__))
ROLES = ("build", "rules", "tests", "spec", "unset")
HOME = os.path.expanduser("~")

SPEC = {"spec"}
TESTS = {"tests"}
RULES = {"rules"}
APP = {"rules", "unset"}
BUILD = {"build", "rules", "unset"}

# path (relative to the throwaway root, or absolute) -> the roles that may write it
EDITS = [
    ("engine/spec/DONE.md", SPEC),
    ("engine/spec/models/new.tally", SPEC),
    ("engine/SPEC/done.md", SPEC),
    ("engine/crates/tally/../../spec/DONE.md", SPEC),
    ("engine/tests/runner/src/lib.rs", TESTS),
    ("engine/tests/TESTS.sha256", TESTS),
    # the oracle: the spec's reference interpreter, written with the spec
    ("engine/tools/refcore/walk.py", SPEC),
    ("engine/tools/refcore/tests/L8_5_unread.test", SPEC),
    ("engine/tools/Refcore/WALK.py", SPEC),
    ("engine/tools/refcore_fast/layer.py", SPEC),
    ("engine/tools/SHA256SUMS", SPEC),
    ("engine/tools/frozen.toml", RULES),
    ("engine/clippy.toml", RULES),
    ("Engine/Clippy.toml", RULES),
    ("engine/crates/tally/clippy.toml", RULES),
    ("engine/crates/tally/src/.clippy.toml", RULES),
    ("engine/crates/tally/rustfmt.toml", RULES),
    ("engine/crates/tally/.rustfmt.toml", RULES),
    ("engine/crates/tally/rust-toolchain.toml", RULES),
    ("engine/crates/tally/.cargo/config.toml", RULES),
    ("engine/deny.toml", RULES),
    ("engine/deny-tally.toml", RULES),
    ("engine/sgconfig.yml", RULES),
    ("engine/rust-toolchain.toml", RULES),
    ("engine/.cargo/mutants.toml", RULES),
    ("engine/.cargo/config.toml", RULES),
    ("engine/.config/nextest.toml", RULES),
    ("engine/tools/guard.py", RULES),
    ("engine/tools/exceptions-budget.toml", RULES),
    ("engine/tools/RULES.sha256", RULES),
    ("engine/tools/ast-grep/rules/no-float.yml", RULES),
    ("engine/fuzz/fuzz_targets/parse.rs", RULES),
    ("engine/fuzz/regressions/parse/crash-1", RULES),
    ("engine/crates/tally/benches/stub.rs", RULES),
    (".github/workflows/engine.yml", RULES),
    (".claude/settings.json", RULES),
    ("engine/crates/tally/build.rs", APP),
    ("engine/crates/tally-cli/build.rs", APP),
    (".claude/settings.local.json", APP),
    (".claude/hooks/x.sh", APP),
    (".claude/agents/x.md", APP),
    (".cargo/config.toml", APP),
    ("clippy.toml", APP),
    ("package.json", APP),
    ("apps/web/src/x.ts", APP),
    ("engine/README.md", APP),
    ("engine/SCAFFOLD.md", APP),
    (os.path.join(HOME, ".claude", "settings.json"), APP),
    ("/tmp/tally-guard-test-scratch.txt", APP),
    ("engine/crates/tally/src/lib.rs", BUILD),
    ("engine/crates/tally/src/number/kernel.rs", BUILD),
    ("engine/crates/tally/Cargo.toml", BUILD),
    ("engine/crates/tally/tests/integration.rs", BUILD),
    ("engine/crates/tally-new/src/lib.rs", BUILD),
    ("engine/Cargo.toml", BUILD),
    ("engine/Cargo.lock", BUILD),
    ("engine/BUILD-LOG.md", BUILD),
    ("engine/MUTANTS-SURVIVORS.md", BUILD),
    ("engine/EXCEPTIONS.lock", BUILD),
    ("engine/target/debug/x", BUILD),
    ("engine/crates/tally/src/link-to-spec.rs", SPEC),
]

ALL = set(ROLES)
NOT_BUILD = {"rules", "tests", "spec", "unset"}
# (command, the roles that may run it, cwd relative to the root)
BASH = [
    # writing forms onto protected paths
    ("echo x > engine/spec/DONE.md", SPEC, ""),
    ("echo x >> engine/tests/x", TESTS, ""),
    ("echo x 1> engine/clippy.toml", RULES, ""),
    ("echo x &> engine/deny.toml", RULES, ""),
    ("echo x | tee engine/clippy.toml", RULES, ""),
    ("tee -a engine/deny.toml < /dev/null", RULES, ""),
    ("sed -i '' 's/a/b/' engine/sgconfig.yml", RULES, ""),
    ("sed -i.bak s/a/b/ engine/rust-toolchain.toml", RULES, ""),
    ("cp /tmp/x engine/spec/DONE.md", SPEC, ""),
    ("cp -t engine/tests /tmp/x", TESTS, ""),
    ("mv engine/tools/guard.py /tmp/", RULES, ""),
    ("rm -rf engine/spec", SPEC, ""),
    ("rm -rf engine", set(), ""),
    ("rm -rf engine/tools/refcore", SPEC, ""),
    ("rm -rf engine/tools", set(), ""),
    ("python3 -c \"open('engine/tools/refcore_fast/layer.py','w')\"", SPEC, ""),
    ("rm -rf .", set(), ""),
    ("touch engine/.cargo/config.toml", RULES, ""),
    ("ln -sf /tmp/x engine/.config/nextest.toml", RULES, ""),
    ("mkdir -p engine/crates/tally/benches/more", RULES, ""),
    ("chmod -x engine/tools/guard", RULES, ""),
    ("truncate -s 0 engine/spec/DONE.md", SPEC, ""),
    ("dd if=/dev/zero of=engine/spec/DONE.md count=1", SPEC, ""),
    ("git checkout HEAD -- engine/spec/DONE.md", SPEC, ""),
    ("git restore engine/tests/x", TESTS, ""),
    ("git reset HEAD engine/clippy.toml", RULES, ""),
    ("git -C engine checkout -- spec", SPEC, ""),
    ("git reset --hard HEAD~3", NOT_BUILD, ""),
    ("git checkout main", NOT_BUILD, ""),
    ("git stash", NOT_BUILD, ""),
    ("git apply /tmp/x.patch", NOT_BUILD, ""),
    ("git checkout -- engine/crates/tally/src/lib.rs", BUILD, ""),
    ("chmod 644 engine/clippy.toml", RULES, ""),
    ("chown me engine/spec/DONE.md", SPEC, ""),
    ("touch -t 202601010000 engine/tests/x", TESTS, ""),
    ("python3 -c \"open('engine/spec/DONE.md','w').write('x')\"", SPEC, ""),
    ("perl -pi -e 's/a/b/' engine/deny.toml", RULES, ""),
    ("node -e \"require('fs').writeFileSync('.claude/settings.json','{}')\"", RULES, ""),
    ("cd engine/spec && rm DONE.md", SPEC, ""),
    ("cd engine && echo x > spec/DONE.md", SPEC, ""),
    ("rm DONE.md", SPEC, "engine/spec"),
    ("bash -c 'echo x > engine/spec/DONE.md'", SPEC, ""),
    ("sh -c \"rm engine/tools/guard\"", RULES, ""),
    ("eval 'rm engine/clippy.toml'", RULES, ""),
    ("find engine/spec -name '*.md' -delete", SPEC, ""),
    ("find engine/tests -exec rm {} \\;", TESTS, ""),
    ("ls engine/spec | xargs rm", SPEC, ""),
    ("cat > engine/clippy.toml <<'EOF'\nit's a rule\nEOF", RULES, ""),
    ("rm engine/SPEC/done.md", SPEC, ""),
    ("env FOO=1 rm engine/spec/DONE.md", SPEC, ""),
    ("sudo rm engine/spec/DONE.md", SPEC, ""),
    ("timeout 5 rm engine/spec/DONE.md", SPEC, ""),
    # reading and scratch work stays free
    ("awk '{print > \"engine/spec/x\"}' /dev/null", SPEC, ""),
    ("python3 - <<'EOF'\nopen('engine/spec/DONE.md', 'w').write('x')\nEOF", SPEC, ""),
    ("bash <<'EOF'\nrm engine/tests/x\nEOF", TESTS, ""),
    ("cat <<'EOF' | sh\ncd engine && rm clippy.toml\nEOF", RULES, ""),
    ("cat engine/spec/DONE.md 2>&1 | head", ALL, ""),
    ("python3 -I engine/spec/tools/loadcheck.py --selftest", ALL, ""),
    ("bun run engine:check", ALL, ""),
    ("bun engine:spec-check", ALL, ""),
    ("sed -i '' 's/a/b/' /tmp/tally-guard-sed.txt", ALL, ""),
    ("sed -n 's/a/b/p' engine/spec/DONE.md", ALL, ""),
    ("grep -rn token engine/spec > /dev/null", ALL, ""),
    ("cp engine/spec/DONE.md /tmp/tally-guard-copy.md", ALL, ""),
    ("cargo build 2>&1 | tee /tmp/tally-guard-log.txt", ALL, ""),
    ("find engine/spec -name '*.md'", ALL, ""),
    ("git diff -- engine/spec", ALL, ""),
    ("git commit -m 'Add the parser'", ALL, ""),
    ("cat <<'EOF' > /tmp/tally-guard-note.txt\nrm -rf engine/spec; it's only text\nEOF", ALL, ""),
    # the build role's own places
    ("echo x > engine/crates/tally/src/lib.rs", BUILD, ""),
    ("cargo clippy 2>&1 | tee engine/target/clippy.log", BUILD, ""),
    ("echo '## STOP-REPORT' >> engine/BUILD-LOG.md", BUILD, ""),
    ("rm -rf engine/crates/tally", RULES, ""),  # it holds benches/, a rule folder
    ("rm -rf engine/crates/tally/src", BUILD, ""),
    ("echo x > engine/crates/tally/build.rs", APP, ""),
    # the build role's extra refusals (R-81, R-100)
    ("jq '.disableAllHooks=true' .claude/settings.local.json > /tmp/s.json", NOT_BUILD, ""),
    ("claude -p 'edit the spec'", NOT_BUILD, ""),
    ("npx @anthropic-ai/claude-code --settings '{}'", NOT_BUILD, ""),
    ("TALLY_ROLE=rules engine/tools/guard < /tmp/x", NOT_BUILD, ""),
    ("git commit -m 'Add parser\n\nCo-Authored-By: Claude <noreply@anthropic.com>'", {"unset"}, ""),
    ("git commit -F - <<'EOF'\nAdd parser\n\nCo-authored-by: an AI\nEOF", {"unset"}, ""),
    ("echo x > ~/.claude/settings.json", NOT_BUILD, ""),
    ("echo x > \"$TALLY_GUARD_UNDEFINED_VAR\"", NOT_BUILD, ""),
    ("echo \"unbalanced", NOT_BUILD, ""),
    ("echo x > ~/.cargo/config.toml", NOT_BUILD, ""),
]


def setup(root):
    for d in ("engine/tools", "engine/spec", "engine/tests", "engine/crates/tally/src",
              "engine/crates/tally/benches", "engine/crates/tally-cli/src", ".claude"):
        os.makedirs(os.path.join(root, d), exist_ok=True)
    for name in ("guard", "guard.py", "stop-check", "stop_check.py"):
        shutil.copy2(os.path.join(TOOLS, name), os.path.join(root, "engine/tools", name))
    files = {
        "engine/spec/DONE.md": "the contract\n",
        "engine/tests/x": "a test\n",
        "engine/crates/tally/src/lib.rs": "//! stub\n",
        "engine/crates/tally/benches/stub.rs": "//! bench\n",
        "engine/BUILD-LOG.md": "# Build log\n",
        ".claude/settings.json": "{}\n",
    }
    for path, text in files.items():
        with open(os.path.join(root, path), "w", encoding="utf-8") as f:
            f.write(text)
    os.symlink("../../../spec/DONE.md", os.path.join(root, "engine/crates/tally/src/link-to-spec.rs"))
    with open(os.path.join(root, "engine/tools/check"), "w", encoding="utf-8") as f:
        f.write('#!/usr/bin/env bash\n'
                'case "${FAKE_CHECK:-green}" in\n'
                '  green) echo "check fast: green"; exit 0 ;;\n'
                '  red) echo "FAIL clippy-lib: error[E0001] something"; exit 1 ;;\n'
                '  red2) echo "FAIL fmt: Diff in lib.rs"; exit 1 ;;\n'
                'esac\n')
    os.chmod(os.path.join(root, "engine/tools/check"), 0o755)
    git = ["git", "-C", root, "-c", "user.name=guard-test", "-c", "user.email=guard-test@example.invalid"]
    subprocess.run(["git", "init", "-q", root], check=True)
    subprocess.run([*git, "add", "-A"], check=True)
    subprocess.run([*git, "commit", "-q", "-m", "fixture"], check=True)


def call(root, args, payload, role, extra_env=None, raw=None):
    env = {k: v for k, v in os.environ.items() if k not in ("TALLY_ROLE", "TALLY_GUARD_SELFTEST_CRASH", "FAKE_CHECK")}
    if role != "unset":
        env["TALLY_ROLE"] = role
    env.update(extra_env or {})
    data = raw if raw is not None else json.dumps(payload)
    r = subprocess.run([os.path.join(root, "engine/tools", args[0]), *args[1:]], input=data,
                       capture_output=True, text=True, env=env, check=False)
    return r.returncode, r.stderr


def main():
    failures = []
    total = 0

    def expect(label, got, want, stderr=""):
        nonlocal total
        total += 1
        if got != want:
            failures.append(f"{label}: exit {got}, want {want}  {stderr.strip()[:200]}")

    with tempfile.TemporaryDirectory(prefix="tally-guard-test-") as tmp:
        root = os.path.realpath(tmp)
        setup(root)
        # Edit, Write, MultiEdit and NotebookEdit: every role against every path
        tools = ("Edit", "Write", "MultiEdit", "NotebookEdit")
        for i, (path, allowed) in enumerate(EDITS):
            full = path if os.path.isabs(path) else os.path.join(root, path)
            for j, role in enumerate(ROLES):
                tool = tools[(i + j) % len(tools)]  # each tool meets every role and path class
                key = "notebook_path" if tool == "NotebookEdit" else "file_path"
                code, err = call(root, ["guard"], {"tool_name": tool, "tool_input": {key: full}, "cwd": root}, role)
                expect(f"{tool} {path} as {role}", code, 0 if role in allowed else 2, err)
        # a relative path resolves against the hook's cwd
        for role in ROLES:
            code, err = call(root, ["guard"], {"tool_name": "Edit", "tool_input": {"file_path": "spec/DONE.md"},
                                               "cwd": os.path.join(root, "engine")}, role)
            expect(f"Edit spec/DONE.md from engine/ as {role}", code, 0 if role == "spec" else 2, err)
        # Bash: each form, every role
        for command, allowed, cwd in BASH:
            for role in ROLES:
                code, err = call(root, ["guard"], {"tool_name": "Bash", "tool_input": {"command": command},
                                                   "cwd": os.path.join(root, cwd)}, role)
                expect(f"Bash `{command}` as {role}", code, 0 if role in allowed else 2, err)
        # malformed input, unknown roles and tools, and a forced crash: all refuse
        edit = {"tool_name": "Edit", "tool_input": {"file_path": os.path.join(root, "engine/crates/tally/src/lib.rs")}, "cwd": root}
        code, err = call(root, ["guard"], edit, "unset", {"TALLY_ROLE": "bulid"})
        expect("unknown role refuses a harmless edit", code, 2, err)
        for label, raw in (("not JSON", "{oops"), ("empty input", ""), ("a JSON list", "[]")):
            code, err = call(root, ["guard"], None, "build", raw=raw)
            expect(f"{label} refuses", code, 2, err)
        code, err = call(root, ["guard"], {"tool_name": "Delete", "tool_input": {"file_path": "x"}, "cwd": root}, "unset")
        expect("unknown tool refuses", code, 2, err)
        code, err = call(root, ["guard"], {"tool_name": "Edit", "tool_input": {}, "cwd": root}, "unset")
        expect("Edit without a path refuses", code, 2, err)
        for role in ROLES:
            code, err = call(root, ["guard"], edit, role, {"TALLY_GUARD_SELFTEST_CRASH": "1"})
            expect(f"forced internal failure as {role} refuses", code, 2, err)
            if "internal error" not in err:
                failures.append(f"forced internal failure as {role}: no 'internal error' message: {err.strip()[:200]}")
        code, err = call(root, ["guard", "--bogus"], edit, "unset")
        expect("unknown argument refuses", code, 2, err)
        # ConfigChange: blocked and logged in build, free elsewhere
        log = os.path.join(root, "engine/BUILD-LOG.md")
        event = {"hook_event_name": "ConfigChange", "source": "local_settings", "file_path": os.path.join(root, ".claude/settings.local.json")}
        before = open(log, encoding="utf-8").read()
        code, err = call(root, ["guard", "--config-change"], event, "build")
        expect("ConfigChange as build", code, 2, err)
        after = open(log, encoding="utf-8").read()
        if "CONFIG-CHANGE BLOCKED" not in after[len(before):]:
            failures.append("ConfigChange as build: no CONFIG-CHANGE BLOCKED line in BUILD-LOG.md")
        total += 1
        for role in ("rules", "tests", "spec", "unset"):
            code, err = call(root, ["guard", "--config-change"], event, role)
            expect(f"ConfigChange as {role}", code, 0, err)
        if open(log, encoding="utf-8").read() != after:
            failures.append("ConfigChange outside build wrote to BUILD-LOG.md")
        total += 1
        # Stop hook
        stop = {"hook_event_name": "Stop", "stop_hook_active": False}
        for role in ("rules", "tests", "spec", "unset"):
            code, err = call(root, ["stop-check"], stop, role, {"FAKE_CHECK": "red"})
            expect(f"Stop as {role} with check red", code, 0, err)
        code, err = call(root, ["stop-check"], stop, "build", {"FAKE_CHECK": "green"})
        expect("Stop as build with check green", code, 0, err)
        for n in (1, 2, 3):
            code, err = call(root, ["stop-check"], stop, "build", {"FAKE_CHECK": "red"})
            expect(f"Stop as build, red, block {n}", code, 2, err)
        code, err = call(root, ["stop-check"], stop, "build", {"FAKE_CHECK": "red"})
        expect("Stop as build, red, fourth identical stop lets go", code, 0, err)
        if "STOPPED RED" not in open(log, encoding="utf-8").read():
            failures.append("Stop: no STOPPED RED line after three identical blocks")
        total += 1
        code, err = call(root, ["stop-check"], stop, "build", {"FAKE_CHECK": "red"})
        expect("Stop as build, red, counting restarts", code, 2, err)
        code, err = call(root, ["stop-check"], stop, "build", {"FAKE_CHECK": "red2"})
        expect("Stop as build, a different failure blocks afresh", code, 2, err)
        if "block 1 of 3" not in err:
            failures.append(f"Stop: a different failure didn't restart the count: {err.strip()[:200]}")
        total += 1
        with open(log, "a", encoding="utf-8") as f:
            f.write("\n## STOP-REPORT 2026-10-22: the spec seems wrong\n")
        code, err = call(root, ["stop-check"], stop, "build", {"FAKE_CHECK": "red"})
        expect("Stop as build, red, with a new STOP-REPORT", code, 0, err)
        git = ["git", "-C", root, "-c", "user.name=guard-test", "-c", "user.email=guard-test@example.invalid"]
        subprocess.run([*git, "commit", "-q", "-am", "report"], check=True)
        code, err = call(root, ["stop-check"], stop, "build", {"FAKE_CHECK": "red"})
        expect("Stop as build, red, STOP-REPORT in the last commit", code, 0, err)
        subprocess.run([*git, "commit", "-q", "--allow-empty", "-m", "later work"], check=True)
        code, err = call(root, ["stop-check"], stop, "build", {"FAKE_CHECK": "red"})
        expect("Stop as build, red, STOP-REPORT older than the last commit", code, 2, err)

    for f in failures:
        print("FAIL " + f)
    print(f"guard-test: {total} checks, {len(failures)} failed")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
