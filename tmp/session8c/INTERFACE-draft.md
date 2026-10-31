# The engine's command line: a draft for T (session 8c)

T places this as `engine/tests/INTERFACE.md`, wires `run.py --interp engine`, writes `TESTS.sha256` and shows the stub red (plan §2). It is read from refcore's `engine/tools/refcore/cli.py` (`main`, `run_paths`, `run_one`, `joint`), `loader.py` (`collect`) and `engine/tests/run.py` (`argv_of`, `execute`) at commit `05103df`. **The rule: the engine's output equals refcore's, byte for byte, for every check T runs.** Where this draft and refcore differ, refcore wins, and the difference is a gap for T's log.

## 1. Commands

| Command | Reads | Prints (stdout) | Exit |
|---|---|---|---|
| `tally run <path>… [--pin=bytes] [--ask <id>…]` | the reality: every path given, then the members its `use` lines name | the asked values, one line each, or one `refused: …` line | 0, or 1 when the first line starts with `refused: ` |
| `tally reprint <path>…` | as `run` | each file's canonical text (L §2.4) and the reality's sha256 (E §3.1) | 0, or 1 on `refused: ` |
| no command, or another word | nothing | refcore prints its usage (its docstring) | 2 |

- **Argument order** (refcore's loop): every word before `--ask` that isn't `--pin=…` is a path; every word after `--ask` is an ask id. `--pin=bytes` may stand anywhere. `run.py` always sends `run <files> <options> --ask <ids>`, and `--ask` only when the check has asks.
- **What the suite uses today** (`engine/tests/checks.tsv`, 634 rows): `run` with files and asks in 631; `--pin` and `reprint` in none. Two checks name a directory or a non-`.tally` path. `R:all` (refcore's own tests) and the two `script` rows don't run the engine.
- **Usage text:** refcore prints its Python docstring. The engine's usage text needn't match unless a check asks for it (none does): T decides.

## 2. Reading the reality

- **A file path** is read as given, whatever its extension.
- **A directory** is walked recursively for `*.tally` files. refcore's order is `os.walk` with each directory's subdirectories and files sorted by name (Python's string order, which is code-point order). **Note for T:** that is a per-directory order. R-40 (amended 2026-10-30) and E §3.1 l.118 sort the **full relative paths** by code point, which differs when names share a prefix (`a.x.tally` before `a/b.tally` by full path; `a/b.tally` first by a per-directory walk). Whether the order reaches the output (the reality hash sorts on its own; a refusal's order may not) is a question for T's log, not settled here.
- **Bytes:** a file that isn't UTF-8 refuses with `refused: LOAD-PARSE cannot parse: file is not UTF-8 (<path>)`.
- **A joint model** (L §6.1): each member named by a `use` line is found **relative to the joint model's file**, read and checked against its pin (`LOAD-PIN`): the sha256 of its canonical text, or of its bytes with `--pin=bytes`. A member's refusal prints `refused: member <alias>: <the refusal>`.
- **Standard input** is never read. (The stub reads it today; the scaffold's audit item C1 removes that read, so the CLI's three exceptions are the arguments, the reality's files and the output.)

## 3. Output and errors

- Everything goes to **stdout**. `run.py` compares stdout and stderr concatenated (`text(out) + text(err)`), so the engine must write **nothing** to stderr in a checked run. `--stats` (the `counters` build, BUILDER §13.2) is the only stderr writer, and never in a checked run.
- Line endings: `run.py` reads the bytes as UTF-8 with universal newlines; the engine writes `\n`.
- **A refusal** is one line: `refused: <CODE> <message>`, the code from the spec's catalogue (R-76). refcore adds its own `REFCORE-NOT-YET` for inputs it can't handle; the engine never prints that code. A check whose stored output is `REFCORE-NOT-YET` is a gap in refcore, not a target.
- **A crash never happens** (R-2, R-3): refcore turns any exception into a refusal line; the engine has no crash path at all.

## 4. What T wires (plan §2)

- `run.py --interp engine`: the release binary at `engine/target/release/tally`, called with `argv_of(c)`'s words, from the check's `cwd`, under the same 8 GB watchdog.
- `TESTS.sha256` over `engine/tests/` (check-frozen verifies it; run output in `cache/`, `out/`, `results/` is skipped).
- The stub shown red: every check fails against "not implemented" (exit 69, the scaffold's placeholder), proving the wiring.
- The exit codes 0/1/2 replace the stub's placeholders 64, 69 and 74 when the builder writes the CLI (SCAFFOLD §8 item 11).

## 5. The guard's side (the lead's, proven at 8c)

The build role is refused every write to `engine/tests/**` (the tests role's only) and to `engine/tools/refcore*` and `engine/tools/SHA256SUMS` (the oracle, the spec role's only). The probe is in `tmp/session8c/progress.md` (step 3).
