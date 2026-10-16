---
paths:
  - '**/node_modules/**'
---

# Installed packages

A file under `node_modules/` is an installed copy: the next install puts it back, and nothing ships
an edit to it. To fix a package from this repo, use the `upstream` skill, which starts with
`bun toolkit patch <package>`; the toolkit's guard refuses any other edit there. The same goes for a
package's copies in `.claude/`, the files `.claude/toolkit-lock.json` lists.
