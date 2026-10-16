# @jswob/toolkit

## 0.1.4

### Patch Changes

- 5ceef0d: The `react-frontend-best-practices` skill names `import-x/order` as the rule that keeps imports in
  order, the template's ESLint 10 setup having moved from eslint-plugin-import to
  eslint-plugin-import-x.

## 0.1.3

### Patch Changes

- 478d7ed: --help prints the help and runs nothing

## 0.1.2

### Patch Changes

- 44f3f49: sync's help names the rules it copies too

## 0.1.1

### Patch Changes

- fd9d320: `toolkit carry` takes each patch made for a version no longer installed to the one that is: deleted when that version has the fix (git can take the patch back out of it), made again when it still applies, and left, with exit 1, when it does neither. `check` says which of these a stale patch is, and also fails when `packageManager` does not pin Bun to one version or a dependency is a folder (`link:`, `file:`).
- 6bc20eb: A package can ship rules (`"rules": "claude/rules"`), which `sync` copies into `.claude/rules/`. The toolkit ships the `upstream` skill and a rule for files under `node_modules/`, and two Claude Code hooks: `toolkit guard` refuses an edit an install or a sync would undo, unless the package is being patched, and `toolkit notice` tells a new session what is half done or out of step. `upstream` writes a breaking commit's `BREAKING CHANGE:` footer into the changeset and the pull request as the migration note.
- 29de4b4: `toolkit link` runs the app on the toolkit's checkout: each `node_modules/<package>`, the root's and every workspace's, becomes a link to the package's folder there, and its copies in `.claude/` are synced from it; `toolkit unlink` puts the installed packages back. `toolkit try` runs the dependents' tests before and after linking and fails only on a test that passed before. `toolkit check` names a linked package.
- 11cb817: `toolkit patch <package>` fixes an installed package in place as Bun's own patch, taking edits made to its copies in `.claude/` into the patch; `toolkit check` names a patch made for a version no longer installed, or left half made.
- 980487a: `toolkit upstream` takes an app's patches home: each patch of a package the toolkit's checkout holds becomes a branch there, `upstream/<app>/<package>`, replayed with a three-way merge, with a changeset and the package's tests run; `--push` opens or updates its pull request, a draft when it conflicts or the tests fail. The app's commit that last changed the patch sets the commit's header and the bump (`fix(kit)!:` is breaking), so a run by hand and the app's workflow agree; `--bump` overrides it.
