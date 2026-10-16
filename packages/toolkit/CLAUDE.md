# Toolkit

Copies the Claude Code files that installed packages ship (skills, commands, agents, rules) into the
repo's `.claude/`, and checks that the copies are still the packages' files. An agent reads the
copies, which are committed; the packages hold the sources. It also fixes an installed package in
place, as Bun's own patch, so the app that found a bug ships the fix the same day, and takes that
patch home to the toolkit's checkout as a branch and a pull request. A change too big for a patch
is tried by running the app on that checkout (`link`, `try`). Nothing here knows the app.

This package ships its own: the `upstream` skill, the `installed-packages` rule (it loads with any
file under `node_modules/`), the `plan-reader` and `smart-commit` skills, the stack skills
`elysia-drizzle-api-best-practices` and `react-frontend-best-practices`, the `dev-docs` commands
and six agents (`code-architecture-reviewer`, `documentation-architect`, `plan-reviewer`,
`postgres-pro`, `ui-designer`, `ux-researcher`), in `claude/`. Change them there, never in
`.claude/`. A stack skill reads the repo's own `PROJECT.md` beside its copy first; `sync` never
writes that file, and the repo keeps it.

## How a package declares its files

In its `package.json`, folders relative to the package:

```json
"claude": { "skills": { "design": "skill" }, "commands": "claude/commands", "agents": "claude/agents" }
```

- A skill's folder is copied whole to `.claude/skills/<name>/`, leaving out dotfiles,
  `__pycache__` and `node_modules`. A skill's `PROJECT.md` is the repo's own: a package that
  ships one is refused, and a sync never writes or removes it.
- Every `.md` file directly in the commands, agents or rules folder goes to `.claude/commands/`,
  `.claude/agents/` or `.claude/rules/`. A rule's `paths` frontmatter scopes when Claude Code
  loads it.
- Only packages the repo depends on are read: the root's and every workspace's `dependencies` and
  `devDependencies`, resolved from the workspace's `node_modules`, then the root's.

## The lock

`.claude/toolkit-lock.json` records, per package, the version synced and the files it wrote (paths
under `.claude/`). It is what lets a sync remove a file a package dropped, and nothing else: a file
no package wrote is the repo's and is never touched. A file that moved to another package stays.
A package no longer depended on has its files removed on the next full sync.

## The app's words

`check` also holds the packages meant for other repos to knowing nothing of this one. The root
`package.json` names them, and the words that would give the app away:

```json
"toolkit": { "agnostic": { "paths": ["packages/kit"], "words": ["ledger"] } }
```

A word is found whatever its case where a word starts (`order` in `Orders`, never in `border`), in
every file git keeps or would keep under the paths. The words live in the repo, never in a
package's own test, where they would be the app's words inside the package.

## Patches

`toolkit patch <package>` runs `bun patch <package>@<installed version>`, which makes an editable
copy at the root's `node_modules/<package>` under either linker, and records it in the marker,
`node_modules/.toolkit/patching.json` (never committed). `--commit`:

1. writes each copy in `.claude/` edited since the last sync into the editable package, at the file
   it was copied from, refusing when the copies were synced at another version;
2. runs `bun patch --commit`;
3. removes the editable copy and runs `bun install`: under the isolated linker `bun patch` leaves
   the copy as a plain folder where the root's link to Bun's store was, and no later install
   replaces it, so after a bump the root would still resolve the old version;
4. syncs the package's copies, and clears the marker.

A package that resolves outside every `node_modules` (a workspace, or a linked folder) is refused:
it is edited where it is. `check` names a package left in the marker, and a `patchedDependencies`
key whose version is no longer installed, which Bun drops in silence, saying where that patch
stands against the version that is (`patchStandings`):

- `fixed`: `git apply -R --check --directory=<installed copy>` passes, so that version has the fix;
- `applies`: it does not, and the patch applies forwards;
- `conflicts`: neither;
- `several`: several versions are installed and none is the patch's; `missing`: none is.

The installed copy is the realpath a workspace resolves, so under the isolated linker it is the
folder in `node_modules/.bun/`; git refuses a path beyond a symlink.

`toolkit carry` acts on it: a `fixed` patch is deleted, file and key, then `bun install`; an
`applies` patch is made again on the installed version (`bun patch`, `git apply` into the editable
copy, then `--commit` without adopting the copies, which are the old version's), and its old file
and key go. The rest are left, and it exits 1. It refuses while a package is in the marker.
Renovate branches run it, then `sync`, in an app's `toolkit-sync.yml`; the lab ran it on Bun 1.4.3
under both linkers.

## Pins

`check` also fails when the root's `packageManager` is not `bun@<x.y.z>` (CI installs that Bun,
and it decides how `bun.lock` is written), and when a dependency of the root or a workspace is a
folder, `link:` or `file:`, which no other clone has. `link` is the way to run on a checkout.

## Upstream

`toolkit upstream [package...]` takes each patch of a package that lives in the toolkit's checkout
home (`--into`, then `TOOLKIT_HOME`, then `toolkit.home` in the root `package.json`, relative to
the repo; it must be a git checkout). A patch of any other package is left alone. Per package:

1. fetches `origin <base>` and adds a worktree in a temporary folder on `upstream/<app>/<package>`,
   reset to `origin/<base>` (`-B`): the checkout's own branch and files are never touched, and the
   branch is made again each run;
2. replays the patch with `git apply --3way --directory=<package dir>`. Bun's patches carry the
   blobs they were made against, so a base that moved on elsewhere still merges; a conflict is
   staged with its markers. Nothing to commit means the base has the fix already;
3. writes `.changeset/upstream-<app>-<package>.md` saying what `--message` says, or the subject
   of the app's commit that last changed the patch, without its type. The bump is `--bump`, or what
   that commit says (`bumpFor`): breaking (`type!:`, a `BREAKING CHANGE:` footer) is a major, a
   minor on `0.x`; `feat` is a minor from 1.0 on; anything else a patch. Deriving it keeps a run
   by hand and the app's workflow from overwriting each other's changeset on the shared branch;
4. runs `bun install --frozen-lockfile` and the package's `test` script with `--run`, unless there
   are conflicts or `--skip-tests`;
5. commits `<type>(<package>)[!]: <message>` (the app's commit's type, `fix` when it has none;
   `!` when the bump breaks) with `--no-verify` (the hooks would lint conflict markers;
   the pull request's CI lints it), its body linking the app's commit (`--source`);
6. with `--push`, fetches the pushed branch and force-pushes only when the tree differs, then opens
   the pull request with `gh`, or edits the open one: a draft when there are conflicts or the tests
   failed, ready otherwise;
7. removes the worktree, whatever happened.

## Link and try

`toolkit link [package...]` runs the app on the checkout (found as for `upstream`): each folder's
`node_modules/<package>` that holds the package, the root's and every workspace's, is replaced by a
link to the package's folder there, recorded in `node_modules/.toolkit/linked.json`, and the
package's copies in `.claude/` are synced from the checkout. With no names, every package of the
checkout that the app installs from the registry. It refuses a checkout with no `node_modules` (a
linked package resolves its own dependencies from its real path), a package being patched, one the
app does not install, one that is the app's own source, and the app as its own checkout. Linking
again restores the links an install replaced. Packages installed elsewhere that depend on the
linked one (Bun's store, a nested `node_modules`) keep their installed copy.

Bun's own `bun link` is not used: from the root it was ignored without a word, and it left the
production build with two Reacts. Whatever must be one copy is deduped by whoever loads the linked
package. The design workbench's Vite config dedupes React, React DOM and the router, and allows the
engine's own folder in `server.fs.allow`, so the bench needs nothing more; but the app's own Vitest
config must dedupe them too (the template's `apps/workbench/vitest.config.ts` does). Without it a
linked engine's tests fail with "Invalid hook call": `try` found exactly that in the lab.

`toolkit unlink [package...]` removes the links, runs `bun install`, and syncs the copies back.
`check` names every linked package, and a link an install put back.

`toolkit try <package...>` runs, in each workspace with a `test` script that depends on the
packages (the root only when none does), `bun run test --run --reporter=json --outputFile=<tmp>`;
then links, runs them again, and unlinks in a `finally`. A failing test is `<file> > <full name>`,
or the file alone when it failed with no failing test. Only what passed before and fails now
counts; a run that gives no report before linking stops it before anything is linked.

## Hooks

`toolkit guard` and `toolkit notice` are Claude Code hooks; the repo wires them in its
`.claude/settings.json` (the template does), through `$CLAUDE_PROJECT_DIR/node_modules/.bin/toolkit`
and only when that exists. `bunx toolkit` is not used there: with no local bin it would fetch an
unrelated package of that name. Both always exit 0, and a call they cannot read passes.

- **guard** (PreToolUse on Edit, Write, MultiEdit, NotebookEdit) resolves the file's deepest
  existing folder, finds its repo by `.git`, and refuses: a file whose path relative to the repo has
  a `node_modules` segment (the package is the name after the last one), unless the marker lists
  that package and the file is inside its editable copy; and a file in `.claude/` that the lock
  lists, unless its package is being patched. A copy of a package that resolves outside every
  `node_modules` (the repo's own, or linked) points at its source file instead. A linked package's
  file resolves into the checkout, so editing through the link passes. The answer is
  `hookSpecificOutput.permissionDecision: deny` with the reason.
- **notice** (SessionStart, `startup`) prints `The toolkit says:` and one line each for the marker,
  the links, `staleProblems` and `check`, and Renovate's open toolkit pull request when its body
  says `**Automerge**: Disabled` (`gh pr list`, 5 s timeout; `waitingAmong` picks it by its branch,
  `renovate/toolkit` or `renovate/major-toolkit` for a major update). Nothing when all is in
  order.

`upstream` writes a breaking commit's `BREAKING CHANGE:` footer (to the next trailer) into the
changeset as `Breaking: …` and into the pull request (`migrationNote`).

## Commands

| Command                                           | What it does                                                                                                                                                                                                           |
| ------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `bun toolkit sync`                                | Copy every installed package's files into `.claude/`, update the lock                                                                                                                                                  |
| `bun toolkit check`                               | Fail when a copy differs, is missing or was never synced, when a patch is stale or half made, when a package is linked, when Bun is not pinned or a dependency is a folder, or when an agnostic path holds an app word |
| `bun toolkit patch <package> [--commit\|--abort]` | Fix an installed package in place (above)                                                                                                                                                                              |
| `bun toolkit guard`, `bun toolkit notice`         | The Claude Code hooks (above)                                                                                                                                                                                          |
| `bun toolkit carry`                               | Take the patches to the versions installed: delete the fixed, make again those that apply (above)                                                                                                                      |
| `bun toolkit upstream [package...] [--push]`      | Take the patches home as branches, and with `--push` pull requests (above)                                                                                                                                             |
| `bun toolkit link [package...]`, `unlink`         | Run the app on the toolkit's checkout, and back (above)                                                                                                                                                                |
| `bun toolkit try <package...>`                    | The dependents' tests before and after linking; fails only on new failures (above)                                                                                                                                     |
| `bun toolkit:check`                               | The same, as the root script; `bun lint` runs it                                                                                                                                                                       |
| `bun toolkit:test`                                | This package's tests                                                                                                                                                                                                   |
| `bun toolkit:typecheck`                           | This package's types                                                                                                                                                                                                   |

The library export (`.`) is `sync(repo, only?)` and `check(repo, only?)`; `only` limits either to
some packages and leaves the rest of the lock as it is. The workbench's `skill` verb is
`sync(repo, [engine])`. The patch functions (`carry` and `patchStandings` among them) and
`MARKER_FILE` are exported too, for a guard that reads the marker, the link functions with
`LINK_FILE`, and `pinProblems`.

## Layout

- `src/assets.ts`: the lock and the plan, as paths and text, with no filesystem. Tested beside it.
- `src/repo.ts`: the repo as Bun laid it out: manifests, workspaces, where a package is installed.
- `src/claude.ts`: the filesystem half: which packages, their files, the copies, `sync` and
  `check`. `claude.test.ts` builds a repo in a temporary folder.
- `src/patches.ts`: `patch`, `carry` and the patch half of `check`. Every Bun command it runs goes
  through a `Run`; `patches.test.ts` hands it a fake Bun that does what Bun 1.4.3 did in the lab,
  and runs real `git apply`.
- `src/pins.ts`: the pin half of `check`. Tested beside it.
- `src/hooks.ts`: `guard` and `notice`. `hooks.test.ts` builds a repo with a `.git` folder in a
  temporary folder; `notice` takes the waiting pull request as an argument.
- `src/upstream.ts`: `upstream`. Every command goes through an `Exec`, which never throws;
  `upstream.test.ts` runs real git on a bare origin, a home and an app in a temporary folder, with
  the global git config off, and fakes Bun and `gh`.
- `src/link.ts`: `link`, `unlink` and the link half of `check`. `link.test.ts` lays out Bun's
  isolated store in a temporary folder, with a fake `bun install` that puts back what was removed.
- `src/try.ts`: `try`. `try.test.ts` fakes Vitest's JSON report, by whether the root's link
  reaches the checkout.
- `src/words.ts`: the app's words in the agnostic paths. `words.test.ts` builds a git repo in a
  temporary folder.
- `src/cli.ts`: the `toolkit` bin. The repo is the nearest folder up from the working directory
  with a `.git`.
- Imports carry no extension, so a package that compiles this one's source with its own
  `tsconfig` (the workbench does) can.
