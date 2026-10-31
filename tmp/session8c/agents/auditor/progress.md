# Auditor progress (session 8c)

- [x] Read brief, SCAFFOLD §2/3/7/8/9, REPORT §2.4/§5, guard.py, guard_test.py, stop_check.py, check_frozen.py, check.py top, rulebook (before-8b) legend + groups
- NOTE: the harness refused the Write of findings.md ("subagents should return findings as text"); the full findings go in the SubagentHandback message instead. Not worked around.
- [x] A. guard probes (probes/probe.py, probes/results.tsv): 113 probes x 6 roles = 678 calls; 174 unexpected allows (3 noise excluded), 12 STRICT; fails-closed F01-F18 done (F12 fails open: hook string exit 127 without bash on PATH). Demo: perl hole + hard link -> check-frozen exit 1.
- [x] B. plants: 14 planted forms; green in check fast/deny: P1 P1c P2 P2b P3 P3b P4 P5 P5b P6 P6b P7 (12); fired: P1b (clippy::panic), P7b (ast-grep-scan, full only)
- [x] C1 (refcore I/O read; widening demo: fs::read_dir/fs::read inside read_input stay under 3/3), C2 (proposal), C3 (demo: guard 0, check fast exit 1, stop-check exit 2; settings.local.json disableAllHooks stays green), C4 (DONE l.25/l.27 vs REASON regex)
- [x] D1 (E39-41, B68-70: rules+unset), D2 (CLEARED quoted; RUSTFMT=/usr/bin/true turns fmt green)
- [x] WT reverted (status clean), guard restored
- [ ] final handback (full findings in the message)

Leads to test (from reading guard.py):
- keyword-led segments (`then`, `do`, `{`, `!`) hide the writer; backticks; `>(tee …)`
- prefixes with valued options (`nice -n 5`, `env -u X`, `xargs -n 1`, `xargs -I {}`, `timeout -s KILL 5`, `sudo -u x`)
- interpreter: code_like inline code makes the file operand "the script" (perl -pi -e 's/a/ b/' file)
- sed --expression= / -es…; ex/vim; tar -C; unzip -d; curl -o; rustfmt; hard links (ln src dest)
- cd $D / cd "$(…)" then relative write; main checkout outside repo; parent .cargo/config.toml [env]/[alias] not in check-rules' list
