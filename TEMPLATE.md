# TEMPLATE.md

This repo is cut from a working app, not written beside it. The source is meetings-scheduler, and
`template/source.json` records the commit the current files come from.

## What the cut leaves in the source

- **The domain.** Its tables, API modules, migrations, fixtures, web routes and hooks, and the UI
  package's domain kit and pages.
- **The solver** (a Python service) and its skill.
- **The self-hosted production path.** Fly.io, Vercel and Neon are the one deployment here.
- **The skill-activation hook.** Skills here load by their own descriptions.
- **The source's design history.** Its system doc, its rounds, its probes and theses.

In their place the template has one example resource, **notes**, end to end. It has sign-in, the
account page with the light/dark switch, and the frame that holds every page.

## The example resource

`docs/removing-notes.md` lists every file notes takes and every file that names it. An app keeps
that file until it removes notes.

## Cutting again

The source keeps improving. To bring this repo up to a newer commit:

```bash
git switch -c recut-<commit>
bun template/cut.ts ../meetings-scheduler <commit>
bun install
bun lint:fix   # the renames reorder imports
```

Cut from a commit on the source's master. Its pull requests merge by rebase, so a branch's commits
are gone once it merges, and the next cut compares the owned files against the commit recorded here.

The cut copies every file the source has, except those `template/manifest.ts` excludes. It renames
paths and words on the way: `@meetings-scheduler/` becomes `@app/`, and the design tokens' `q-`
becomes `ui-`. The source takes the `@jswob/*` packages from the registry; the cut gives them their
`workspace:*` ranges back, never copies the source's patches of them, and leaves `bun.lock` to
`bun install`. It deletes what the source no longer has and writes `template/source.json`. Then it
prints two lists:

- **The source's words still here.** Each one is a file to neutralize by hand, or a pattern the
  manifest is missing.
- **Owned files the source changed since the last cut.** These files are the template's own and are
  never overwritten: the root files, the Dockerfile, the example resource, the frame's wiring and the
  docs. Carry a change over by hand when it applies.

`packages/ui/` and `apps/workbench/` are **seeds**. They were copied once and are this repo's own,
as each app's are its own. The cut never touches them again. A new file that only the template has
must match an `owned` or `seed` pattern, or the next cut deletes it.

The `@jswob/*` packages are seeds too, and so are the copies `bun toolkit sync` makes of their
skills, commands and agents in `.claude/`. This repo is their home: a fix to one is made here and
published, and every app takes it by version, the source app included.

Review the diff, run the gates, and open a PR titled "re-cut from meetings-scheduler@<sha>".

## Releasing the packages

1. A change to a package carries a changeset: `bun changeset`, pick the packages and the bump
   (on `0.x`, minor is breaking), commit the file with the change.
2. Merged to master, the release workflow opens or updates the "Release packages" PR: versions,
   changelogs, `bun.lock` and the `.claude` stamps (`bun release:version`). A package that depends
   on a bumped one is bumped with it (`updateInternalDependents` in `.changeset/config.json`), so
   its range never leaves out the version an app installs.
3. Merging that PR publishes each package whose version is not on npm (`bun release:publish`),
   then tags it and writes its GitHub release.

`bun release:publish --dry-run` packs and checks everything and publishes nothing. The check fails
when a packed range leaves out the version here, which is what a stale `bun.lock` packs: Bun reads
workspace versions from the lock, and only Bun 1.4.3 or later keeps them current
(`packageManager` pins it). Running the workflow by hand publishes a snapshot of the pending
changesets to `next`, versioned as the release they would make with `-next-<timestamp>` after it.

A new package's first publish is by hand, since npm only lets a package that exists trust a
workflow: `npm login`, then `bun release:publish` in a terminal of your own, where npm asks for the
second factor. Then, on npmjs.com, the package's Settings →
Trusted publishing: GitHub Actions, `jswob`, `bun-monorepo`, `release.yml`, no environment, and
under allowed actions `npm publish` as well as `npm stage publish`. A trusted publisher made since
September 2026 allows only `npm stage publish` unless told otherwise, and the workflow then fails
with `E403 … OIDC permission denied for this action`. A connection cannot be edited: delete it and
add it again. With npm 11.15 or later, the same from a terminal, one package at a time:

```bash
npm trust github @jswob/<package> --file release.yml --repo jswob/bun-monorepo \
  --allow-publish --allow-stage-publish --yes
```

No provenance: npm signs it only for a public repo.

An app takes a release through Renovate, by one of the presets in `renovate/`: every release merged
on its own, every release by review, the `next` snapshots, or none. `renovate/README.md` says how an
app extends one.

A fix found in an app is made there first: `bun toolkit patch` edits the installed package and
writes a Bun patch, which the app ships at once. When the patch reaches the app's default branch,
its `upstream.yml` opens a pull request here with the patch replayed, a changeset and the package's
tests, bumped as the app's commit says. Merged, it goes out with the next release, and Renovate
brings the version to every app. On the source app's Renovate branch, its `toolkit-sync.yml`
deletes the patch the new version has the fix of (`bun toolkit carry`); a patch that is not upstream
yet is made again on the new version when it still applies.
A change too big for a patch is made here and tried in an app with `bun toolkit link` and `try`
before it is released (`packages/toolkit/README.md`).

## Starting an app

`bun setup <name>` in a clone (`README.md` § Starting an app from it) runs `template/setup.ts`. It
names the app, takes the packages from npm at the versions here, and deletes what only the template
has: `template/`, this file, the packages' release and the Renovate presets.

What only the template has inside a file an app keeps sits between two marker lines, which setup
strips with the lines between them:

```md
<!-- template:start -->

The template's own paragraph.

<!-- template:end -->
```

`# template:start` in YAML and the Dockerfile, `// template:start` in TypeScript. A file that names
something the template alone has (a package folder, a release script, a root script setup removes,
this file) must say it inside such a block, or say it so that it is true in an app too.
`bun template:test` runs setup on a copy of this repo and fails on any file left pointing at what it
deleted.
