# Changesets

A changeset says which `@jswob/*` packages a change releases and how far: `bun changeset`, then
commit the file it writes here with the change. While a package is `0.x`, a minor is a breaking
change and a patch is everything else.

Merged to master, the changesets become the "Release packages" PR (`.github/workflows/release.yml`);
merging that publishes. A change to an app or to the template alone needs none.
