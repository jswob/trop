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
REPO = os.path.dirname(os.path.dirname(TOOLS))
ROLES = ("build", "rules", "tests", "spec", "unset", "lead")
HOME = os.path.expanduser("~")

SPEC = {"spec"}
TESTS = {"tests"}
RULES = {"rules"}
APP = {"rules", "unset", "lead"}
BUILD = {"build", "rules", "unset", "lead"}
EVERY = set(ROLES)
MASTER = {"unset", "lead"}  # app work, which the master session is, and the lead, which writes like it (issue #18)
LEAD = {"lead"}

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
    # the milestone marker: which milestones' tests gate (issue #14); the build role can't
    # gate or ungate its own milestone, nor T
    ("engine/tools/milestones.toml", RULES),
    ("engine/tools/milestones.py", RULES),
    ("engine/tools/guard.py", RULES),
    # the lead's tools are rule files: the lead runs them, never edits them (issue #18)
    ("engine/tools/merge-pr", RULES),
    ("engine/tools/merge_pr.py", RULES),
    ("engine/tools/worktree-done", RULES),
    ("engine/tools/worktree_done.py", RULES),
    ("engine/tools/exceptions-budget.toml", RULES),
    ("engine/tools/RULES.sha256", RULES),
    ("engine/tools/ast-grep/rules/no-float.yml", RULES),
    ("engine/fuzz/fuzz_targets/parse.rs", RULES),
    ("engine/fuzz/regressions/parse/crash-1", RULES),
    ("engine/crates/tally/benches/p8_m1.rs", RULES),
    (".github/workflows/engine.yml", RULES),
    (".claude/settings.json", RULES),
    # the builder's instructions (F-23)
    ("engine/CLAUDE.md", RULES),
    (".claude/skills/engine-rust/SKILL.md", RULES),
    (".claude/skills/engine-rust/references/numbers.md", RULES),
    (".claude/agents/engine-reviewer.md", RULES),
    # the rules' full texts (issue #3)
    ("engine/RULEBOOK.md", RULES),
    ("engine/BUILDER.md", RULES),
    ("engine/rulebook.md", RULES),
    (".claude/skills/design/SKILL.md", APP),
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
    # tmp/ is scratch for every role (issue #11), but the sealed exam, its record and
    # the master's briefs, which only app work (the master) writes
    ("tmp/t1/notes.md", EVERY),
    ("tmp/i11/x.md", EVERY),
    ("tmp/x.md", EVERY),
    ("tmp/T1/Notes.md", EVERY),
    ("tmp/holdout-notes/x.md", EVERY),
    ("tmp/active-notes/x.md", EVERY),
    ("tmp/holdout/x.md", MASTER),
    ("tmp/holdout", MASTER),
    ("tmp/HOLDOUT/x.md", MASTER),
    ("tmp/cases-holdout/x.md", MASTER),
    ("tmp/Cases-Holdout/x.md", MASTER),
    ("tmp/active/issue5-brief.md", MASTER),
    ("tmp/active", MASTER),
    ("tmp/Active/x.md", MASTER),
    ("tmp/t1/../active/x.md", MASTER),
    # the real path counts: a symlink in tmp/ reaches no protected file
    ("tmp/notes/link-to-active/x.md", MASTER),
    ("tmp/notes/link-to-holdout/x.md", MASTER),
    ("tmp/notes/link-to-brief.md", MASTER),
    ("tmp/notes/link-to-spec.md", SPEC),
    ("tmp/notes/link-to-tests/x", TESTS),
    ("tmp/notes/link-to-tools/guard.py", RULES),
]

ALL = set(ROLES)
NOT_BUILD = {"rules", "tests", "spec", "unset", "lead"}
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
    # the rules' full texts (issue #3)
    ("sed -i '' 's/a/b/' engine/RULEBOOK.md", RULES, ""),
    ("echo x >> engine/BUILDER.md", RULES, ""),
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
    ("echo x > ~/.claude.json", NOT_BUILD, ""),
    ("mkdir -p ../.cargo && cp /tmp/x ../.cargo/config.toml", NOT_BUILD, ""),  # F-9
    ("echo x > ../../.cargo/config.toml", NOT_BUILD, "engine"),
    ("echo x > /tmp/tally-guard-test-scratch.txt", ALL, ""),
    # F-2: shell syntax around the command word
    ("{ cp /tmp/x engine/spec/DONE.md; }", SPEC, ""),
    ("if true; then cp /tmp/x engine/spec/DONE.md; fi", SPEC, ""),
    ("if false; then :; else rm engine/tests/x; fi", TESTS, ""),
    ("for f in engine/spec/*.md; do rm \"$f\"; done", SPEC, ""),
    ("while true; do rm engine/tests/x; done", TESTS, ""),
    ("! rm engine/tests/x", TESTS, ""),
    ("echo `cp /tmp/x engine/spec/DONE.md`", SPEC, ""),
    ("echo \"$(cp /tmp/x engine/spec/DONE.md)\"", SPEC, ""),
    ("x=$(rm engine/clippy.toml);", RULES, ""),
    ("echo x > >(tee engine/spec/DONE.md)", SPEC, ""),
    ("diff <(rm engine/tests/x) /dev/null", TESTS, ""),
    ("nice -n 5 rm engine/spec/DONE.md", SPEC, ""),
    ("env -u HOME rm engine/spec/DONE.md", SPEC, ""),
    ("env -S 'rm engine/spec/DONE.md'", SPEC, ""),
    ("env -C engine rm spec/DONE.md", SPEC, ""),
    ("timeout -s KILL 5 rm engine/spec/DONE.md", SPEC, ""),
    ("timeout --signal=KILL 5 rm engine/tests/x", TESTS, ""),
    ("ls engine/spec | xargs -n 1 rm", SPEC, ""),
    ("xargs -I {} cp {} engine/spec/DONE.md < /dev/null", SPEC, ""),
    ("stdbuf -o0 cp /tmp/x engine/spec/DONE.md", SPEC, ""),
    ("sudo -u root rm engine/spec/DONE.md", SPEC, ""),
    ("flock /tmp/tally-guard-lock rm engine/spec/DONE.md", SPEC, ""),
    ("flock -n /tmp/tally-guard-lock -c 'rm engine/tests/x'", TESTS, ""),
    ("ionice -c 3 rm engine/tests/x", TESTS, ""),
    ("caffeinate -i rm engine/tests/x", TESTS, ""),
    ("unbuffer rm engine/tests/x", TESTS, ""),
    ("echo `date` > /tmp/tally-guard-date.txt", ALL, ""),
    ("echo \"$(cat engine/spec/DONE.md)\" | head", ALL, ""),
    # F-3: inline code is the script, so the operands after it are checked
    ("perl -pi -e 's/a/b/; s/c/d/' engine/spec/DONE.md", SPEC, ""),
    ("perl -i -ne 'print' engine/deny.toml", RULES, ""),
    ("perl -pi -es/a/b/ engine/deny.toml", RULES, ""),
    ("ruby -i -pe 'gsub(/a/, \"b\")' engine/tests/x", TESTS, ""),
    ("python3 -c \"import sys; open(sys.argv[1], 'w')\" engine/clippy.toml", RULES, ""),
    ("node -e \"require('fs').writeFileSync(process.argv[1], '')\" engine/spec/DONE.md", SPEC, ""),
    ("sed -i --expression='s/a/b/' engine/spec/DONE.md", SPEC, ""),
    ("sed -i -es/a/b/ engine/spec/DONE.md", SPEC, ""),
    ("python3 -c 'print(1)'", ALL, ""),
    ("perl -ne 'print' /tmp/tally-guard-x.txt", ALL, ""),
    # issue #15, kept: a bare word that names a protected folder from the cwd is that
    # folder, whatever it means to the program (`fix.py engine` may write there); an
    # option's attached value isn't read, so T's runner takes `--interp=engine`
    ("python3 -I -B engine/tests/run.py --interp engine", set(), ""),
    ("python3 -I -B tmp/t1/fix.py engine", set(), ""),
    ("python3 -I -B fix.py tests", TESTS, "engine"),
    ("python3 -I -B run.py --interp engine", TESTS, "engine/tests"),
    ("python3 -I -B engine/tests/run.py --interp=engine", ALL, ""),
    ("python3 -I -B engine/tests/run.py --interp refcore", ALL, ""),
    # and inline code, a here-document's too, can't tell a read from a write: naming a
    # path the role can't write refuses it; a script in a file isn't opened
    ("python3 - <<'EOF'\nprint(open('engine/spec/BUILD-ORDER.md').read())\nEOF", SPEC, ""),
    ("python3 -I -B tmp/t1/read.py", ALL, ""),
    # F-4: writers and their targets
    ("rustfmt engine/fuzz/fuzz_targets/parse.rs", RULES, ""),
    ("rustfmt --edition 2024 engine/crates/tally/src/lib.rs", BUILD, ""),
    ("rustfmt --check engine/fuzz/fuzz_targets/parse.rs", ALL, ""),
    ("cargo fmt --manifest-path fuzz/Cargo.toml", RULES, "engine"),
    ("cargo fmt", set(), "engine"),  # the workspace holds T's runner
    ("cargo fmt --all", set(), "engine"),
    ("cargo fmt -p tally -p tally-cli", BUILD, "engine"),
    ("cargo fmt", BUILD, "engine/crates/tally"),
    ("cargo fmt -p tally-runner", TESTS, "engine"),
    ("cargo fmt --all --check", ALL, "engine"),
    ("cargo fmt -- --check", ALL, "engine"),
    ("cargo clippy --fix --allow-dirty --all-targets", set(), "engine"),
    ("cargo clippy --fix --allow-dirty -p tally", BUILD, "engine"),
    ("cargo clippy --all-targets -- -D warnings", ALL, "engine"),
    ("cargo fix --allow-dirty -p tally-cli", BUILD, "engine"),
    ("cargo add serde --manifest-path crates/tally/Cargo.toml", BUILD, "engine"),
    ("cargo add serde -p tally-runner", TESTS, "engine"),
    ("cargo remove serde", BUILD, "engine/crates/tally"),
    ("cargo +nightly fmt --manifest-path engine/fuzz/Cargo.toml", RULES, ""),
    ("tar -xf /tmp/x.tar -C engine/spec", SPEC, ""),
    ("tar xzf /tmp/x.tgz --directory=engine/tests", TESTS, ""),
    ("tar -xf /tmp/x.tar", SPEC, "engine/spec"),
    ("tar -czf engine/spec/x.tgz /tmp/y", SPEC, ""),
    ("tar -tf /tmp/x.tar", ALL, ""),
    ("tar -cf /tmp/tally-guard.tar engine/spec", ALL, ""),
    ("unzip -o /tmp/x.zip -d engine/tests", TESTS, ""),
    ("unzip /tmp/x.zip", TESTS, "engine/tests"),
    ("unzip -l /tmp/x.zip", ALL, ""),
    ("curl -sSo engine/spec/DONE.md https://example.invalid/x", SPEC, ""),
    ("curl --output engine/tests/x https://example.invalid/x", TESTS, ""),
    ("curl -O https://example.invalid/DONE.md", SPEC, "engine/spec"),
    ("curl -s https://example.invalid/x > /tmp/tally-guard-x.txt", ALL, ""),
    ("wget -O engine/spec/DONE.md https://example.invalid/x", SPEC, ""),
    ("wget https://example.invalid/x", TESTS, "engine/tests"),
    ("sort -o engine/spec/DONE.md /tmp/x", SPEC, ""),
    ("sort -uo engine/tests/x /tmp/x", TESTS, ""),
    ("sort --output=engine/clippy.toml /tmp/x", RULES, ""),
    ("sort engine/spec/DONE.md | head", ALL, ""),
    ("uniq /tmp/x engine/tests/x", TESTS, ""),
    ("uniq engine/spec/DONE.md", ALL, ""),
    ("ditto /tmp/x engine/spec/DONE.md", SPEC, ""),
    ("vim -es -c '%s/a/b/' -c 'wq' engine/spec/DONE.md", SPEC, ""),
    ("ex -sc '%s/a/b/|x' engine/clippy.toml", RULES, ""),
    ("vim -es -c 'w engine/spec/DONE.md' -c 'q' /tmp/x", SPEC, ""),
    ("ed -s engine/tests/x < /tmp/x.ed", TESTS, ""),
    ("patch -p1 < /tmp/x.diff", NOT_BUILD, ""),
    ("patch -i /tmp/x.diff", NOT_BUILD, ""),
    ("patch engine/crates/tally/src/lib.rs /tmp/x.diff", BUILD, ""),
    # F-5: a hard link is a second name for its source
    ("ln engine/spec/DONE.md engine/crates/tally/src/h.md", set(), ""),
    ("ln engine/spec/DONE.md /tmp/tally-guard-h", SPEC, ""),
    ("cp -l engine/spec/DONE.md /tmp/tally-guard-h", SPEC, ""),
    ("cp --link engine/tests/x /tmp/tally-guard-h", TESTS, ""),
    ("ln -s engine/spec/DONE.md /tmp/tally-guard-h", ALL, ""),
    ("ln -sf engine/spec/DONE.md /tmp/tally-guard-h", ALL, ""),
    # F-8: R-100 reads the message file and refuses a computed message
    ("git commit -F engine/target/msg-ai.txt", {"unset"}, ""),
    ("git commit -aF engine/target/msg-ai.txt", {"unset"}, ""),
    ("git commit --file=engine/target/msg-ai.txt", {"unset"}, ""),
    ("git commit -F msg-ai.txt", {"unset"}, "engine/target"),
    ("git commit -F engine/target/msg-ok.txt", ALL, ""),
    ("git commit -F engine/target/missing.txt", {"unset"}, ""),
    ("git commit -m \"$(cat engine/target/msg-ok.txt)\"", {"unset"}, ""),
    ("M=$(cat engine/target/msg-ok.txt); git commit -m \"$M\"", {"unset"}, ""),
    ("git commit -m 'Rename `parse` to `read`; costs $0'", ALL, ""),
    ("cat engine/target/msg-ok.txt | git commit -F -", {"unset"}, ""),
    ("git commit -F - <<'EOF'\nAdd the parser\nEOF", ALL, ""),
    ("git commit -m \"$(cat <<'EOF'\nAdd the parser\nEOF\n)\"", ALL, ""),
    ("git commit -m \"$(cat <<'EOF'\nAdd the parser\n\nCo-Authored-By: Claude <noreply@anthropic.com>\nEOF\n)\"", {"unset"}, ""),
    ("git merge -m 'Merge\n\nCo-Authored-By: Claude <noreply@anthropic.com>' feature", {"unset"}, ""),
    ("git merge --no-ff -m 'Merge feature' feature", NOT_BUILD, ""),
    ("git tag -a v1 -F engine/target/msg-ai.txt", {"unset"}, ""),
    ("git tag -a v1 -m 'Release'", ALL, ""),
    ("git notes add -m 'Generated with Claude Code' HEAD", {"unset"}, ""),
    # R-132: no session pushes to master, deletes it, force-pushes it or merges a PR;
    # every role refused, unset included (the throwaway has master checked out)
    ("git push origin master", set(), ""),
    ("git push origin Master", set(), ""),
    ("git push origin HEAD:master", set(), ""),
    ("git push origin HEAD:refs/heads/master", set(), ""),
    ("git push origin feature:master", set(), ""),
    ("git push origin +master", set(), ""),
    ("git push origin +HEAD:master", set(), ""),
    ("git push -f origin master", set(), ""),
    ("git push --force origin master", set(), ""),
    ("git push --force-with-lease origin master", set(), ""),
    ("git push --force-with-lease=master:abc123 origin HEAD:master", set(), ""),
    ("git push origin :master", set(), ""),
    ("git push origin --delete master", set(), ""),
    ("git push -d origin master", set(), ""),
    ("git push --all origin", set(), ""),
    ("git push --mirror origin", set(), ""),
    ("git push --branches origin", set(), ""),
    ("git push --dry-run origin HEAD:master", set(), ""),
    ("git push -o ci.skip origin master", set(), ""),
    ("git push --repo=origin master", set(), ""),
    ("git push origin feature master", set(), ""),
    ("git -C engine push origin master", set(), ""),
    ("cd engine && git push origin master", set(), ""),
    ("bash -c 'git push origin master'", set(), ""),
    ("echo \"$(git push origin HEAD:master)\"", set(), ""),
    ("B=master; git push origin \"$B\"", set(), ""),
    ("git push", set(), ""),
    ("git push origin", set(), ""),
    ("git push -u origin HEAD", set(), "engine"),
    ("git push --follow-tags", set(), ""),
    ("gh pr merge 3", set(), ""),
    ("gh pr merge --squash --auto 3", set(), ""),
    ("gh -R jswob/trop pr merge 3", set(), ""),
    ("gh api -X PUT repos/jswob/trop/pulls/3/merge", set(), ""),
    ("gh api repos/jswob/trop/branches/master/protection -X DELETE", set(), ""),
    ("gh api -X PUT repos/jswob/trop/branches/master/protection --input /tmp/p.json", set(), ""),
    ("gh api -X PATCH repos/jswob/trop/git/refs/heads/master -f sha=abc -F force=true", set(), ""),
    ("gh api repos/jswob/trop/merges -f base=master -f head=feature", set(), ""),
    ("gh api graphql -f query='mutation { mergePullRequest(input: {pullRequestId: \"x\"}) { clientMutationId } }'", set(), ""),
    # R-132's controls: a branch, a tag, the PR's own commands
    ("git push -u origin rules/rulebook-home", ALL, ""),
    ("git push origin feature:feature", ALL, ""),
    ("git push origin master-notes", ALL, ""),
    ("git push origin v1", ALL, ""),
    ("git push origin refs/tags/v1", ALL, ""),
    ("git push origin --tags", ALL, ""),
    ("git push -o ci.skip origin rules/x", ALL, ""),
    ("gh pr create --title 'Rulebook home' --body 'Closes #3'", ALL, ""),
    ("gh pr view 3", ALL, ""),
    ("gh pr checks 3 --watch", ALL, ""),
    ("gh api repos/jswob/trop/pulls/3", ALL, ""),
    ("git log origin/master..HEAD", ALL, ""),
    ("echo 'git push origin master'", ALL, ""),
    # R-132 as amended (issue #18): the lead merges only through merge-pr and removes a
    # worktree only through worktree-done; every other role is refused both, in any form
    ("engine/tools/merge-pr 9", LEAD, ""),
    ("./tools/merge-pr 9", LEAD, "engine"),
    ("merge-pr 9", LEAD, "engine/tools"),
    ("TALLY_ROLE=lead engine/tools/merge-pr 9", LEAD, ""),
    ("env TALLY_ROLE=lead engine/tools/merge-pr 9", LEAD, ""),
    ("nice -n 10 engine/tools/merge-pr 9", LEAD, ""),
    ("echo 9 | xargs engine/tools/merge-pr", LEAD, ""),
    ("bash engine/tools/merge-pr 9", LEAD, ""),
    ("bash -c 'engine/tools/merge-pr 9'", LEAD, ""),
    ("echo \"$(engine/tools/merge-pr 9)\"", LEAD, ""),
    ("python3 -I -B engine/tools/merge_pr.py 9", LEAD, ""),
    # (inline code naming a rule file's path is refused to the lead too, by R-80's tripwire)
    ("python3 -c \"import subprocess; subprocess.run(['engine/tools/merge-pr', '9'])\"", set(), ""),
    ("cd engine/tools && python3 -c 'import merge_pr; merge_pr.main([\"9\"])'", LEAD, ""),
    ("engine/tools/Merge-PR 9", LEAD, ""),
    ("engine/tools/worktree-done ../trop-i13", LEAD, ""),
    ("python3 engine/tools/worktree_done.py ../trop-i13", LEAD, ""),
    ("sh -c 'engine/tools/worktree-done ../trop-i13'", LEAD, ""),
    # reading, diffing and committing them stays open to every role
    ("cat engine/tools/merge-pr", ALL, ""),
    ("grep -n refused engine/tools/merge_pr.py engine/tools/worktree_done.py", ALL, ""),
    ("git diff master -- engine/tools/merge_pr.py", ALL, ""),
    ("git add engine/tools/merge-pr engine/tools/worktree-done", ALL, ""),
    ("echo 'engine/tools/merge-pr 9'", ALL, ""),
    # and the lead is refused the raw merges like every role
    ("gh pr merge 9 --merge", set(), ""),
    ("gh pr merge 9 --merge --match-head-commit abc123", set(), ""),
    ("TALLY_ROLE=lead gh pr merge 9", set(), ""),
    # tmp/ is scratch for every role (issue #11)
    ("mkdir tmp/t1", ALL, ""),
    ("mkdir -p tmp/t1/deep", ALL, ""),
    ("echo ok > tmp/t1/x.md", ALL, ""),
    ("echo ok >> tmp/x.md", ALL, ""),
    ("cat > tmp/i11/log.md <<'EOF'\nnotes\nEOF", ALL, ""),
    ("touch tmp/i11/x.md && rm tmp/i11/x.md", ALL, ""),
    ("cp engine/spec/DONE.md tmp/t1/spec-copy.md", ALL, ""),
    ("rm -rf tmp/t1", ALL, ""),
    ("cd tmp && mkdir t1 && echo ok > t1/x.md", ALL, ""),
    ("echo ok > x.md", ALL, "tmp/t1"),
    ("cat tmp/active/issue5-brief.md", ALL, ""),
    # but not the sealed exam, its record or the master's briefs
    ("echo x > tmp/active/x.md", MASTER, ""),
    ("mkdir -p tmp/active/sub", MASTER, ""),
    ("cp /tmp/x tmp/Active/issue5-brief.md", MASTER, ""),
    ("sed -i '' 's/a/b/' tmp/active/issue5-brief.md", MASTER, ""),
    ("mkdir -p tmp/holdout/x", MASTER, ""),
    ("echo x > tmp/HOLDOUT/x.md", MASTER, ""),
    ("rm -rf tmp/cases-holdout", MASTER, ""),
    ("touch tmp/cases-holdout/x", MASTER, ""),
    ("python3 -c \"open('tmp/holdout/x','w')\"", MASTER, ""),
    ("rm -rf tmp", MASTER, ""),  # it holds all three
    ("cd tmp && rm -rf active", MASTER, ""),
    ("echo x > x.md", MASTER, "tmp/active"),
    ("echo x > tmp/notes/link-to-active/x.md", MASTER, ""),
    ("echo x > tmp/notes/link-to-holdout/x.md", MASTER, ""),
    ("echo x > tmp/notes/link-to-spec.md", SPEC, ""),
]


# merge-pr's input, recorded from GitHub for PR #17 on 2026-10-31 (`gh pr view 17 --json
# …`, `gh api …/pulls/17/files`, `gh api …/actions/jobs/124674870333`) and trimmed; its
# state and mergeable set as an open PR's would be, and a status added (issue #18)
HEAD_SHA = "e4994fc3977be78f70fd97129fbf70d21c788092"
RECORDED_PR = {
    "number": 17, "state": "OPEN", "baseRefName": "master", "headRefName": "tests/before-s",
    "headRefOid": HEAD_SHA, "isCrossRepository": False, "mergeable": "MERGEABLE", "changedFiles": 3,
    "body": "Closes #5. Session T1, role `tests`; writes `engine/tests/**` only.",
    "statusCheckRollup": [
        {"__typename": "CheckRun", "conclusion": "SUCCESS", "name": "changes", "status": "COMPLETED", "workflowName": "CI",
         "detailsUrl": "https://github.com/jswob/trop/actions/runs/41247837439/job/124674870431"},
        {"__typename": "CheckRun", "conclusion": "SUCCESS", "name": "check full", "status": "COMPLETED", "workflowName": "Engine",
         "detailsUrl": "https://github.com/jswob/trop/actions/runs/41247837446/job/124674870333"},
        {"__typename": "CheckRun", "conclusion": "SKIPPED", "name": "lint-and-typecheck", "status": "COMPLETED", "workflowName": "CI",
         "detailsUrl": "https://github.com/jswob/trop/actions/runs/41247837439/job/124674907210"},
        {"__typename": "CheckRun", "conclusion": "SUCCESS", "name": "counts and instructions (gate 12, Linux)", "status": "COMPLETED",
         "workflowName": "Engine", "detailsUrl": "https://github.com/jswob/trop/actions/runs/41247837446/job/124674870139"},
        {"__typename": "CheckRun", "conclusion": "NEUTRAL", "name": "mutants baseline", "status": "COMPLETED", "workflowName": "Engine",
         "detailsUrl": "https://github.com/jswob/trop/actions/runs/41247837446/job/124674871342"},
        {"__typename": "StatusContext", "context": "ci/extra", "state": "SUCCESS", "targetUrl": "https://example.invalid/x"},
    ],
}
RECORDED_FILES = [
    {"filename": "engine/tests/INTERFACE.md", "previous_filename": None, "status": "added"},
    {"filename": "engine/tests/README.md", "previous_filename": None, "status": "modified"},
    {"filename": "engine/tests/TESTS.sha256", "previous_filename": None, "status": "added"},
]
RECORDED_JOB = {
    "id": 124674870333, "head_sha": HEAD_SHA, "name": "check full", "status": "completed", "conclusion": "success",
    "workflow_name": "Engine",
    "steps": [
        {"conclusion": "success", "name": "Set up job", "status": "completed"},
        {"conclusion": "success", "name": "Install the toolchains and the cargo tools", "status": "completed"},
        {"conclusion": "success", "name": "engine/tools/check full", "status": "completed"},
        {"conclusion": "success", "name": "engine/tools/commit-check", "status": "completed"},
        {"conclusion": "skipped", "name": "Run actions/upload-artifact@cf430e030ddbb5b0abf93d22962f4752f3646cd9", "status": "completed"},
        {"conclusion": "success", "name": "Complete job", "status": "completed"},
    ],
}
# a stand-in for gh: it answers from the recorded files in $FAKE_GH and logs every call
FAKE_GH = '''#!/usr/bin/env python3
import json, os, sys
d = os.environ["FAKE_GH"]
a = sys.argv[1:]
with open(os.path.join(d, "calls.log"), "a") as log:
    log.write(json.dumps(a) + "\\n")
def out(name):
    p = os.path.join(d, name)
    if not os.path.exists(p):
        sys.exit(f"fake gh: no {name}")
    sys.stdout.write(open(p).read())
if a[:2] == ["pr", "view"] and "mergeCommit" in a:
    out("merged.json")
elif a[:2] == ["pr", "view"]:
    out("pr.json")
elif a[:2] == ["pr", "merge"]:
    out("merge.out")
elif a[:1] == ["api"] and any(x.endswith("/files?per_page=100") for x in a):
    files = json.load(open(os.path.join(d, "files.json")))
    sys.stdout.write("".join(json.dumps(f) + "\\n" for f in files))
elif a[:1] == ["api"] and any("/actions/jobs/" in x for x in a):
    out("job.json")
else:
    sys.exit(f"fake gh: unexpected call {a}")
'''


def mutate(change):
    """The recorded (pr, files, job), changed by `change`, a function of the three."""
    pr, files, job = json.loads(json.dumps([RECORDED_PR, RECORDED_FILES, RECORDED_JOB]))
    change(pr, files, job)
    return pr, files, job


def set_check(name, **fields):
    def change(pr, files, job):
        for c in pr["statusCheckRollup"]:
            if c.get("name") == name or c.get("context") == name:
                c.update(fields)
    return change


def touch(*paths, previous=None):
    def change(pr, files, job):
        for p in paths:
            files.append({"filename": p, "previous_filename": previous, "status": "renamed" if previous else "modified"})
        pr["changedFiles"] = len(files)
    return change


def set_step(name, conclusion):
    def change(pr, files, job):
        for s in job["steps"]:
            if s["name"] == name:
                s["conclusion"] = conclusion
    return change


def drop_step(name):
    def change(pr, files, job):
        job["steps"] = [s for s in job["steps"] if s["name"] != name]
    return change


def app_only(pr, files, job):
    files[:] = [{"filename": "apps/web/src/x.ts", "previous_filename": None, "status": "modified"}]
    pr["changedFiles"] = 1
    pr["statusCheckRollup"] = [c for c in pr["statusCheckRollup"] if c.get("workflowName") != "Engine"]


def drop_check_full(pr, files, job):
    pr["statusCheckRollup"] = [c for c in pr["statusCheckRollup"] if c.get("name") != "check full"]


def both(*changes):
    def change(pr, files, job):
        for c in changes:
            c(pr, files, job)
    return change


def field(key, value):
    def change(pr, files, job):
        pr[key] = value
    return change


def job_field(key, value):
    def change(pr, files, job):
        job[key] = value
    return change


# (label, a change to the recorded input, the words the refusal must hold, or None: it merges)
MERGES = [
    ("the recorded PR", lambda *_: None, None),
    ("an app-only PR without check full", app_only, None),
    ("a neutral and a skipped check", set_check("lint-and-typecheck", conclusion="SKIPPED"), None),
    ("Closes in lower case", field("body", "Some work.\n\ncloses #12"), None),
    ("commit-check skipped", set_step("engine/tools/commit-check", "skipped"), None),
    ("commit-check absent", drop_step("engine/tools/commit-check"), None),
    ("a merged PR", field("state", "MERGED"), "not open"),
    ("a closed PR", field("state", "CLOSED"), "not open"),
    ("a base other than master", field("baseRefName", "engine"), "not master"),
    ("master as the head", field("headRefName", "master"), "head is master"),
    ("a fork's branch", field("isCrossRepository", True), "isn't a branch of this repo"),
    ("a milestone branch", field("headRefName", "build/m0"), "milestone"),
    ("a milestone branch in another case", field("headRefName", "Build/S"), "milestone"),
    ("no Closes line", field("body", "Session T1's work."), "Closes #<issue>"),
    ("Closes without a number", field("body", "Closes the issue"), "Closes #<issue>"),
    ("an empty body", field("body", None), "Closes #<issue>"),
    ("conflicting", field("mergeable", "CONFLICTING"), "not MERGEABLE"),
    ("mergeable unknown", field("mergeable", "UNKNOWN"), "not MERGEABLE"),
    ("a failed check", set_check("counts and instructions (gate 12, Linux)", conclusion="FAILURE"), "ended FAILURE"),
    ("a cancelled check", set_check("changes", conclusion="CANCELLED"), "ended CANCELLED"),
    ("a timed-out check", set_check("changes", conclusion="TIMED_OUT"), "ended TIMED_OUT"),
    ("an action-required check", set_check("changes", conclusion="ACTION_REQUIRED"), "ended ACTION_REQUIRED"),
    ("a queued check", set_check("changes", status="QUEUED", conclusion=""), "pending"),
    ("a running check", set_check("check full", status="IN_PROGRESS", conclusion=""), "pending"),
    ("a pending status", set_check("ci/extra", state="PENDING"), "pending"),
    ("an expected status", set_check("ci/extra", state="EXPECTED"), "pending"),
    ("a failed status", set_check("ci/extra", state="FAILURE"), "is FAILURE"),
    ("an errored status", set_check("ci/extra", state="ERROR"), "is ERROR"),
    ("no checks at all", field("statusCheckRollup", []), "no check ran"),
    ("a check of an unknown kind", field("statusCheckRollup", [{"__typename": "Other"}]), "unknown kind"),
    ("an engine PR without check full", drop_check_full, "didn't run"),
    ("check full in another workflow", set_check("check full", workflowName="CI"), "didn't run"),
    ("check full with no job link", set_check("check full", detailsUrl="https://example.invalid/x"), "couldn't be read"),
    ("check full's job on another commit", job_field("head_sha", "0" * 40), "not on the head commit"),
    ("check full's job failed", job_field("conclusion", "failure"), "job ended failure"),
    ("check full's step failed", set_step("engine/tools/check full", "failure"), "check full` didn't pass"),
    ("check full's step absent", drop_step("engine/tools/check full"), "check full` didn't pass"),
    ("commit-check failed", set_step("engine/tools/commit-check", "failure"), "commit-check` ended failure"),
    ("files missing from GitHub's list", field("changedFiles", 4000), "can't be checked"),
    ("an app-only PR changing the CI", both(app_only, touch(".github/workflows/engine.yml")), "walls"),
]
for wall in ("engine/tools/guard.py", "engine/tools/guard", "engine/tools/merge-pr", "engine/tools/merge_pr.py",
             "engine/tools/worktree-done", "engine/tools/worktree_done.py", "engine/tools/check.py", "engine/tools/check",
             "engine/tools/check_frozen.py", "engine/tools/check-frozen", "engine/tools/RULES.sha256",
             ".claude/settings.json", ".github/workflows/engine.yml"):
    MERGES.append((f"a PR changing {wall}", touch(wall), "walls"))
MERGES += [
    ("a wall renamed away", touch("engine/tools/old-guard.py", previous="engine/tools/guard.py"), "walls"),
    ("a wall in another case", touch("Engine/Tools/Guard.py"), "walls"),
]


def merge_pr_checks(root, expect, failures):
    """merge-pr on the recorded input: every role but the lead refused before gh is asked;
    the lead's merges and each refusal, with gh a stand-in that logs its calls."""
    bin_dir = os.path.join(root, "fake-bin")
    os.makedirs(bin_dir)
    with open(os.path.join(bin_dir, "gh"), "w", encoding="utf-8") as f:
        f.write(FAKE_GH)
    os.chmod(os.path.join(bin_dir, "gh"), 0o755)
    data = os.path.join(root, "fake-gh")

    def run(argv, role, pr, files, job, extra=None):
        shutil.rmtree(data, ignore_errors=True)
        os.makedirs(data)
        for name, value in (("pr.json", pr), ("files.json", files), ("job.json", job),
                            ("merged.json", {"mergeCommit": {"oid": "f00d" * 10}})):
            with open(os.path.join(data, name), "w", encoding="utf-8") as f:
                json.dump(value, f)
        with open(os.path.join(data, "merge.out"), "w", encoding="utf-8") as f:
            f.write("")
        env = {k: v for k, v in os.environ.items() if k != "TALLY_ROLE"}
        env.update({"FAKE_GH": data, "PATH": bin_dir + os.pathsep + env.get("PATH", "")}, **(extra or {}))
        if role != "unset":
            env["TALLY_ROLE"] = role
        r = subprocess.run([os.path.join(root, "engine/tools/merge-pr"), *argv], capture_output=True, text=True,
                           env=env, cwd=root, check=False)
        log = os.path.join(data, "calls.log")
        calls = [json.loads(line) for line in open(log, encoding="utf-8")] if os.path.exists(log) else []
        return r, calls

    recorded = mutate(lambda *_: None)
    for role in ROLES:
        if role == "lead":
            continue
        r, calls = run(["17"], role, *recorded)
        expect(f"merge-pr as {role} refuses", r.returncode, 1, r.stderr)
        if calls or "only the lead" not in r.stderr:
            failures.append(f"merge-pr as {role}: asked gh {calls} or said {r.stderr.strip()[:200]!r}")
    for argv in ([], ["17", "18"], ["abc"], ["0"], ["17;x"]):
        r, calls = run(argv, "lead", *recorded)
        expect(f"merge-pr {argv} as lead: usage", r.returncode, 2, r.stderr)
    for label, change, says in MERGES:
        r, calls = run(["17"], "lead", *mutate(change))
        merged = [c for c in calls if c[:2] == ["pr", "merge"]]
        if says is None:
            expect(f"merge-pr, {label}: merges", r.returncode, 0, r.stderr)
            want = [["pr", "merge", "17", "--merge", "--match-head-commit", HEAD_SHA]]
            if merged != want or "f00d" not in r.stdout:
                failures.append(f"merge-pr, {label}: merged {merged}, said {r.stdout.strip()[:200]!r}")
        else:
            expect(f"merge-pr, {label}: refuses", r.returncode, 1, r.stderr)
            if merged or says not in r.stderr:
                failures.append(f"merge-pr, {label}: merged {merged}, or the refusal lacks {says!r}: {r.stderr.strip()[:300]}")
    # gh failing is a refusal: fail closed
    r, calls = run(["17"], "lead", *recorded, extra={"FAKE_GH": os.path.join(root, "nowhere")})
    expect("merge-pr when gh fails: refuses", r.returncode, 1, r.stderr)


def worktree_done_checks(root, expect, failures):
    """worktree-done on scratch worktrees of a throwaway clone with its own origin: the
    lead's removal of a clean, merged one, and each refusal, every other role refused."""
    git_id = ["-c", "user.name=guard-test", "-c", "user.email=guard-test@example.invalid"]

    def git(cwd, *args):
        subprocess.run(["git", "-C", cwd, *git_id, *args], check=True, capture_output=True)

    origin = os.path.join(root, "wt-origin.git")
    main_tree = os.path.join(root, "wt-main")
    subprocess.run(["git", "init", "-q", "--bare", "--initial-branch=master", origin], check=True)
    subprocess.run(["git", "clone", "-q", origin, main_tree], check=True, capture_output=True)
    git(main_tree, "commit", "-q", "--allow-empty", "-m", "base")
    git(main_tree, "push", "-q", "origin", "master")

    def tree(name, merged=True, dirty=None, detach=False):
        path = os.path.join(root, f"wt-{name}")
        git(main_tree, "worktree", "add", "-q", "-b", f"x/{name}", path, "origin/master")
        git(path, "commit", "-q", "--allow-empty", "-m", f"work on {name}")
        if merged:
            git(main_tree, "fetch", "-q", "origin")
            git(main_tree, "merge", "-q", "--no-ff", "-m", f"Merge {name}", f"x/{name}")
            git(main_tree, "push", "-q", "origin", "master")
        if detach:
            git(path, "switch", "-q", "--detach")
        if dirty:
            with open(os.path.join(path, dirty), "w", encoding="utf-8") as f:
                f.write("x\n")
        return path

    def run(path, role, cwd=main_tree):
        env = {k: v for k, v in os.environ.items() if k != "TALLY_ROLE"}
        if role != "unset":
            env["TALLY_ROLE"] = role
        return subprocess.run([os.path.join(root, "engine/tools/worktree-done"), path], capture_output=True,
                              text=True, env=env, cwd=cwd, check=False)

    def branches():
        out = subprocess.run(["git", "-C", main_tree, "branch", "--format=%(refname:short)"], capture_output=True, text=True, check=True)
        return set(out.stdout.split())

    done = tree("done")
    for role in ROLES:
        if role == "lead":
            continue
        r = run(done, role)
        expect(f"worktree-done as {role} refuses", r.returncode, 1, r.stderr)
        if not os.path.isdir(done) or "only the lead" not in r.stderr:
            failures.append(f"worktree-done as {role}: removed it, or said {r.stderr.strip()[:200]!r}")
    unmerged = tree("unmerged", merged=False)
    edited = tree("edited")
    with open(os.path.join(edited, "README"), "w", encoding="utf-8") as f:
        f.write("x\n")
    git(edited, "add", "README")
    refusals = [
        ("an unmerged branch", unmerged, "isn't merged"),
        ("an untracked file", tree("untracked", dirty="notes.txt"), "isn't clean"),
        ("a staged change", edited, "isn't clean"),
        ("a detached HEAD", tree("detached", detach=True), "detached HEAD"),
        ("the main checkout", main_tree, "main checkout"),
        ("a folder that isn't a checkout", os.path.join(root, "engine", "target"), "isn't the top"),
        ("a folder inside a checkout", os.path.join(main_tree, "sub"), "isn't the top"),
        ("a path that doesn't exist", os.path.join(root, "wt-none"), "isn't a folder"),
        ("another repo", root, "another repo"),
    ]
    os.makedirs(os.path.join(main_tree, "sub"), exist_ok=True)
    for label, path, says in refusals:
        before = branches()
        r = run(path, "lead")
        expect(f"worktree-done, {label}: refuses", r.returncode, 1, r.stderr)
        gone = label != "a path that doesn't exist" and not os.path.exists(path)
        if says not in r.stderr or gone or branches() != before:
            failures.append(f"worktree-done, {label}: lacks {says!r} or changed something: {r.stderr.strip()[:300]}")
    r = run(done, "lead", cwd=done)
    expect("worktree-done of the checkout it runs from: refuses", r.returncode, 1, r.stderr)
    for argv in ([], ["a", "b"]):
        r = subprocess.run([os.path.join(root, "engine/tools/worktree-done"), *argv], capture_output=True, text=True,
                           env={**os.environ, "TALLY_ROLE": "lead"}, cwd=main_tree, check=False)
        expect(f"worktree-done {argv}: usage", r.returncode, 2, r.stderr)
    r = run(done, "lead")
    expect("worktree-done of a clean, merged worktree as lead", r.returncode, 0, r.stderr)
    if os.path.exists(done) or "x/done" in branches() or "removed" not in r.stdout:
        failures.append(f"worktree-done as lead: the worktree or its branch is still there: {r.stdout.strip()[:200]} {r.stderr.strip()[:200]}")


def setup(root):
    for d in ("engine/tools", "engine/spec", "engine/tests/runner", "engine/crates/tally/src",
              "engine/crates/tally/benches", "engine/crates/tally-cli/src", "engine/fuzz", "engine/target", ".claude"):
        os.makedirs(os.path.join(root, d), exist_ok=True)
    for name in ("guard", "guard.py", "stop-check", "stop_check.py", "merge-pr", "merge_pr.py", "worktree-done", "worktree_done.py"):
        shutil.copy2(os.path.join(TOOLS, name), os.path.join(root, "engine/tools", name))
    files = {
        "engine/spec/DONE.md": "the contract\n",
        "engine/tests/x": "a test\n",
        "engine/crates/tally/src/lib.rs": "//! stub\n",
        "engine/crates/tally/benches/p8_m1.rs": "//! bench\n",
        "engine/BUILD-LOG.md": "# Build log\n",
        ".claude/settings.json": "{}\n",
        # a cargo workspace like the engine's, for the cargo writers (F-4)
        "engine/Cargo.toml": '[workspace]\nmembers = ["crates/tally", "crates/tally-cli", "tests/runner"]\nexclude = ["fuzz"]\n',
        "engine/crates/tally/Cargo.toml": '[package]\nname = "tally"\n',
        "engine/crates/tally-cli/Cargo.toml": '[package]\nname = "tally-cli"\n',
        "engine/tests/runner/Cargo.toml": '[package]\nname = "tally-runner"\n',
        "engine/fuzz/Cargo.toml": '[package]\nname = "tally-fuzz"\n\n[workspace]\nmembers = ["."]\n',
        # commit messages for R-100 (F-8)
        "engine/target/msg-ai.txt": "Add the parser\n\nCo-Authored-By: Claude <noreply@anthropic.com>\n",
        "engine/target/msg-ok.txt": "Add the parser\n",
    }
    for path, text in files.items():
        with open(os.path.join(root, path), "w", encoding="utf-8") as f:
            f.write(text)
    os.symlink("../../../spec/DONE.md", os.path.join(root, "engine/crates/tally/src/link-to-spec.rs"))
    # the throwaway's own tmp/: a brief, and a notes folder of symlinks out of it (issue
    # #11); its holdout is only named by a dangling link, never made
    os.makedirs(os.path.join(root, "tmp/active"))
    os.makedirs(os.path.join(root, "tmp/notes"))
    with open(os.path.join(root, "tmp/active/issue5-brief.md"), "w", encoding="utf-8") as f:
        f.write("a brief\n")
    for name, target in (("link-to-active", "../active"), ("link-to-holdout", "../holdout"),
                         ("link-to-brief.md", "../active/issue5-brief.md"), ("link-to-spec.md", "../../engine/spec/DONE.md"),
                         ("link-to-tests", "../../engine/tests"), ("link-to-tools", "../../engine/tools")):
        os.symlink(target, os.path.join(root, "tmp/notes", name))
    with open(os.path.join(root, "engine/tools/check"), "w", encoding="utf-8") as f:
        f.write('#!/usr/bin/env bash\n'
                'case "${FAKE_CHECK:-green}" in\n'
                '  green) echo "check fast: green"; exit 0 ;;\n'
                '  red) echo "FAIL clippy-lib: error[E0001] something"; exit 1 ;;\n'
                '  red2) echo "FAIL fmt: Diff in lib.rs"; exit 1 ;;\n'
                'esac\n')
    os.chmod(os.path.join(root, "engine/tools/check"), 0o755)
    git = ["git", "-C", root, "-c", "user.name=guard-test", "-c", "user.email=guard-test@example.invalid"]
    subprocess.run(["git", "init", "-q", "--initial-branch=master", root], check=True)  # R-132's rows need master checked out
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


def hook_command(event, tool="guard"):
    """The command of the engine's hook for `event` that runs engine/tools/<tool>, from the real settings.json."""
    with open(os.path.join(REPO, ".claude", "settings.json"), encoding="utf-8") as f:
        settings = json.load(f)
    for entry in settings.get("hooks", {}).get(event, []):
        for hook in entry.get("hooks", []):
            if f"engine/tools/{tool}" in hook.get("command", ""):
                return hook["command"]
    return None


def run_hook(command, project, payload, role, path=None):
    """The hook string run the way Claude Code runs it: a shell, with the project's dir."""
    env = {k: v for k, v in os.environ.items() if k not in ("TALLY_ROLE", "TALLY_GUARD_SELFTEST_CRASH")}
    env["CLAUDE_PROJECT_DIR"] = project
    if role != "unset":
        env["TALLY_ROLE"] = role
    if path is not None:
        env["PATH"] = path
    r = subprocess.run(["/bin/sh", "-c", command], input=json.dumps(payload), capture_output=True,
                       text=True, env=env, check=False)
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
        # R-132 on another branch: a bare push pushes that branch; master by name still refuses
        git = ["git", "-C", root, "-c", "user.name=guard-test", "-c", "user.email=guard-test@example.invalid"]
        subprocess.run([*git, "switch", "-q", "-c", "feature"], check=True)
        for command, allowed in [("git push", ALL), ("git push origin", ALL), ("git push -u origin HEAD", ALL),
                                 ("git push origin @", ALL), ("git push origin HEAD:master", set()),
                                 ("git push origin feature:refs/heads/master", set())]:
            for role in ROLES:
                code, err = call(root, ["guard"], {"tool_name": "Bash", "tool_input": {"command": command}, "cwd": root}, role)
                expect(f"Bash `{command}` on branch feature as {role}", code, 0 if role in allowed else 2, err)
        subprocess.run([*git, "switch", "-q", "master"], check=True)
        # every checkout of the repo is judged alike (F-1): a linked worktree beside the
        # throwaway and one inside it, the way Claude Code makes them
        git = ["git", "-C", root, "-c", "user.name=guard-test", "-c", "user.email=guard-test@example.invalid"]
        linked = os.path.realpath(tempfile.mkdtemp(prefix="tally-guard-test-linked-"))
        os.rmdir(linked)
        nested = os.path.join(root, ".claude", "worktrees", "nested")
        for tree in (linked, nested):
            subprocess.run([*git, "worktree", "add", "-q", "--detach", tree, "HEAD"], check=True)
        try:
            crossing = [
                (linked, root, "engine/spec/DONE.md", SPEC),
                (linked, root, "engine/tests/x", TESTS),
                (linked, root, "engine/tools/refcore/walk.py", SPEC),
                (linked, root, "engine/clippy.toml", RULES),
                (linked, root, ".claude/settings.json", RULES),
                (linked, root, ".claude/skills/engine-rust/SKILL.md", RULES),
                (linked, root, "engine/crates/tally/src/lib.rs", APP),  # the build role keeps to its own
                (root, linked, "engine/spec/DONE.md", SPEC),
                (root, linked, "engine/tools/guard.py", RULES),
                (root, nested, "engine/spec/DONE.md", SPEC),
                (nested, root, "engine/tests/x", TESTS),
                (linked, root, "tmp/t1/x.md", EVERY),  # scratch in any checkout (issue #11)
                (linked, root, "tmp/active/x.md", MASTER),
                (root, linked, "tmp/holdout/x.md", MASTER),
            ]
            for here, there, path, allowed in crossing:
                full = os.path.join(there, path)
                for role in ROLES:
                    code, err = call(here, ["guard"], {"tool_name": "Edit", "tool_input": {"file_path": full}, "cwd": here}, role)
                    expect(f"Edit of {path} in another checkout as {role}", code, 0 if role in allowed else 2, err)
                    code, err = call(here, ["guard"], {"tool_name": "Bash", "tool_input": {"command": f"cp /tmp/x {full}"}, "cwd": here}, role)
                    expect(f"Bash cp onto {path} in another checkout as {role}", code, 0 if role in allowed else 2, err)
        finally:
            # a raw worktree removal is refused in every role, the lead included: it
            # holds the spec (issue #18's lead removes one only through worktree-done)
            for command in (f"git worktree remove {linked}", f"git worktree remove --force {nested}"):
                for role in ROLES:
                    code, err = call(root, ["guard"], {"tool_name": "Bash", "tool_input": {"command": command}, "cwd": root}, role)
                    expect(f"Bash `{command}` as {role}", code, 2, err)
            for tree in (linked, nested):
                subprocess.run([*git, "worktree", "remove", "--force", tree], check=False)
        # the lead's tools themselves (issue #18)
        merge_pr_checks(root, expect, failures)
        worktree_done_checks(root, expect, failures)
        # it fails closed when the git files can't be read
        with tempfile.TemporaryDirectory(prefix="tally-guard-test-nogit-") as bare:
            os.makedirs(os.path.join(bare, "engine/tools"))
            for name in ("guard", "guard.py"):
                shutil.copy2(os.path.join(TOOLS, name), os.path.join(bare, "engine/tools", name))
            harmless = {"tool_name": "Edit", "tool_input": {"file_path": os.path.join(bare, "x.txt")}, "cwd": bare}
            code, err = call(bare, ["guard"], harmless, "unset")
            expect("no .git: refuses", code, 2, err)
            with open(os.path.join(bare, ".git"), "w", encoding="utf-8") as f:
                f.write("not a gitdir line\n")
            code, err = call(bare, ["guard"], harmless, "unset")
            expect("an unreadable .git file: refuses", code, 2, err)
        # places named by the environment, watched in the build role by name (F-9, F-22)
        with tempfile.TemporaryDirectory(prefix="tally-guard-test-env-") as elsewhere:
            elsewhere = os.path.realpath(elsewhere)
            named = [
                ({"CLAUDE_CONFIG_DIR": os.path.join(elsewhere, "claude-config")}, "claude-config/settings.json", NOT_BUILD),
                ({"CARGO_HOME": os.path.join(elsewhere, "cargo-home")}, "cargo-home/config.toml", NOT_BUILD),
                ({}, "claude-config/settings.json", ALL),
            ]
            for extra, path, allowed in named:
                for role in ROLES:
                    command = f"echo x > {os.path.join(elsewhere, path)}"
                    code, err = call(root, ["guard"], {"tool_name": "Bash", "tool_input": {"command": command}, "cwd": root}, role,
                                     {"CLAUDE_CONFIG_DIR": "", "CARGO_HOME": "", **extra})
                    expect(f"Bash `{command}` with {extra or 'neither set'} as {role}", code, 0 if role in allowed else 2, err)
        # the real hook strings (F-10, F-22): they refuse when bash or python is missing
        # from PATH, and say why when the guard isn't executable or is missing
        pre, change = hook_command("PreToolUse"), hook_command("ConfigChange")
        if pre is None or change is None:
            failures.append("settings.json: no PreToolUse or ConfigChange hook runs engine/tools/guard")
        else:
            lib = {"tool_name": "Edit", "tool_input": {"file_path": os.path.join(root, "engine/crates/tally/src/lib.rs")}, "cwd": root}
            spec = {"tool_name": "Edit", "tool_input": {"file_path": os.path.join(root, "engine/spec/DONE.md")}, "cwd": root}
            event = {"hook_event_name": "ConfigChange", "source": "local_settings"}
            rows = [
                ("PreToolUse hook, a harmless edit", pre, lib, "build", None, 0, ""),
                ("PreToolUse hook, the spec as build", pre, spec, "build", None, 2, "the spec is frozen"),
                ("PreToolUse hook, PATH without bash or python", pre, lib, "build", "/nonexistent", 2, ""),
                ("ConfigChange hook as rules", change, event, "rules", None, 0, ""),
                ("ConfigChange hook as build", change, event, "build", None, 2, "R-81"),
                ("ConfigChange hook, PATH without bash or python", change, event, "rules", "/nonexistent", 2, ""),
            ]
            for label, command, payload, role, path, want, says in rows:
                code, err = run_hook(command, root, payload, role, path)
                expect(label, code, want, err)
                if says not in err:
                    failures.append(f"{label}: stderr lacks {says!r}: {err.strip()[:200]}")
            shim = os.path.join(root, "engine/tools/guard")
            os.chmod(shim, 0o644)
            for label, command, payload in (("PreToolUse", pre, lib), ("ConfigChange", change, event)):
                code, err = run_hook(command, root, payload, "rules")
                expect(f"{label} hook, guard not executable", code, 2, err)
                if "isn't executable" not in err:
                    failures.append(f"{label} hook, guard not executable: says {err.strip()[:200]!r}")
            os.rename(shim, shim + ".away")
            for label, command, payload in (("PreToolUse", pre, lib), ("ConfigChange", change, event)):
                code, err = run_hook(command, root, payload, "rules")
                expect(f"{label} hook, guard missing", code, 2, err)
                if "missing" not in err:
                    failures.append(f"{label} hook, guard missing: says {err.strip()[:200]!r}")
            os.rename(shim + ".away", shim)
            os.chmod(shim, 0o755)
            with tempfile.TemporaryDirectory(prefix="tally-guard-test-app-") as app:
                code, err = run_hook(pre, app, lib, "build")
                expect("PreToolUse hook in a project without engine/", code, 0, err)
        # the Stop hook's string fails closed too (the rest of F-10, G2-5): a stop-check that
        # can't run holds the turn, and says why
        stop = hook_command("Stop", "stop-check")
        if stop is None:
            failures.append("settings.json: no Stop hook runs engine/tools/stop-check")
        else:
            event = {"hook_event_name": "Stop", "stop_hook_active": False}
            for label, role, path, want in (("Stop hook as rules", "rules", None, 0),
                                            ("Stop hook as build, PATH without bash or python", "build", "/nonexistent", 2)):
                code, err = run_hook(stop, root, event, role, path)
                expect(label, code, want, err)
            shim = os.path.join(root, "engine/tools/stop-check")
            os.chmod(shim, 0o644)
            code, err = run_hook(stop, root, event, "rules")
            expect("Stop hook, stop-check not executable", code, 2, err)
            if "isn't executable" not in err:
                failures.append(f"Stop hook, stop-check not executable: says {err.strip()[:200]!r}")
            os.rename(shim, shim + ".away")
            code, err = run_hook(stop, root, event, "rules")
            expect("Stop hook, stop-check missing", code, 2, err)
            if "missing" not in err:
                failures.append(f"Stop hook, stop-check missing: says {err.strip()[:200]!r}")
            os.rename(shim + ".away", shim)
            os.chmod(shim, 0o755)
            with tempfile.TemporaryDirectory(prefix="tally-guard-test-app-") as app:
                code, err = run_hook(stop, app, event, "build")
                expect("Stop hook in a project without engine/", code, 0, err)
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
        for role in ("rules", "tests", "spec", "unset", "lead"):
            code, err = call(root, ["guard", "--config-change"], event, role)
            expect(f"ConfigChange as {role}", code, 0, err)
        if open(log, encoding="utf-8").read() != after:
            failures.append("ConfigChange outside build wrote to BUILD-LOG.md")
        total += 1
        # Stop hook
        stop = {"hook_event_name": "Stop", "stop_hook_active": False}
        for role in ("rules", "tests", "spec", "unset", "lead"):
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
