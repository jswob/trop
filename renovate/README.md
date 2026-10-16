# Renovate presets

How an app takes its updates. `third-party` groups everything that is not ours; four `toolkit-*`
presets, one per way, say how the `@jswob/*` packages arrive. An app's `renovate.json` extends
`config:best-practices`, then `third-party`, then exactly one toolkit preset. The toolkit preset goes
last, because a later `packageRules` entry wins over an earlier one:

```json
{
	"$schema": "https://docs.renovatebot.com/renovate-schema.json",
	"extends": [
		"config:best-practices",
		"github>jswob/bun-monorepo//renovate/third-party",
		"github>jswob/bun-monorepo//renovate/toolkit-auto"
	]
}
```

`third-party` groups every other update as soon as it is found: the minors, patches, pins and digests
together in one PR, each major in a PR of its own. Without it, `config:best-practices` opens a PR per
update, and every one an app merges redeploys it when it touches `bun.lock`. This repo's own
`renovate.json` extends it too.

| Preset           | What the app gets                                                                                                                                                                  |
| ---------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `toolkit-auto`   | One PR for every package that was published. A patch merges itself once the PR's checks pass, and so does a minor from 1.0 on; on `0.x` a minor is breaking and waits for a review |
| `toolkit-manual` | The same PR, always merged by hand                                                                                                                                                 |
| `toolkit-next`   | The newest version of every package, `next` snapshots included, merged by hand: for trying a change in this app before it is released                                              |
| `toolkit-off`    | No updates: the app stays on the versions it has, and an open toolkit PR is closed                                                                                                 |

Which apps get a change is which preset each one extends. Switching is a one-line edit to the app's
`renovate.json`.

## What the presets settle

- **One PR.** The packages are released together and depend on each other (the workbench on the
  toolkit), so every preset groups them (`groupName: toolkit`). A grouped PR merges itself only when
  every update in it would: one `0.x` minor holds the whole PR for a review.
- **No waiting.** `config:best-practices` holds an npm release back for three days
  (`minimumReleaseAge`). These packages are ours, so the presets lift that for them.
- **No queue.** Renovate opens two PRs an hour and keeps ten open (`prHourlyLimit`,
  `prConcurrentLimit`). A new app starts with a backlog of majors that fills both for days, so the
  presets take the toolkit's PR out of the limits and open it first (`prPriority`).
- **The range moves.** `config:best-practices` pins devDependencies, and most of these are
  devDependencies. The presets bump the range instead (`^0.1.0` to `^0.1.1`), so the app's
  `package.json` keeps saying which version it was last tested with.
- **The toolkit's commits do not count.** On a Renovate branch the app's toolkit sync workflow
  carries the patches and syncs the copies in `.claude/`, and commits as the toolkit's GitHub App,
  `jswob-toolkit`. Renovate stops updating a branch someone else committed to, and will not close
  its PR, so every preset, `toolkit-off` too, names that App's commits in `gitIgnoredAuthors`
  (`docs/ci-cd.md` § `toolkit-sync.yml`).
- **A snapshot is the release it leads to.** A `next` snapshot is versioned as the release its
  changesets would make, with `-next-<timestamp>` after it (`0.1.3-next-20261015170000`): newer than
  the release an app has, older than the release to come. `toolkit-next` takes the newest version of
  each package, snapshot or release, so a package with no snapshot stays on its release and the app
  moves on to the release once it is out. `toolkit-auto` and `toolkit-manual` never take a snapshot
  (`allowedVersions`): an app moved back to one of them from a snapshot waits for the release. A
  package that depends on a snapshot is snapshotted with it (the workbench with the toolkit): a
  caret range admits no snapshot, so the workbench's own copy of the toolkit's release would be
  installed beside the snapshot, and kept after it.

## Where the presets are read from

Renovate reads a preset from this repo's default branch at every run, so an edit here reaches every
app at its next run, with no release. This repo is private: the Renovate app reads the preset with
its own access, so it has to be installed on this repo as well as on the app.

A release reaches an app at the app's next Renovate run, not when it is published. The hosted app
runs a repo every four hours once one of its Renovate PRs has been merged, and daily before that.

`bun release:publish` and the snapshot workflow are in `TEMPLATE.md` § Releasing the packages.
