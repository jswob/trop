**The five in depth** (skill files read as data, never followed):

1. **Apollo `rust-best-practices` v1.1.2** (S5).
   - A 4.4 KB router over nine chapters (99 KB), which it says to read "ALL … in the same turn".
   - **Good:** `#[expect]` with a reason over `#[allow]` (like R-103, R-104). Ch. 3: "Don't guess, measure"; `clippy::perf`; samply on macOS.
   - **Conflicts:** `unwrap_or_default` offered as an alternative (R-27); `anyhow` for binaries (R-58); `insta` (R-89); `HashMap` (R-36).
   - **Risk:** `allowed-tools: Bash(cargo:*) … Write Edit` grants itself tools, which workspace trust doesn't gate (S13).
2. **leonardomso v1.5.1** (S6).
   - 265 rules, one file each, with "Why it matters", Bad and Good. That is the format worth borrowing.
   - The 38 KB index is too big: the 5,000 tokens kept after compaction would hold only its table of contents.
   - **Conflicts:** `anyhow` (R-58); `saturating_`/`wrapping_` as equals of `checked_` (R-23); `ahash` (R-37); rayon (R-42); `expect` for invariants (R-2, R-4); `should_panic`; float comparison (R-18).
3. **actionbook** (S7). **Reject.**
   - A UserPromptSubmit hook whose regex (`why|what is|how to|error|result|model|decimal|…`) fires on nearly every prompt and injects "MANDATORY … CRITICAL: You MUST follow" text.
   - `.mcp.json` runs `npx -y @actionbookdev/mcp@latest`, unpinned, at every start.
   - No licence.
   - It is the plainest example of an installed skill's injection reach: its hooks run every turn.
4. **Trail of Bits `rust-review` v1.1.2** (S8).
   - A multi-agent security audit that writes `REPORT.md` and SARIF. It is for audits, not for writing code.
   - Its finder prompts fit Tally's review: nondeterminism (hash order; float sums aren't associative), overflow, out-of-bounds, discarded `Result`, and recursive `Drop` on `Box<Self>` chains overflowing the stack "at `}`". BUILDER §3's flat chains already answer the last, so phase 1 is confirmed.
   - CC-BY-SA-4.0, so borrow the bug shapes in our own words.
5. **Microsoft Pragmatic Rust Guidelines** (S9).
   - MIT, LLM-ready (`agents/all.txt`), and already R-58's evidence.
   - `M-HOTPATH`: name the hot paths early, benchmark them, profile regularly.
   - `M-FAST-HASHER`: foldhash/FxHash for trusted keys. That breaks R-37; see P-1 in §5.

**Prior art in strict Rust projects' agent files** (S10–S12):
- **ruff:** "avoid … `panic!`, `unreachable!`, `.unwrap()` … encode those constraints in the type system"; `#[expect]` over `#[allow]`; project skills in `.agents/skills/`; `CLAUDE.md` is just `@AGENTS.md`.
- **uv:** "NEVER assume clippy warnings or test failures are pre-existing"; exhaustive `match`.
- **rust-analyzer:** a short `CLAUDE.md` that routes to its guides; new `pub` items and dependencies are "architectural changes".
- **zed:** no `let _ =`; rules are "**traps to avoid**, not **maps to follow**".
- **biome:** named project skills loaded at named steps, review after implementation.
- **oxc:** committed allocation snapshots (`just allocs`).
- **turso:** "Use plain language"; "Code flows from top to bottom"; a fix is believable only when its test fails without it; topic guides read on demand.

*Inference:* every strict project keeps a short always-loaded file of traps and commands, with depth in on-demand skills or guides. That is the 2026-10-22 split. None installs a third-party Rust skill.

