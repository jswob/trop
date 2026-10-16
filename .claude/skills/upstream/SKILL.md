---
name: upstream
description: Fix an installed package from inside the app that found the bug, ship the fix with the app at once, and send it home to the package's repo. Use when a dependency's code, or one of a package's skills, commands, agents or rules copied into .claude/, is wrong; when the toolkit's guard refuses an edit under node_modules/ or to a copy in .claude/; when `bun toolkit check` names a stale patch; or when a change to a @jswob package is too big for a patch and has to be tried on its checkout.
---

# Upstream

A package installed from the registry is not edited where it lies: the next install puts it back,
and nothing ships the edit. The toolkit gives three lanes instead. Pick one before touching
anything:

| The change                                                        | Lane                            |
| ----------------------------------------------------------------- | ------------------------------- |
| A fix this app needs today, a few lines, in any installed package | **Patch** (below)               |
| A redesign, a new feature, or many files of a `@jswob` package    | **Link** the toolkit's checkout |
| The package is right and the app uses it wrong                    | Neither: fix the app            |

## Patch

1. `bun toolkit patch <package>`. Bun makes an editable copy at `node_modules/<package>`, and the
   guard lets edits through there and to the package's copies in `.claude/`.
2. Edit it there. A skill, command, agent or rule the package ships may be edited in its copy in
   `.claude/` instead; the patch takes the copy's edits into the package.
3. Prove it in the app: run the tests of the workspaces that use it.
4. `bun toolkit patch <package> --commit`. This writes `patches/<package>@<version>.patch`, updates
   `package.json` and `bun.lock`, installs the package again with the patch, and syncs its copies.
   To throw the edits away instead: `--abort`.
5. Commit the patch with `package.json`, `bun.lock` and `.claude/` in one Conventional Commit,
   scoped to the package without its scope: `fix(workbench): keep the sheet open on resize`. That
   commit is what the package's repo sees, so it says what was wrong, not what the app was doing.
   - **Breaking for other apps?** That is when an app must change its own code to take the fix: a
     renamed export, a changed default, a different config shape. If so, mark it: `fix(workbench)!:`,
     and a footer that tells another app what to change, in its own words:
     `BREAKING CHANGE: pass the sheet's width as a number of pixels, not a class name.` The footer
     becomes the migration note in the release, and the version bump follows from it. On `0.x` a
     breaking change is a minor, which no app takes without a review.
6. Merge it. The app ships the fix at once. When the patch reaches the default branch, the app's
   `upstream.yml` opens a pull request in the toolkit's repo with the patch replayed, a changeset and
   the package's tests. To do it from here instead: `bun toolkit upstream --push` (it needs the
   checkout, named by `toolkit.home` in the root `package.json`, or `--into`).
7. Once that is released, Renovate's branch in this app takes the version, and the app's
   `toolkit-sync.yml` deletes the patch (`bun toolkit carry`).

Only a package the toolkit's checkout holds travels home. A patch of anything else stays in the app.
Report that one to its own project.

### A patch made for another version

`bun toolkit check` says where it stands against the version installed:

- **has the fix**: `bun toolkit carry` deletes the patch;
- **still applies**: `bun toolkit carry` makes the patch again on the new version;
- **does not apply**: read the old patch file, `bun toolkit patch <package>` on the new version,
  make the same fix by hand, `--commit`, and delete the old patch file and its key in
  `patchedDependencies`.

## Link

Make the change in the toolkit's checkout, on a branch, with a changeset (`bun changeset` there).
Then try it in this app before it is released:

```bash
bun toolkit link <package>    # this app runs on the checkout's source
bun toolkit try <package>     # the tests of every workspace that uses it, before and after
bun toolkit unlink            # back to what the app installs
```

While anything is linked, `bun toolkit check` fails, so nothing is committed on the checkout's
files. An edit under `node_modules/<package>` reaches the checkout through the link, which the guard
allows. The change reaches apps through the checkout's own pull request and release. To try a
release candidate in an app before that, the app can extend the `toolkit-next` Renovate preset.

## Never

- Edit under `node_modules/` without `bun toolkit patch`, or work around the guard with the shell.
- Run `bun patch` directly: `toolkit patch` also takes the `.claude/` copies and handles the
  isolated linker.
- Hand-edit a patch file or `patchedDependencies`, except to delete a patch you made again.
- Commit while a package is linked, or depend on a folder (`link:`, `file:`): `bun toolkit check`
  fails on both.
