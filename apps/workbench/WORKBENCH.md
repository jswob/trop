# Workbench — this project's instance

What the `design` skill's `WORKBENCH.md` describes as a contract, built out here. Read this for the
addresses and the file map; read the skill's for what a workbench is _for_.

`bun designs` from the monorepo root. Serves `http://localhost:5175`, and **refuses to boot if that
port is taken** rather than drifting to 5176 — a second server on the next port serves a stale build
to every command below while appearing to work.

No API, no database, no `.env`, no auth. Nothing here can fetch: the UI package imports neither the
web nor the API, so a frame draws only what its fixtures hand it. Boot cost is effectively zero —
start it without asking.

**The workbench is an engine and a binding.** The engine is `@jswob/design-workbench`: the bench, the
routes, the frame, the verbs and the probes, with nothing of this app in it. What is here is the
app's half: the binding (`workbench.config.ts` for the terminal, `workbench.setup.tsx` for the
browser), the places, the fixtures and the bans. What has settled stands in the UI package,
`packages/ui/src/` (`@app/ui`), which the web draws too. In this file a path that starts `bench/`,
`routes/`, `components/`, `cli/` or `lib/` is the engine's, under its `src/`; `kit/`
and `views/` name a module wherever it stands, the UI package's when it has settled and this
folder's when only the workbench draws it; `kit-docs/`, `fixtures/`, `proto/`, `theses/` and
`probe/` are here. A design round never writes the engine.

---

## Commands

`bun workbench <verb>` from the monorepo root. `bun workbench` alone prints the full set.
`bun designs:x`, `designs:probe` and `designs:census` are the same verbs under other names.

| Command                                                      | What it is for                                                                      |
| ------------------------------------------------------------ | ----------------------------------------------------------------------------------- |
| `workbench frames --space view`                              | Every state with the address that opens it. Never build one by hand                 |
| `workbench drive <id>:<state> --list`                        | What every control is actually called at that width                                 |
| `workbench drive <id>:<state> --do "click:…; wait:400"`      | Press the live frame; prints the change with the before-number                      |
| `workbench drive <id>:<state> --list --do "click:…"`         | Both, in that order — the list is a preface, not an alternative                     |
| `workbench drive <id>:<state> --width mobile --shot out.png` | A capture at a real viewport with motion off. `--theme light` for light             |
| `workbench sweep <id>[:<state>]`                             | Press every control at every width; prints only what went wrong                     |
| `workbench conform <id>`                                     | Hold the frames against `theses/<id>.thesis.ts`; prints what was not kept           |
| `workbench new <id> --title "…"` · `rm <id>`                 | Scaffold or delete a candidate, registry included                                   |
| `workbench new <id> --from <drawing.tsx>`                    | Start the candidate from a board's approved drawing, as the file it is              |
| `workbench board <folder>`                                   | Build the board kit into a board's folder, outside the repo, in ~2s                 |
| `workbench promote <id> --to kit --dry`                      | The caller count before anything moves                                              |
| `workbench ban find --for ground`                            | The sections of `BANS.md` that apply, verbatim                                      |
| `workbench ban add --section 4 --round <n> --title "…"`      | Append a ban in the right place. Body on stdin                                      |
| `workbench check`                                            | eslint · types · tests · census · the skill's copy · blast radius, design-scoped    |
| `workbench probe --space view --quiet`                       | Only what changed since the accepted baseline                                       |
| `workbench probe --space view --theme light --quiet`         | The same in the light mode. The baseline is dark's; only dark accepts               |
| `workbench census`                                           | Every state the census type names and the files that paint it                       |
| `workbench shots <folder> [--against <folder>]`              | A picture of every frame, to prove a change moved nothing. See below                |
| `workbench skill`                                            | Copy the engine's skill into `.claude/skills/design`: `toolkit sync` for the engine |

**`drive --list` before `drive --do`, every time.** A frame can carry controls with near-identical
names in two places — a note's `Delete` in each row, the rail's `Notes` and the page's own heading —
and a name guessed from the source is how a press lands on the wrong one. `drive` refuses an
ambiguous name and prints the candidates rather than taking the first match.

**`drive` reports layout as well as content.** `columns`, `moved` and `extent` appear whenever they
change, because a state reached by clipping a column away changes no text: the ledger collapse reads
`text —` and `columns 1 → 2`. Movement is measured in content coordinates, so a click scrolling its
own target into view is never mistaken for a reflow.

**`sweep` is `drive` asked of the whole frame.** It presses every drawn, enabled control that is not
already standing on its answer, at every width, and prints each fault once with the states and
widths it was found in: a press that moved nothing, a keyboard left on the page body, something
Escape does not close or that does not hand the keyboard back, sideways scroll after a press, a
press that throws, text wider than its box, a control with no name. A run of more than eight alike
controls is pressed at its first, middle and last (`--every` presses them all). About 2.5s a state
and width, since each press is made on a fresh frame. A frame is a pinned state, so some controls
are inert on purpose; read such a finding, then hold it with `--accept --only '<regex>'`, which
writes the matching findings to `probe/sweep-baseline.json` and leaves the rest failing. Each
finding is held on its own, so one new fault does not reprint the held ones.

A specimen draws its controls with no handler, so the kit space (`--space kit`) holds findings of
its own: a press on a specimen moves nothing by design. A specimen has no shell to land the
keyboard, so its ground does (`kit-docs/specimens/frames`).

**Do not save a source file while a sweep runs.** The dev server reloads the frames under it; the
sweep opens a frame once more before it gives a press up, and a whole-space run can still lose
presses to it.

Three things keep a verdict the press's own. Every press is made on a frame just opened, because a
press that moves nothing on the glass can still end a hold and make the next control read as inert.
The frame is read once the control is in view, because a pointer drawn by the scroll was charged to
the press. And a press that seems inert, or leaves the keyboard on `<body>`, is read again a moment
later, for a smooth scroll and for a hand-off that waits on an answer.

**`conform` holds the frames against a thesis file.** `theses/<id>.thesis.ts` is created by `new`
and filled in when the thesis is approved, before the build. Each check names a part and what it
measures (`width`, `height`, `lines`, `chars`, `count`, any computed style, or the `gap` between two
parts), with one number or one a width. A part is `data-part="<name>"` in the frame, or a selector
the file's `parts` names.

**The probe baseline.** `probe/baseline.json` holds the findings that are readings rather than
faults — a target covered by the slug on purpose, say — in `PROBES.md`'s own words. They still
print; they no longer fail the command. `--full` ignores the file, `--accept` rewrites it, and a held
finding that stops occurring is reported so the file cannot quietly stop being true. Each key names
the covering **object** and carries the **count** of targets it covers, so it is stable when the
markup inside it changes and fails when it starts covering more.

The probe runs four frames at a time, always writes the full report to a temp file it names, and
ends in a `RESULT:` line — a pipe returns the last command's status, so the exit code is the first
thing `| tail` throws away.

**`shots` is the check for a change that must move nothing.** `bun workbench shots <folder>` takes
one PNG of every frame at every width in both appearances, motion off, into a folder outside the
repo. Run it before the change, then `shots <new folder> --against <folder>` after it, and it names
every file whose bytes differ. The fonts and every other answer from another host are kept in the
first folder and replayed, so a second set is not at the mercy of the network. A picture that
differs is taken twice more before it is called different.

---

## Routes

| URL                | What                                                                                                                                                   |
| ------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `/`                | Opens on the ground                                                                                                                                    |
| `/tokens`          | Every token the theme declares, read from `packages/ui/src/theme.css`. Never written                                                                   |
| `/kit`             | The kit's catalogue, as a page that is read rather than staged, with each component's caller count read off the source                                 |
| `/kit/$component`  | One kit component's page, drawn from its **specimen**                                                                                                  |
| `/kit/stress`      | **The kit's stress page** — every primitive on one sheet at once                                                                                       |
| `/views`           | Every place the app has drawn, in the rail's order, as a page that is read rather than staged                                                          |
| `/views/$view`     | One place, every state it has                                                                                                                          |
| `/proto/$name`     | One candidate, every state it has, with its notes                                                                                                      |
| `/frame/$kind/$id` | **One variant alone, owning the whole viewport.** `$kind` is `view`, `kit` or `proto` — a label only; `lib/registry` resolves the id across the spaces |

Every page but `/frame` is drawn in the bench, the workbench's chrome (_The bench_ below). An
address with no page says so inside the chrome (`routes/_bench.$`), with the sidebar beside it. The
routes are the engine's: a place, a component or a candidate is a file here and never a route.

`/frame` is the screenshot address and the iframe source. Everything else is chrome around it.

**Every frame in a grid carries an `Open` link to it**, in the caption beside the variant's name —
the same address the iframe is loading, but in a tab where the browser window is the viewport: no
`scale()`, no stage, the design at the display's real size. Reach for it whenever a frame has to be
_used_ rather than looked at, and whenever the review depends on which side of a breakpoint the
screen falls on.

### Search params

On a staged page (`/views/$view`, `/kit/stress`, a component's page). What the bench adds to these
is under _The bench_:

| Param      | Values                                                                                                                                                                                                                                     | Default   |
| ---------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | --------- |
| `variants` | comma-separated ids — `notes,empty`                                                                                                                                                                                                        | all       |
| `frames`   | cross-space list, `id:variant` comma-separated — `notes:notes,account:account`. Resolved through `lib/registry`, so entries from two spaces sit in one list. Drawn as **Side by side**, each frame named by its subject and then its state | absent    |
| `width`    | `mobile` · `tablet` · `desktop`                                                                                                                                                                                                            | `desktop` |
| `chrome`   | `bare` — the one frame there is                                                                                                                                                                                                            | `bare`    |
| `scale`    | `0.1`–`2` — share of the row one frame gets; `1` is one across                                                                                                                                                                             | fit all   |
| `t`        | `0`–`10000` ms to pin every animation at. Beats `rate`. Each animation is clamped to its own end, so one `t` is a coherent moment across a page whose transitions are different lengths                                                    | absent    |
| `rate`     | `0`–`1` — clock speed as a multiple of real time. `0.2` is five times slower. **`0` is off** — every animation at its end state                                                                                                            | `1`       |
| `theme`    | `dark` · `light` — the appearance every frame on the page is drawn in. Written to the page's own URL, and carried to the next place                                                                                                        | `dark`    |
| `notes`    | `on` — a candidate's notes, open under its frames. `/proto/$name` alone has them                                                                                                                                                           | absent    |

`/frame/$kind/$id` takes `variant` (a single id), `chrome`, `t`, `rate` and `theme`. The frame
writes `theme` onto its root while it renders, as the app's boot does, so a variant reads the
appearance it is drawn in (`useAppearance`) on its first render.

Everything the stage shows is in the URL, so any arrangement is deep-linkable. Build frame URLs with
`frameUrl()` from `lib/frame-url.ts`, never by hand.

### Viewports

`lib/viewports.ts` — mobile `390×844`, tablet `834×1112`, desktop `1440×900`.

Frames are **real iframes at true pixel width**, scaled only to fit the stage. Each is a genuine
viewport: `h-screen` measures the frame, a modal portalled to `document.body` lands inside its own
frame, and **media queries actually apply**.

---

## The bench — the workbench's chrome

The bench is a pathless layout (`routes/_bench.tsx`), so its name is in no address. It is the
engine's `bench/` (`src/bench/` in `@jswob/design-workbench`), and it takes nothing from the host app:
`bench.css` is plain CSS with every variable and class prefixed `wb`, the face is a file in
`bench/fonts/` (Sofia Sans Semi Condensed, OFL), and the stylesheet is imported by `bench/Shell`
alone, so none of it loads in a frame.

- **Two modes.** `show=widths` draws one state at the three widths, at one scale. `at` names the
  state the stage is standing on, which `]` shows alone and Widths draws. `Across` is `scale`
  (absent, `0.5`, `1`) and `Motion` is `rate` (absent, `0.5`, `0.2`, `0`).
- **Every option of the bar survives moving to another place.** `bench/search`'s `carry` keeps
  `show`, `width`, `theme`, `scale`, `rate`, `parts`, `try` and `notes` on every link in the sidebar
  and the jump list, and drops what names one place's states (`variants`, `at`, `frames`) and a pin.
- **Keys are the ones printed on the bar**: `[` `]` step, `A` all, `W` mode, `1 2 3` width, `T`
  frames, `F` across, `M` motion, `P` parts on a component and `N` notes on a candidate; `\` folds
  the sidebar and `⌘K` opens the jump list. A key whose switch the page does not print does
  nothing (`bench/keys`). A frame takes the focus as soon as it is pressed, so each frame's window
  repeats its keys on the page's (`bench/frame-keys`) unless the frame is being typed into.
- **A value the address cannot read is dropped alone** (`parseBenchSearch`): `?width=wide` costs the
  width and nothing else.
- **The bar in less room than it needs** drops its keys, then its labels, on a container query over
  `.wb-main`, so a folded sidebar gives them back. Under 1024px it is the sheet.
- **Widths stacks under 700px of stage**, widest first, each frame as wide as the stage allows.
- **The workbench's own light and dark is not in the address.** It follows the device until chosen
  and is kept in `localStorage` (`workbench-appearance`), as is the folded sidebar
  (`workbench-sidebar`).
- **A place's words are its own**: a `description` of at most 100 characters on the place and a
  `note` of at most 60 on a state (`lib/subject`'s `DESCRIPTION_CAP` and `NOTE_CAP`).
  `workbench.setup.test` fails a longer one in the settled spaces. A candidate is not held to them:
  its bet is read in full on its notes.
- **`/views` is the places list** (`bench/PlaceList`): every place in the rail's order, its
  description and how many states it lists, each row the way to its place. A destination with no
  place under it is named in one line beneath, so the list reads as a map of what is drawn.
- **A candidate's page is a place's, plus notes.** `/proto/<id>` stages its states the way a place's
  are. `N` (`notes=on`) opens a strip under the stage with its bet in full and the type it would
  hand over, `PROPOSED_SHAPE`, both read off its own file (`bench/ProtoNotes`,
  `lib/proposed-shape`). A settled place has no such panel.
- **A `frames` list is drawn as Side by side** (`bench/cells`' `listCells`): frames of more than
  one subject on one stage, which `variants` cannot name. A cell is named by its subject, since
  that is what differs across the row, with the state after it; a state left off draws the first,
  and a subject nobody registered is named back rather than drawn as a hole.

### Tokens and components on the bench

**A component's page is a declared specimen**, `<Name>.specimen.tsx` beside the component
(`packages/ui/src/kit/Button.specimen.tsx`), and the bench draws every one of them the same way. It
is the file's default export, typed against the component:

```ts
export default specimen(Button, {
	kit: 'button', // the kit entry it is the page of
	is: 'Every action a hand can take, at three rungs of emphasis.', // at most 100 characters
	base: { children: 'Save', weight: 'normal' }, // what every state is drawn with
	states: {
		running: { label: 'Running', note: '…', args: { weight: 'primary', running: 'Saving' } },
	},
	props: {
		weight: { is: '…', options: { primary: true, normal: true, quiet: true } },
		running: { is: '…', text: true },
	},
	parts: { plate: { select: 'button', is: 'The ground the word sits on.' } },
	holds: ['…'],
	refuses: ['…'],
});
```

- **The compiler is the check.** `props` must hold a line for **every** prop the component takes;
  `options` must name every word a prop takes and no other; `toggle` and `text` are offered only
  to a boolean and a string; a state's `args` are the component's props. `specimen.types.test`
  holds each refusal with a `@ts-expect-error`.
- **How a prop is turned**: `options` (a short list of words), `choices` (named values that are not
  words, an icon), `toggle`, `text`, or nothing, for a function, an element or a fixture.
- **A component that needs a whole screen** says `stands: 'screen'`: each state owns the frame's
  viewport (`Screen` in `kit-docs/specimens/ground`), and it is staged as a place is, fitted to the
  stage, with the specimen's words under its frames. A shell and the modal stand this way. Parts
  are measured only for a component drawn at its own size.
- **A component whose frames are already staged** is `specimenOf(subject, …)`: its states are the
  subject's, and the specimen adds the words (`SplitShell`, `PagedList`).
- **A state `args` cannot say** is a `draw` state: a controlled component in a small local
  component that owns its state with the kit's own hook, or a sibling export beside the lead one.
  Its props cannot be turned from the bar.
- **It is found by its name.** `kit-docs/specimens/index.ts` gathers every `*.specimen.tsx` in the
  UI package's `kit/` and in this folder's, in the catalogue's order. Its states become frames of
  `kit-<id>` (`kit-docs/specimens/frames`), so `drive`, `probe` and a board reach them like any
  other.
- **A specimen is the workbench's, never the app's.** The web's Tailwind scan leaves it out, and
  ESLint and the web's `src/frame/boundaries.test.ts` fail an import of one; in the UI package
  nothing shipped imports one (`src/boundaries.test.ts`). The engine never counts it among the
  callers of what it draws, nor its file among the design root's sources.
- **True size.** A specimen's frame is a real viewport at the width the bar names, cut to the
  `[data-specimen]` box inside it (`bench/Frame`, hugged), so nothing is scaled and a breakpoint
  still applies: a Button is 32px tall at desktop and 40px on a phone.
- **`args`** in the address is what was turned on the one state showing (`weight:quiet,icon:plus`),
  and the frame's own address carries it, so `Open` shows what the stage showed. Stepping to
  another state forgets it.
- **`parts=on`** outlines each named part on the live frame and prints its box, padding, type and
  colours as `bench/measure` reads them off the page. Nothing in that table is typed by anyone.
- **`try`** is a token value being tried (`color-ui-ink.dark:%23ffb000`). `bench/Frame` writes it
  onto the root of every frame drawn in that appearance, on every page of the bench; it is carried
  between places and the words line says `Trying n values` until `Reset`. Never written to the
  stylesheet. A frame opened on its own with `Open` does not take it.
- **Everything the bench takes from the app comes through the binding.** `workbench.config.ts`
  names the theme's source, the ground token, the app's name and the files a token's users are
  counted in, and the server hands them to the bench as modules (`bench/host.ts`,
  `bench/host-sources.ts`). `workbench.setup.tsx` hands it the kit's catalogue, the places, the
  candidates and `kit-docs/specimens/` (the specimens and the `Ground` they stand on).

| Command                                        | What it prints                                                    |
| ---------------------------------------------- | ----------------------------------------------------------------- |
| `workbench kit`                                | Every kit component, its callers and its line                     |
| `workbench kit button`                         | One component: its states, its props, its callers                 |
| `workbench kit button --state running --parts` | One state alone, with what the page measures for each named part  |
| `workbench kit button --parts --width mobile`  | The same at a phone's width. `--theme light` for the light frames |
| `workbench tokens`                             | Every token: dark and light value, contrast on its ground, users  |

Every component of the kit with a page has a specimen, and `kit-docs/specimens/index.test` fails
when the catalogue (`kit-docs/index.ts`) and the pages stop listing the same things. A specimen's
frames are swept and probed like any other (`bun workbench sweep kit-<id>`).

## Boards

A board is the page a round shows before anything is built (the skill's `BOARDS.md`). It lives in
the session's scratchpad, never here — and **its drawings are the workbench's own frames**, not
redrawings of them.

`bun workbench board <folder>` builds four files into the board's folder, from the working tree,
and refuses a folder inside the repo:

| File         | What it is                                                                                        |
| ------------ | ------------------------------------------------------------------------------------------------- |
| `frame.js`   | The kit, every frame `lib/registry` stages, the board's own drawings, and the app's React         |
| `frame.css`  | **The app's sheets** (through `index.css` here), plus whatever classes the board's drawings added |
| `frame.html` | The document every frame on the board opens. What it draws is in the `name` its frame carries     |
| `board.js`   | Turns a `[data-frame]` box on the page into that frame, at its true size and scaled to fit        |

```html
<div data-frame="notes:notes" data-width="390" data-height="844"></div>
<div data-frame="notes:notes?theme=light" data-width="1440" data-height="900"></div>
<script src="board.js"></script>
```

- **`data-frame` is `id:state`, as `drive` and `--frames` write it**, then the frame address's own
  params: `theme`, `rate`, `t`, and `where` for a place that has walked (`lib/board-spec`). Any
  state `workbench frames` lists can be named. The state may be left off; the first one draws.
- **A frame is a real viewport.** `board.js` lays the iframe out at `data-width` × `data-height`
  and scales it with a transform, which is why the place folds at 390 and splits at 1440 on the
  same page. Absent sizes mean a phone, `lib/viewports`' `mobile`.
- **`Board.theme('light')`** turns every frame on the page over; `Board.mount(element)`
  frames a box added later.
- **A new drawing is `<id>.tsx` beside the page, written exactly as a prototype is**: it imports
  `../kit`, `../fixtures/…` and `@jswob/design-workbench` (for `Subject`), and exports
  `<camelId>: Subject`. The build resolves the relative ones from `proto/`, where the file would
  stand, so `workbench new <id> --from <folder>/<id>.tsx` makes the approved drawing the candidate
  without an edit. A file that really is beside the drawing still wins.
- **Run the verb again after every change to a drawing.** Nothing is watched, and nothing is
  stored: the kit is whatever the working tree is when it is built.
- **The frame is `components/FrameDocument`**, which `/frame/$kind/$id` draws too. A place on a
  board is the place the stage shows by construction.
- **Nothing the kit writes is named after the app**: not its files, not the page's global, not the
  two messages a frame and its page exchange (`boardFrame`, `boardTheme`). The machinery is the
  engine's (`cli/board`, `lib/board-*`) and goes to another repo as it is.

**The frame's `body` is the app's, on a board and on the stage.** `index.css` here adds no rule to
it, so what a place portals to `body` is set in the app's face everywhere it is drawn.

## Settled: the UI package

Settled code stands in its own package, `packages/ui` (`@app/ui`), so the app and the workbench draw
**one copy**: the web draws it and the workbench stages it, and it imports neither. Its `CLAUDE.md`
says what it holds and the walls around it.

| Where                                               | What                                                                                                |
| --------------------------------------------------- | --------------------------------------------------------------------------------------------------- |
| `packages/ui/src/{theme,scroll,glaze,document}.css` | The tokens and the stylesheet utilities. The web's `src/index.css` and `index.css` here import them |
| `packages/ui/src/kit/`                              | Settled kit modules with their own barrel, each with its specimen beside it                         |
| `packages/ui/src/views/`                            | The frame, the destinations and the pages of settled views. Fixtures never follow                   |
| `packages/ui/src/lib/`                              | What both lean on (`plural`)                                                                        |

- **Move with the verb**: `bun workbench promote kit/<Name> --to app --dry`, then without `--dry`,
  then `bun lint:fix`. It refuses and lists every import that would not resolve from the UI
  package — fixtures, `*-places.tsx`, `kit-docs`, an unmoved sibling. That list is the hand split:
  the fixture constant goes to `fixtures/`, the sibling moves first. A specimen does not travel
  with its module yet: move `<Name>.specimen.tsx` beside it by hand.
- **A test that needs fixtures stays here** and reaches the settled module through `@app/ui/...`.
- **Caller counts, the census and `check` read both places.** Settled files are named from the repo
  root (`packages/ui/src/kit/Mark.tsx`), so the kit page and `promote` count a file once.
- **The app never imports the workbench or a specimen** — ESLint and the web's
  `src/frame/boundaries.test.ts` both fail it.
- **The verb moves an untracked file too**, where `git mv` cannot.
- **What the fixture world supplies to settled code** lives in `fixtures/`: the rail's resting facts
  and the ground's reading (`rail-facts`), the notes the Notes place opens on (`notes`), and the
  stand-in sign-in server (`sign-in-server`). The workbench frames a place with `views/place-frame`
  (`framePlace`), handing the app's `Frame` those fixtures and the home page as its ledger.
- **What stays here** is what only the workbench draws — the places and what stages them, the
  stress page — and the tests that need fixtures (`views/frame.test`, `views/flow-frame.test`).

## The four spaces

Tokens, kit, views, proto. A page belongs to exactly one. A place is argued against its earlier
drawing on a board, never against a mirror kept in the workbench.

## Views — places

**A view is a place you can be: the ground, or one errand of one destination over it.**

| id        | destination | errand                     | states                                                       |
| --------- | ----------- | -------------------------- | ------------------------------------------------------------ |
| `home`    | the ground  | the app's name and its map | home · menu                                                  |
| `notes`   | `notes`     | write, read, delete notes  | notes · empty · loading · failed                             |
| `account` | `account`   | appearance, sign out       | account · signed-out · signing-in · not-set-up · unreachable |

**A place lists a state only when the live frame cannot be driven into it** — a different record, a
different mode of it, or a condition on it. Adding a note, arming a delete and answering it are
presses, so none of them is listed: open the place and do it. The Notes place keeps its notes in a
store the variant opens with, so a note added stands first and a note deleted goes.

The exception is **the raised register**, and it is counted: `home menu`, once in the whole space,
because it is the phone's entire navigation. `views/frame.test` fails a second.

The other counted exception is **a nested page listed open**, through `ViewDraw.nested`: at most
one per place, on a record the place's nest opens. `views/frame.test` fails a second. No place
lists one yet.

A destination is a cell in the rail before it is a place: the frame still opens one with nothing
drawn in it, its head from the rail and skeleton rows under it. `undrawnDests()` in `views/index`
names them, and `views/frame.test` fails while any is left.

**There are no phone variants and no press variants.** Every state draws at every width from one
seed, so `?width=mobile` is a control and never a variant; and every state the frame can be driven
into from another is reached by driving it, not by listing it. `ViewDraw` has no field for a hand's
position — the ledger's, a peek, a scroll — so the compiler refuses one.

A place declares a `ViewDraw` — a page, what it calls itself, the ledger, an overlay, what the slug
says — and **never its own chrome**. `views/frame.tsx` in the UI package is the only file allowed to
import `SplitShell` or `RailShell`, and `views/frame.test` asserts it stays that way.

**The frame is live.** `views/frame` seeds `kit/useSplit`, so the rail's cells, the logo and the slug
all work — a seed is a starting position, not a still.

**A frame walks.** `views/flow-frame` holds one mounted `Frame` around whichever place it stands on:

- **A rail cell and a register row land on the place drawn there**, in that place's first state, at
  every width. A destination with several places lands on the first of them in `PLACES`. The
  frame's own destination is its own place again, in the state it was opened on. A destination
  nothing is drawn in still draws the waiting page.
- **A press that would lose something asks first.** A place whose state holds something
  (`ViewDraw.held`) is guarded by the frame: a rail press raises the confirm, naming where it was
  going, and goes on the discard.
- **Where it stands is one word in the frame's own address**, `where` (`lib/frame-position`): a
  place's id, or a destination's where nothing is drawn in it. No word is home, so a frame that
  never moves has the address it always had. It is written with `replaceState`: a walk is never a
  history entry, a reload keeps it, and nothing is mounted again. The arithmetic is `views/flow`.
- **The stage's `Open` follows the frame** (`bench/Frame`'s `onPosition`, `frameUrl`'s `where`): it
  opens the frame where it stands now, and in Widths the one that moved last. The frame's `src`
  never carries the word, or every step would load the frame again. A switch on the bar that
  redraws the frames (`T`, `M`) puts them back at rest.
- **The stage says nothing of a walk**: the line of print over a frame goes on naming the state it
  was opened on, and no act on it puts the frame back.
- **A board pins a walk with the same word**: `data-frame="notes:notes?where=account"`.
- **What a walk does not carry.** A place walked to is drawn fresh and holds nothing of the page
  that was left. A hold declared by the home state is read once by the frame, so coming home after
  discarding it finds the page without its hold; load the frame again for it.
- **The nest is not in the word.** A nested page keeps its own history entry, a hash on the frame's
  address, and its listed `nested` seed.
- **A sweep is not moved by any of it**: it loads the frame's plain address before every press.

The frame is `kit/SplitShell` — the ledger, the rail on the split, and the page — with the page
itself `kit/PageSpread` and the scroll drawn on the page-break rule by `kit/RuledScroll`. Below `lg`
it delegates to `kit/RailShell`, where the rail folds into the slug at the foot.

**The ledger is the home page** (`views/home`): the app's name and what each destination is for.
`views/place-frame` hands it to every frame as the resting ledger, as the web's own ground does; a
place overrides it only to show what its own page did to it.

Compare with `/views/notes?frames=notes:notes,notes:empty`, and at 390 with
`?variants=notes&width=mobile`.

Protos: **empty**. The sidebar's Proto group holds only what is unsettled, and says so when nothing
is. A question that settles leaves the space, and its losing candidates and harness are deleted
with it; what they taught is in `BANS.md`. **Do not recreate them to "compare".**

---

## File map

```
apps/workbench/    the app's half: the design root (`@app/workbench-app`)
  BANS.md          the live rules, folded, with revoked ones struck
  WORKBENCH.md     this file
  workbench.config.ts  the terminal's half of the binding: the port, the Vite config, the theme,
                   the settled spaces, the bans, the census, the gates `check` runs, and where
                   the skill's copy lives
  workbench.setup.tsx  the browser's half: the kit's catalogue, the places, the candidates, the
                   specimens and the frames they stand on
  vite.config.ts   the plugins a frame is drawn with: Tailwind and React, nothing else
  index.css        imports the UI package's sheets as the web's does and adds @source lines,
                   nothing else; motion is left real (see the engine's lib/motion.ts)
  kit-docs/        the kit's catalogue (index.ts) · stress.tsx · specimens/ (index.ts gathers
                   every *.specimen.tsx; frames makes their states frames; ground is what
                   they stand on)
  kit/             index.ts: the UI package's kit as the one barrel a candidate or a board
                   imports as ../kit. **Nothing here imports views/, fixtures/ or kit-docs/ —
                   kit.test asserts it**
  views/           the places. types.ts (Place · ViewDraw) · <section>-places.tsx · index.ts ·
                   frame.test.ts · place-frame (a place as a frame) · flow-frame (**the frame
                   that walks**) · flow (where a walked frame stands)
  proto/           prototypes — only what is unsettled. Empty right now
  fixtures/        literal data the places draw on: notes · rail-facts · sign-in-server
  theses/          <id>.thesis.ts — a candidate's numbers, which `conform` holds the frames to.
                   Created by `new`
  probe/           baseline.json · sweep-baseline.json — the findings read and held. Created by
                   the first --accept

packages/ui/src/   the settled code the web and the workbench both draw (its CLAUDE.md)
  kit/             the settled primitives, each with <Name>.specimen.tsx beside it. SplitShell +
                   useSplit is the frame; PageSpread / PageColumn is the page in it; RailShell is
                   the below-lg half the split delegates to. **Nothing here imports views/ —
                   boundaries.test asserts it**
  views/           settled drawings. destinations.ts · frame.tsx (**the only file that may
                   import SplitShell**) · nest.tsx (a page opening a record inside itself —
                   useNest) · home · notes · account · sign-in · read-failed
  lib/             plural
  *.css            theme · scroll · glaze · document

@jswob/design-workbench   the engine: nothing in it is this app's
  skill/           the design skill's source. `bun workbench skill` copies it into
                   .claude/skills/design, where PROJECT.md alone is this repo's own
  src/
    index.ts       what a design root imports: Subject, specimen, defineSetup, the frame's place
    config.ts      defineWorkbench — what workbench.config.ts is checked against
    vite.ts        how the bench is served: the app's own plugins and aliases, behind the stubs
    index.html · main.tsx · router.ts   the bench's document, React root and router
    routes/        _bench (the chrome, a pathless layout) · _bench.index (opens on the ground)
                   _bench.views.index · _bench.views.$view · _bench.proto.$name
                   _bench.kit.index · _bench.kit.$component · _bench.kit.stress
                   _bench.tokens · _bench.$ (no such page) · frame.$kind.$id (outside the chrome)
    bench/         the workbench's chrome: Shell · Sidebar · Stage · Frame · the pages it
                   draws · bench.css · fonts/. See The bench
    components/    SubjectFrame · MissingEntry
                   FrameDocument (**what a frame is** — the frame route and a board both draw it)
    lib/           registry (**the one lookup that crosses the four spaces**)
                   subject (the Subject shape every frame takes) · setup (the binding's shape)
                   frame-url · stage-search · viewports
                   frame-position (where a frame stands, kept in its own address)
                   scan-source · staged-interaction · proposed-shape · use-element-size
                   motion (**the frame clock**)
                   board-spec · board-frame · board-page (a board's frames; see Boards)
    cli/           the verbs. main.ts is `bun workbench`
    probe/         the measured floor and the cross-drawing census
```

The engine serves its own document with the plugins in `vite.config.ts` here, behind its own router
plugin, on port 5175. Nothing is built to disk; its dependency cache is
`apps/workbench/node_modules/.vite-workbench`. `bun workbench` from the monorepo root is this
folder's script, which reads `workbench.config.ts` beside it.

Tests are collocated, `[name].test.ts`, run with `bun designs:test`, and the types with
`bun designs:typecheck`. Test paths are relative to **this folder** — `fixtures/foo.test.ts`, not
`apps/workbench/...`. The UI package's are `bun ui:test` and `bun ui:typecheck`; the engine's own are the
`workbench:test` and `workbench:typecheck` scripts of the repo that holds its source.

---

## Gotchas specific to this build

**A brand-new route file needs a restart too**, and it is the engine's to add
(its `src/routes/`). The TanStack Router plugin runs in the workbench's server, not
the app's copy of it but the engine's own: it code-splits only the routes its own generator has
listed, so without it every frame would load the whole bench. It writes `routeTree.gen.ts` only
when a route is added, renamed or removed, and then the running server can end up in a full-reload
loop — the page reloads about once a second and the React root never mounts, with no console
error. Symptom: repeated `[vite] connecting… / connected`. Restart, do not debug.

**Never read `index.css` here with `?raw`.** It is the workbench's Vite CSS entry; reading it from
a module its own styles apply to makes every CSS update invalidate that module. The tokens live in
their own file, `packages/ui/src/theme.css`, for exactly this reason.

**The app's source reaches the bench as text the server reads** (`virtual:workbench/sources`,
`virtual:workbench/spend`), not as `?raw` imports, so a caller count and a token's users are read
by the function the verbs use. Those modules exist only on the workbench's server, so a test here
cannot import them. Assert on file contents with `node:fs` instead.

**Tailwind content scope.** The web's `src/index.css` scans its own `src/` and the UI package's,
without the specimens and the tests (`@source not`); `index.css` here scans nothing it is not told
to (`source(none)`), then adds the UI package and this folder's own scripts with `@source`.
**Anything invented here, or in a specimen, can never leak into production** — new tokens and
utilities are safe by construction. The engine's own source is not scanned: its chrome is plain CSS
and what it draws inside a frame is styled inline, so it needs no utility of the app's.

**`index.css` here imports the UI package's sheets and adds no token to them.** The theme's
`--color-ui-*`, `--text-ui-*` and `--spacing-ui-*` are the only tokens a frame has, and every
utility a frame draws with is a `ui-` one (`bg-ui-bg`, `text-ui-ink-2`). The faces are Archivo and
Geist Mono, loaded by the `head` in `workbench.config.ts` with the same link as the web's
`index.html`. The bench takes none of it: `bench/bench.css` is its own, in `wb` names, and no frame
loads it.

**Motion is real here.** The engine's `lib/motion.ts` shims the frame clock and settles or pins each
animation by hand, so real durations survive: `getAnimations()` reports true timing, and `?rate=0`
means _off_ rather than _declared as zero_. `index.css` here never zeroes a duration. The general
consequences are in the skill's `WORKBENCH.md`.
