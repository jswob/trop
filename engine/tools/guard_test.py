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
ROLES = ("build", "rules", "tests", "spec", "unset")
HOME = os.path.expanduser("~")

SPEC = {"spec"}
TESTS = {"tests"}
RULES = {"rules"}
APP = {"rules", "unset"}
BUILD = {"build", "rules", "unset"}
EVERY = set(ROLES)
MASTER = {"unset"}  # app work, which the master session is

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
    ("echo x > tmp/active/x.md", {"unset"}, ""),
    ("mkdir -p tmp/active/sub", {"unset"}, ""),
    ("cp /tmp/x tmp/Active/issue5-brief.md", {"unset"}, ""),
    ("sed -i '' 's/a/b/' tmp/active/issue5-brief.md", {"unset"}, ""),
    ("mkdir -p tmp/holdout/x", {"unset"}, ""),
    ("echo x > tmp/HOLDOUT/x.md", {"unset"}, ""),
    ("rm -rf tmp/cases-holdout", {"unset"}, ""),
    ("touch tmp/cases-holdout/x", {"unset"}, ""),
    ("python3 -c \"open('tmp/holdout/x','w')\"", {"unset"}, ""),
    ("rm -rf tmp", {"unset"}, ""),  # it holds all three
    ("cd tmp && rm -rf active", {"unset"}, ""),
    ("echo x > x.md", {"unset"}, "tmp/active"),
    ("echo x > tmp/notes/link-to-active/x.md", {"unset"}, ""),
    ("echo x > tmp/notes/link-to-holdout/x.md", {"unset"}, ""),
    ("echo x > tmp/notes/link-to-spec.md", SPEC, ""),
]


def setup(root):
    for d in ("engine/tools", "engine/spec", "engine/tests/runner", "engine/crates/tally/src",
              "engine/crates/tally/benches", "engine/crates/tally-cli/src", "engine/fuzz", "engine/target", ".claude"):
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
            for tree in (linked, nested):
                subprocess.run([*git, "worktree", "remove", "--force", tree], check=False)
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
