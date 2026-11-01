# The engine's command line

T's interface (session T1, issue #5, 2026-10-31): what the suite sends the engine and what it reads back. It places session 8c's draft (`tmp/session8c/INTERFACE-draft.md`), checked against refcore's `engine/tools/refcore/cli.py` (`main`, `run_paths`, `run_texts`, `joint`, `reprint_lines`), `loader.py` (`collect`) and `joint.py` (`use_member_files`) at master `eb16d53`, and against `run.py` (`argv_of`, `execute`, `compare`). It adds no meaning to the language: what a construct means is `LANGUAGE.md`'s and `ENGINE.md`'s.

**The rule: the engine's output equals refcore's, byte for byte, for every check T runs** (DONE gates 1–3 and 10). Where this page and refcore differ, refcore wins, and the difference is a gap (§8). A household check (`I:<H>:<ask>`) is compared in run5.py's form, the ask's blocks with warning lines dropped (`run.py`, `compare`); every other check compares the whole output.

## 1. Commands

| Command | Reads | Prints (stdout) | Exit |
|---|---|---|---|
| `tally run <path>… [--pin=bytes] [--ask <id>…]` | the reality: every path given, then the members its `use` lines name (§2) | the asked values in L §5.16's form, or one `refused: …` line | 1 when the first line starts with `refused: `, else 0 |
| `tally reprint <path>…` | as `run` | for each file in path order, `--- <path>` and its canonical text (L §2.4); then `sha256 <hex>`, the reality's hash (E §3.1 step 1) | as `run` |
| no command, or any other word (`test` included: refcore's own tests aren't the engine's) | nothing | a usage text of the engine's own | 2 |

- **Arguments** (refcore's loop, `cli.py` `main`): every word before `--ask` that isn't `--pin=…` is a path; every word after `--ask` is an ask id; `--pin=bytes` may stand anywhere. With no `--ask`, every ask of the reality is answered. `run.py` sends `run <files> <options> [--ask <ids>]` (`argv_of`), `--ask` only when the check names asks.
- **`--pin=bytes`:** a `use` line's pin is the sha256 of the member's file bytes instead of its canonical text (E §3.1 step 1). refcore stores any word after `--pin=` and acts only on `bytes`; no check sends another word (§8, gap I-4).
- **What the suite sends** (`checks.tsv`, 634 rows): `run` with files in 631 rows, `--ask` in 389 of them (the rest answer every ask); `--pin` and `reprint` in none; a directory in none. `R:all` (refcore's own tests, `options` `test`) and the two key scripts (`options` `script`) run no engine, and the 85 `not-run:` rows are load rows: `run.py --interp engine` lists all 88 as not run, with their reason.
- **Usage text:** no check reads it, so its words are the engine's. It goes to stdout, and the exit is 2.

## 2. Reading the reality

- **A file path** is read as given, whatever its extension, relative to the working directory (`run.py` runs each check in its `cwd`, with its files relative to it).
- **A directory** stands for every `*.tally` file under it, at any depth. Which order they are taken in is §5's question.
- **Bytes:** a file that isn't UTF-8 refuses the run: `refused: LOAD-PARSE cannot parse: file is not UTF-8 (<path>)`, the path as given.
- **A path that names no file or folder** refuses the run: `refused: LOAD-PATH <path> doesn't exist`, the path as given (tally 2.1.1, L §8.3; T's fixture `codes/`, milestone M0). Every file is read before any is split or parsed, the files in full-path order, so of several such refusals the first by full path prints (L §8.3 "Which refusal prints").
- **A joint model** (L §6.1): each member a `use` line names is found relative to the joint model's file (`os.path.normpath(os.path.join(dirname(model file), use path))`), a file or a directory; a member's files given beside the joint model on the command line are the member's. Each is checked against its pin (`LOAD-PIN`). A member's refusal prints `refused: member <alias>: <the refusal>`.
- **Standard input is never read**, and no environment variable is (R-39). The reality is the files; the request is the arguments.

## 3. Output, exit codes and errors

- **Everything goes to stdout, and stderr stays empty.** `run.py` compares stdout and stderr concatenated, so any byte on stderr is a difference. `--stats` (the `counters` build, BUILDER §13.2, debug builds only) is the one stderr writer, and never in a checked run.
- **Line endings:** `\n`. `run.py` reads the bytes as UTF-8 with universal newlines.
- **A refusal** is one line, `refused: <CODE> <message>`, the code from the spec's catalogue (L §8, E §11; R-76). refcore's own code `REFCORE-NOT-YET` (an input it can't handle) is never the engine's: a check whose stored output holds it is a gap in refcore, not a target.
- **Exit codes:** 0 answered (holes and per-value refusals included: they are printed answers), 1 refused (the first line starts `refused: `), 2 usage. These replace the stub's placeholders 64, 69 and 74 (SCAFFOLD §8 item 11): 69, "not implemented", is what `run.py --interp engine` shows red today (§7). A failed write of the output keeps its own code, 74; no check sees it.
- **An exit that disagrees with the output is an error,** whatever the output: `run.py --interp engine` marks `error` any exit but 1 after a `refused: ` first line and 0 otherwise (the stub's 69, a crash's 101 or 134).
- **A crash never happens** (R-2, R-3): refcore turns any exception into a refusal line; the engine has no crash path.

## 4. In process: `tally_cli::run`

T's runner (`engine/tests/runner/`, the crate `tally-runner`) calls the engine in process, so property tests and mutation testing (gate 8) need no binary and no files. T's tests may not touch the file system (clippy bans `std::fs` in tests as in the library, and an exception would change `EXCEPTIONS.lock`, which isn't T's). So the reality reaches the CLI's library as bytes, which is what R-39 asks of the library anyway.

```rust
pub fn run(args: &[OsString], files: &[u8]) -> Output   // Output { stdout, stderr, code }
```

- **`args`** are the words after the program name, exactly as §1 (`run a.tally b.tally --ask q`).
- **`files`** is the **file table**: every file the run may read, as pairs of netstrings (D. J. Bernstein's form, `<decimal length>:<bytes>,`), the path then the content, pair after pair. A path is UTF-8, `/`-separated, written as the run names it: a path given in `args` as given; a file under a directory given in `args` as `<dir>/<relative path>`; a joint member's file as §2 resolves it. An empty table is `b""`.
- **The binary** (`main.rs`, the CLI's one sanctioned I/O file, R-39 and R-108) reads the arguments, reads every path they name, walks each directory given, reads each member a `use` line names (it may ask a pure library function for a file's member paths), packs the table, calls `run` and writes `Output`. It never reads standard input.
- **The library** resolves a path in `args` against the table: a key is that file; a path that is no key but a directory prefix of keys (`<path>/…`) is every key under it ending `.tally`; anything else is a path that doesn't exist, refused `LOAD-PATH` (§2).
- **Today's stub** has this signature, `run(args: &[OsString], stdin: &[u8])`, with the second argument named for standard input and unused. The table takes its place without a signature change, so T's runner compiles against the stub now and against the engine later. The name `stdin` and the stub's doc comment are the builder's to change at S.

## 5. The order of a reality's files

refcore takes the files in the order of its command line, and a directory in `os.walk`'s order: each directory's own files sorted by name, then its subdirectories, each sorted by name (`loader.collect`). R-40 (amended 2026-10-30) and E §3.1 step 1 take the **full relative paths, sorted by code point**. The two differ beyond 8c's example (`a.x.tally` before `a/b.tally` by full path): a walk puts `C/c.tally` before `C/asks/q.tally`, a full-path sort after it.

**Measured** (T1, 2026-10-31; `tmp/t1/order.py`, through `run.py`'s own `run_one` with its budget, cache keying and watchdog; whole outputs compared, nothing dropped):

- **The suite:** 276 checks name more than one file, and 133 of the runnable ones name them in an order other than R-40's. Each was run on refcore with its files given in full-path order and compared with its stored order (2 jobs at a time, niced; `tmp/t1/order-runs.log`):
  - 9 print the same bytes;
  - **124 print the same lines but their `WARN-UNREAD` warnings in another order**: refcore lists them in the order it read the files. 100 of them are household checks, which `run.py` compares with warning lines dropped; **24 compare the whole output** (`I:A:A-13~A1..A4`, `I:A:A-14~A1`, `I:A:A-15~A1`, `I:B:B-13~B1..B5`, `I:C:C-01..C-12`, `I:E2:E2-01~e2c`), so an engine that takes R-40's order and warns in it fails them;
  - none differs in a value, a hole, a refusal or a register line.
- **A synthetic pair** (`tmp/t1/syn/`, two files that each fail to load): refcore prints only the refusal of the file it read first, so `run a.tally b.tally` and `run b.tally a.tally` print different lines.
- **The spec:** L §9 says the order of files changes nothing, and E §3.1 orders files only for the hash. Neither says in what order warnings print (L §8.5), nor which refusal prints when several files have one (L §8.3).

**So the order reaches the output, and the spec doesn't say what it should be.** A spec question (DONE rule 3), **decided by the user at G-T, 2026-10-31: "Sort warnings"**: warnings print sorted by code, then by their text, whatever the files' order; when several load refusals exist, the first in full-path order (then by line) prints. The spec patch and refcore's fix are tally 2.1.1 (issue #13: L §8.3, §8.5). Issue #16 re-ran on refcore 2.1.1 every whole-output check whose stored warnings were out of that order, the 24 above and 5 more (`U:scen/SCEN-01..03`, `I:D:D-13~D1`, `D2`), and rewrote their baselines: in each, only the order of the `WARN-*` lines changed. Two of `perf-digests.tsv`'s rows changed the same way and were rewritten.

## 6. What the tests role keeps of this page

- `run.py --interp engine` (§7) follows §1–§3 exactly; `INTERFACE.md` changes only with a T commit, and the engine changes to match it, never the reverse (DONE rule 2).
- The file table (§4) is the runner's only way into the engine; the runner never spawns the binary.

## 7. `run.py --interp engine`

- **The binary:** `engine/target/release/tally` (the build role builds it), run with `argv_of(c)`'s words, from the check's `cwd`, under the same 90 s budget and the same 8 GB watchdog (`tools/memcap.py` now also matches `target/release/tally run`).
- **The cache:** keyed by the binary's sha256 instead of refcore's and the runtime's, then the same parts as refcore's runs (each input file's path and bytes in run order, the asks, the options, the argv and the cwd).
- **Never a baseline:** an engine output is never stored as one, and never `ran`. A check with no stored baseline (9 today) is compared with refcore's own run of it from the cache, and is an `error` until refcore has run it.
- **Not run, with the reason printed:** `R:all`, the two key scripts and the 85 load rows.
- **Spelling:** `--interp=engine`. At the repository root the guard reads a bare `engine` word given to `python3` as the folder `engine/` and refuses the call, so `run.py` takes `--opt=value` for every option.
- **The stub, red** (2026-10-31, the release stub at master `eb16d53`):

  | layer | checks | same | differ | error | not run |
  |---|---|---|---|---|---|
  | interpreter | 1 | 0 | 0 | 0 | 1 (refcore's own tests) |
  | unit | 154 | 0 | 0 | 154 | 0 |
  | integration | 479 | 0 | 0 | 392 | 87 (85 load rows, 2 key scripts) |

  Every run check is `error`: `tally run: not implemented`, exit 69. None is skipped silently and none is `ran`.

## 8. Gaps

Each is something refcore does that the engine can't copy, or a place the spec is silent. None is reached by a check today.

| # | Where | refcore | The engine | Who decides |
|---|---|---|---|---|
| I-1 | a path that doesn't exist | crashed until tally 2.1.1; now refuses `LOAD-PATH` (§2) | as refcore; T's fixture `codes/` | settled by tally 2.1.1 (issue #13) |
| I-2 | `reprint` of a file that isn't UTF-8 | crashed until tally 2.1.1; now refuses as `run` does | as `run`: `LOAD-PARSE … file is not UTF-8` | settled by tally 2.1.1 (issue #13) |
| I-3 | `reprint`'s path names | relative to a directory given, else the file's base name, so two files given from different folders with one name collide | follows refcore | T, when a check needs `reprint` |
| I-4 | `--pin=` with a word other than `bytes` | taken as the canonical pin, silently | follows refcore, or refuses as a usage error | the spec role, if a check ever sends one |
| I-5 | the order of the files: which refusal prints when several files each have one, and the order of `WARN-UNREAD` lines | since tally 2.1.1, warnings sorted by code then text, and the first refusal by full path | as refcore | decided at G-T; settled by tally 2.1.1 (issue #13) |

8c's draft said two checks name a directory or a non-`.tally` path; at `eb16d53` none does (the two key scripts name a `.py` and `.md` files, and run no engine).
