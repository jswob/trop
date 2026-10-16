# @jswob/toolkit

Copies the Claude Code files that installed packages ship (skills, commands, agents, rules) into a repo's
`.claude/`, records them in `.claude/toolkit-lock.json`, and checks that the committed copies are
still the packages' files. Fixes an installed package in place, as Bun's own patch, and takes the
fix home to the packages' own repo as a pull request.

```bash
bun add -d @jswob/toolkit
bunx toolkit sync
```

Requires Bun. It ships TypeScript source.

## How a package ships its files

In its `package.json`, folders relative to the package:

```json
"claude": { "skills": { "design": "skill" }, "commands": "claude/commands", "agents": "claude/agents" }
```

- A skill's folder is copied whole to `.claude/skills/<name>/`, except a `PROJECT.md`: that is the
  repo's own, and a sync never writes or removes it.
- Every `.md` file directly in the commands, agents or rules folder (`"rules": "claude/rules"`)
  goes to `.claude/commands/`, `.claude/agents/` or `.claude/rules/`.
- Only packages the repo depends on are read.

A file no package wrote is the repo's, and a sync never touches it. A file a package dropped is
removed on the next sync.

## Commands

| Command                            | What it does                                                                                                                                                |
| ---------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `toolkit sync`                     | Copy every installed package's files into `.claude/` and update the lock                                                                                    |
| `toolkit check`                    | Fail when a copy differs, is missing or was never synced, a patch is stale or half made, a package is linked, Bun is not pinned or a dependency is a folder |
| `toolkit patch <package>`          | Make the installed copy editable (`bun patch`), at `node_modules/<package>`                                                                                 |
| `toolkit patch <package> --commit` | Write the patch: the edits there and to the package's copies in `.claude/`; then sync its copies                                                            |
| `toolkit patch <package> --abort`  | Throw the edits away                                                                                                                                        |
| `toolkit carry`                    | Take the patches to the versions installed: delete those the version has the fix of, make again those that apply                                            |
| `toolkit upstream [package...]`    | Take the app's patches home: a branch and a changeset each in the packages' checkout; `--push` opens the PRs                                                |
| `toolkit link [package...]`        | Run the app on the packages' checkout instead of what it installs                                                                                           |
| `toolkit unlink [package...]`      | Put the installed packages back                                                                                                                             |
| `toolkit try <package...>`         | The app's tests on what it installs, then on the checkout; fails only on a test that passed before                                                          |
| `toolkit guard`, `toolkit notice`  | Claude Code hooks (below)                                                                                                                                   |

`check` can also hold shared packages to knowing nothing of the app. Name them, and the app's
words, in the root `package.json`:

```json
"toolkit": { "agnostic": { "paths": ["packages/kit"], "words": ["ledger"] } }
```

## Fixing a package in the app

```bash
bunx toolkit patch @scope/kit           # node_modules/@scope/kit is now editable
# edit it there, or edit the copies of its skills in .claude/
bunx toolkit patch @scope/kit --commit  # patches/@scope%2Fkit@<version>.patch
```

Commit the patch with `package.json` and `bun.lock`: the app ships the fix at once. An edit to a
copy in `.claude/` goes into the patch with the package's own files, so the copies stay the
package's. A patch is made for one version. When the package is updated, Bun drops the patch
without a word and keeps its key; `check` names it, and says which of these it is:

- the new version has the fix (git can take the patch back out of it): `carry` deletes the patch;
- it does not, and the patch still applies to it: `carry` makes the patch again on it;
- it does not apply: make it again by hand.

`check` also fails when `packageManager` does not pin Bun to one version, and when a dependency is
a folder (`link:`, `file:`), which no other clone has.

## Taking a fix home

The packages' home is a git checkout of the repo they are published from. Name it once, relative
to the app, in the root `package.json` (or pass `--into`, or set `TOOLKIT_HOME`):

```json
"toolkit": { "home": "../bun-monorepo" }
```

```bash
bunx toolkit upstream           # every patch of a package the checkout holds
bunx toolkit upstream --push    # and push each branch, and open or update its pull request (gh)
```

Each patch becomes a branch there, `upstream/<app>/<package>`, made again from `origin`'s base each
run, with the patch replayed by `git apply --3way` and a changeset. The commit says what the app's
commit that last changed the patch said, and links it. That commit also sets the bump, so a run by
hand and the app's workflow agree: `fix(kit)!:` or a `BREAKING CHANGE:` footer is a major (a minor
on `0.x`, where a minor is breaking), `feat` a minor from 1.0 on, anything else a patch; `--bump`
overrides it. The package's tests run on the branch. A conflict is committed with its
markers, and its pull request opens as a draft that names the files, as it does when the tests
fail. Running it again pushes nothing new and updates the same pull request. The checkout's own
branch and files are never touched: the work happens in a worktree of its own.

When the base has the fix already, it says so: delete the patch once the app takes that version.

## Trying a bigger change

A change too big for a patch is made in the checkout, and tried in the app before it is released:

```bash
bunx toolkit link @scope/kit    # the app runs on ../bun-monorepo/packages/kit
bunx toolkit unlink             # and on what it installs again
bunx toolkit try @scope/kit     # the tests of every workspace that uses it, before and after
```

`link` turns each folder's `node_modules/@scope/kit`, the root's and every workspace's, into a link
to the package's folder in the checkout, and syncs its copies in `.claude/` from there. Bun's own
`bun link` is not used: from the root it is ignored without a word. A linked package takes its own
dependencies from the checkout, so the checkout must be installed, and every config that loads it
and must see one React dedupes it: `resolve.dedupe` in the app's Vite and Vitest configs. An
install puts the installed copies back; `link` again restores the links. `check` fails while
anything is linked, so nothing is committed on the checkout's files. `try` runs each workspace's
Vitest with the JSON reporter, links, runs it again, and unlinks whatever happened: a test the app
already failed is not the change's.

## Hooks

Two commands for Claude Code's hooks, in the repo's `.claude/settings.json`. Neither ever fails, and
both do nothing before the install:

```json
"hooks": {
	"PreToolUse": [{ "matcher": "Edit|Write|MultiEdit|NotebookEdit", "hooks": [{ "type": "command",
		"command": "t=\"$CLAUDE_PROJECT_DIR/node_modules/.bin/toolkit\"; [ -x \"$t\" ] || exit 0; exec \"$t\" guard" }] }],
	"SessionStart": [{ "matcher": "startup", "hooks": [{ "type": "command",
		"command": "t=\"$CLAUDE_PROJECT_DIR/node_modules/.bin/toolkit\"; [ -x \"$t\" ] || exit 0; exec \"$t\" notice" }] }]
}
```

- `guard` refuses an edit that an install or a sync would undo: a file in an installed copy of a
  package, or a package's copy in `.claude/`, unless that package is being patched. It says what to
  do instead: patch it, or edit the source when the package is the repo's own or linked. It does
  not see the shell.
- `notice` tells a new session what is half done or out of step: a package being patched, a link,
  a stale patch or copy, and Renovate's toolkit pull request when it waits for a review (asked
  through `gh`, which it skips offline).

A breaking fix's `BREAKING CHANGE:` footer is what `upstream` writes into the changeset, so it is
in the release notes that Renovate's pull request shows.

## What it ships itself

The `upstream` skill (the patch and link lanes, and how a breaking fix is marked), a rule that
loads with any file under `node_modules/`, the `plan-reader` and `smart-commit` skills, the stack
skills `elysia-drizzle-api-best-practices` and `react-frontend-best-practices` (each reads the
repo's own `PROJECT.md` first), the `dev-docs` commands and six agents.

## License

MIT
