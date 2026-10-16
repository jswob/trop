# @jswob/design-workbench

A design workbench for a React + Vite app: a second dev server that stages the app's screens and
components on fixtures, with no backend, and the verbs that measure them. It ships the Claude Code
`design` skill that runs a design round on it.

```bash
bun add -d @jswob/design-workbench
```

Requires Bun, React 19, Vite 7 and TanStack Router. It ships TypeScript source. The probes drive
a browser through Playwright.

## Plug in

A design root (say `apps/workbench`) holds the two halves of the binding:

- `workbench.config.ts`, the terminal's half: plain data the verbs and the server read.

  ```ts
  import { defineWorkbench } from '@jswob/design-workbench/config';

  export default defineWorkbench({
  	name: 'My app',
  	port: 5175,
  	app: { root: '.', vite: './vite.config.ts' },
  	setup: './workbench.setup.tsx',
  	styles: './index.css',
  	theme: {
  		css: '../../packages/ui/src/theme.css',
  		ground: '--color-bg',
  		face: '--font-sans',
  		spentIn: ['../../packages/ui/src/**/*.{ts,tsx,css}'],
  	},
  	// settled spaces, the bans file, the census, the gates `check` runs…
  });
  ```

- `workbench.setup.tsx`, the browser's half (`defineSetup`): the kit's catalogue, the places, the
  candidates, the specimens, and what wraps a frame.

The app's own Vite config is read, not merged: its plugins and aliases are taken, its server and
environment are not, so a frame compiles exactly as the app compiles it and never reaches the API.
A component's page on the bench is a `<Name>.specimen.tsx` beside the component.

## Verbs

`workbench serve` starts the bench; `workbench` alone lists every verb. Among them: `check` (the
design root's gates, the census, the skill's copy, blast radius), `probe` and `sweep` (measured
floor and every control pressed at every width), `shots`, `census`, `ban`, and the verbs that
create, promote and delete candidates.

## Claude Code

`@jswob/toolkit` copies the `design` skill into `.claude/skills/design/` (`bunx toolkit sync`). The
repo writes its own `PROJECT.md` beside the copy.

## License

MIT
