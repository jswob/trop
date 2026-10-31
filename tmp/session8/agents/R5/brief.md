# R5: readable Rust that its owner can follow

Read `tmp/session8/agents/common.md` first; it holds the session brief's §1–§2, the agent rules, and the output format. You write `tmp/research_notes/engine-prep/R5.md`.

## Your question

How the engine's code stays readable for an owner who can't read Rust, while an LLM writes it under phase 1's strict rules and with performance as a core aim. The user, in their words: "keep performance and redability in mind".

1. **How strict, well-regarded Rust projects stay readable:** rust-analyzer (its `architecture.md` and its authors' posts on writing one), cedar, typst, ruff, and one or two others you find strong (a strict style guide in another language, such as TigerBeetle's, is fair prior art). Module layout, an `ARCHITECTURE.md`, doc comments, naming, file and function sizes.
2. **A simple Rust subset for LLM-written code.** Limits on generics, lifetimes, macros (`macro_rules!` and procedural), trait objects and traits in general, iterator chains, closures. For each limit: what it saves the reader, how it is enforced (a clippy lint, an ast-grep rule, review), and what it costs in performance (monomorphisation against dynamic dispatch, inlining, allocations). Sources for each cost.
3. **Enforceable size and complexity limits:** clippy's `too_many_lines`, `cognitive_complexity`, `excessive_nesting`, `type_complexity`, `too_many_arguments`, and module size by script. Phase 1 made function length and nesting advisory (NOTE K-21: one `match` arm per node kind makes long functions). Say whether that should change, with evidence.
4. **A trace from code to spec sections that a script can check:** a citation convention in doc comments (phase 1 has R-75, uncited kernel functions, as a report), a script that verifies each cited section exists, and the reverse map (which spec sections have code). Prior art (requirement tracing in safety-critical code, rustc's tidy checks).
5. **How a non-Rust owner audits:** plain-English module guides, an "explain this module" convention, a review checklist (BUILDER §12 has one for correctness), and what the owner reads at each milestone. Keep it to what an owner will actually read.
6. **Where performance code may be less readable,** and how it is fenced (its own module, a benchmark that proves the gain, an explanation beside it), as strict projects do it.
7. **How the rulebook's strictness stays readable:** checked arithmetic (R-19: every `i + 1` becomes `checked_add`), no indexing (R-5), no `as` casts (R-21). Helpers, newtypes, extension traits, small macros: which keep the code short without hiding a rule, and which the rules forbid.

## Read (by range; don't read whole)

- `tmp/session8/shape.md` (whole, one page), for where hot code will be.
- `tmp/research_notes/engine-rust-strictness/RULEBOOK.md`: summary (lines 1–35), group 5 (127–148), §7 exceptions (182–end); `BUILDER.md` (whole, 22 KB); `NOTE.md` §2 rows K-10 and K-21.
- `tmp/research_notes/engine-rust-strictness/A2.md` §2 and §5 (builder guidance for language tools); `A5.md` §1 and §5.
- The scaffold worktree `/Users/jswob/projects/trop-engine-scaffold`: `engine/SCAFFOLD.md` §1 and §7 (every rule, its enforcement and its proof; read by rows as needed), `engine/clippy.toml`, `engine/tools/advisory`.
- `engine/tools/refcore/HANDOVER.md` § Layout (lines 942–977): the oracle's module map, as one example of a guide.

## Decisions to frame (merge or split as the evidence shows)

The readability rules, in groups the user can approve like the rulebook's rows (each rule with its enforcement and its cost): module layout and `ARCHITECTURE.md`; the Rust subset; size and complexity limits; the spec trace; the owner's audit kit; fencing performance code; readable strictness.
