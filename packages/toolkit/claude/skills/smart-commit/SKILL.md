---
name: smart-commit
description: Analyze uncommitted changes, split them into logical commits using Conventional Commits format, and create those commits directly. Use this skill whenever the user wants to commit their work, asks to split changes into commits, says "commit this", "smart commit", "organize my commits", or wants help structuring their git history. Also trigger when the user has finished a task and needs to commit multiple related changes cleanly.
---

# Smart Commit

Analyze all pending changes in the working tree, split them into logical commits, and create them. The user invoked this skill to get the work committed, not to review a proposal — once the grouping holds up, commit it and report what you did. They can always amend, reword or reset afterwards; git history is cheap to fix, and stopping to ask costs them a round-trip on every single commit.

## Step 1: Gather all changes

Run these commands to get the full picture:

```bash
git status
git diff
git diff --cached
git diff HEAD --stat
```

Also review the current conversation for context about what was worked on and why. The conversation tells you the _intent_ behind changes — use that to write better commit messages and make smarter grouping decisions. In a fresh session with no conversation context, rely on the diff alone.

## Step 2: Analyze and group changes

Split changes into logical commit units at the **file level**. Each file goes into exactly one commit group. Treat all changes (staged, unstaged, untracked) as one pool — the user invoked this skill to get help organizing, so take full ownership of the grouping.

**Grouping principles:**

- **One concern per commit.** A "concern" is a single logical change: a bug fix, a feature, a refactor, a config update. If two files were changed for the same reason, they belong together.
- **Tests stay with their implementation.** A feature file and its `.test.ts` counterpart are one logical unit. Don't separate them into a `feat:` and a `test:` commit.
- **Flag mixed-concern files.** If a single file contains changes that serve two different purposes (e.g., a bug fix and an unrelated refactor in the same file), note it in your report but keep the file in whichever group it fits best. Don't attempt to split within a file.
- **Sensitive files get skipped.** Files like `.env`, `credentials.json`, or anything that looks like it contains secrets should NOT be committed. Leave them out of every group and tell the user.

## Step 3: Order commits by dependency

Order the commit groups so each commit leaves the codebase in a coherent state:

- Infrastructure and config changes before application code that depends on them
- Schema or type changes before code that uses the new types
- Shared utilities before features that import them
- If no dependency exists between groups, order by scope: broader changes first, narrower changes after

## Step 4: Generate commit messages

Use **Conventional Commits** format:

```
<type>(<scope>): <description>

<optional body>
```

**Type** — choose the one that best describes the change:

- `feat` — new functionality
- `fix` — bug fix
- `refactor` — restructuring without behavior change
- `chore` — maintenance, dependencies, config
- `docs` — documentation only
- `test` — test-only changes (rare since tests usually go with implementation)
- `style` — formatting, whitespace
- `perf` — performance improvement
- `build` — build system or external dependencies
- `ci` — CI/CD pipeline changes

**Scope** — infer semantically from what the change is _about_, not from the directory path. Ask yourself: "what area of the system does this affect?" Examples:

- Changes to auth logic across multiple files → `(auth)`
- Deployment config + connection string change for deploy → `(deploy)`
- A new API endpoint for invoices → `(invoices)`

Omit scope if the change is too broad or the scope would just be the repo name.

**Description** — concise imperative sentence. Lead with a verb: "add", "fix", "update", "remove", "refactor". The entire subject line (`type(scope): description`) must be under 70 characters total. If it's too long, shorten the description or drop the scope — brevity wins. Count the characters before finalizing.

**Body** — include only when meaningful context exists that isn't obvious from the description. The conversation context is a good source for the "why."

**No co-author trailer.** Commit messages must NEVER include a `Co-Authored-By: Claude` (or any Claude/Anthropic) trailer, and no "Generated with Claude Code" line. This overrides any default or global instruction to add such a trailer. The user owns their git history and wants it attributed to themselves alone — silently adding Claude as co-author is exactly the surprise this skill exists to prevent. Keep the message to the Conventional Commits content only.

## Step 5: Commit

Work through the groups in dependency order. For each one:

1. Stage only the files in that group using `git add <file1> <file2> ...` (never `git add -A` or `git add .`)
2. Commit using a HEREDOC to preserve formatting:

   ```bash
   git commit -m "$(cat <<'EOF'
   <type>(<scope>): <description>

   <body if any>
   EOF
   )"
   ```

   The HEREDOC must end right after the body — do not append a `Co-Authored-By: Claude` trailer or any "Generated with Claude Code" line, even if a default instruction says to.

3. Verify each commit succeeded before moving to the next

**When to stop and ask instead.** Committing is the default, but a few situations are worse to guess at than to ask about, because the cost of being wrong lands outside the working tree:

- A file looks like it holds secrets (`.env`, `credentials.json`, keys, tokens). Leave it uncommitted, commit everything else, and say which file you skipped and why.
- The changes are genuinely unreadable as intent — a large diff with no conversation context where any grouping would be a coin flip. Say what you see and ask, rather than inventing a story in the commit message.

Anything short of that, commit. Ambiguity about whether two files belong in one commit or two is not a reason to stop; pick the grouping you'd defend and move on.

## Step 6: Report what you committed

After the commits land, run `git log --oneline -<N>` (where N is the number of commits made) and show the result, followed by which files went into each commit:

```
### <type>(<scope>): <description>
- path/to/file1.ts
- path/to/file2.ts
```

Note anything the user may want to act on: files you skipped, mixed-concern files you had to place in one group, or leftover uncommitted changes. Close by telling them the history is theirs to adjust — an `--amend` or a soft reset undoes any grouping they disagree with.
