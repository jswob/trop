import type { ElementHandle, Page } from '@playwright/test';

import type { Appearance } from '../lib/appearance';

import { frameUrl } from '../lib/frame-url';
import { VIEWPORTS, type ViewportName } from '../lib/viewports';
import { bool, flag, type Argv } from './args';
import { host } from './host';
import { readFrame, type FrameRead, type Target } from './inspect';
import { selectOne } from './select';
import { describeStep, matchTargets, parseSteps, type Step } from './steps';
import { readCensus, settle, withWorkbench } from './workbench';

/**
 * **Probe 11 — "the thing has been _used_, not looked at" — with a command behind it.**
 *
 * It was the one duty in `PROBES.md` that named no tool, and the gap was expensive rather than
 * theoretical. One round's question was whether a listed state could be reached by pressing the live
 * frame, thirteen times over; each answer cost a driver script written to the repo root (the
 * scratchpad cannot resolve `playwright-core`), a guessed selector, a timeout, a `page.evaluate`
 * dump of the real accessible names, a re-run, and a delete. Thirty-five references to throwaway
 * drivers in one session, and the evidence they produced — *2630 → 2893 chars* — survived only as a
 * number somebody typed into a report.
 *
 * Three things this refuses to do, each because the manual version did it and it cost something:
 *
 * - **It never takes the first match.** `matchTargets` tiers the resolution and ambiguity is an
 *   error naming every candidate. Taking the first is how a press landed on the sheet's control
 *   instead of the document's and reported a state that does not exist.
 * - **It never reads a coordinate off an image.** `--shot` writes a file for a person to look at;
 *   every number in the report comes from `readFrame` running in the page. `PROBES.md` is
 *   unambiguous about which of those is evidence.
 * - **It prints the before-number.** A frame read only after the press proves the current state,
 *   not the change, and the change is the entire question.
 */
type DriveOptions = {
	url: string;
	token: string;
	width: ViewportName;
	steps: Step[];
	listOnly: boolean;
	shot: string | undefined;
	motion: boolean;
	all: boolean;
	/** The appearance the frame is drawn in. */
	theme: Appearance;
};

function pad(value: string, width: number): string {
	return value.length >= width ? value : value + ' '.repeat(width - value.length);
}

function delta(before: number, after: number): string {
	const change = after - before;
	const sign = change > 0 ? '+' : '';
	return `${String(before).padStart(6)} → ${String(after).padStart(6)}   ${change === 0 ? '—' : `${sign}${change}`}`;
}

/** Let React commit and the frame repaint before it is read again. */
export async function paint(page: Page): Promise<void> {
	await page.evaluate(
		() =>
			new Promise(resolve =>
				requestAnimationFrame(() => requestAnimationFrame(() => resolve(null)))
			)
	);
	await page.waitForTimeout(60);
}

/** The live element behind target `index`, parked on `window` by the read that reported it. */
export async function handleFor(page: Page, index: number): Promise<ElementHandle<HTMLElement>> {
	const handle = await page.evaluateHandle(
		position =>
			(window as unknown as { __DESIGN_TARGETS__: HTMLElement[] }).__DESIGN_TARGETS__[position],
		index
	);
	const element = handle.asElement();
	if (element === null) throw new Error(`target ${index} is no longer in the frame`);
	return element;
}

function named(targets: Target[], all: boolean): Target[] {
	return targets.filter(target => (all || target.visible) && target.name !== '');
}

/**
 * Where each named target sits, keyed so one name appearing eight times stays eight entries.
 *
 * Matching by name alone would collapse the running order's eight `Configure Lead guitar` buttons
 * into one and report a reflow that moved all of them as having moved nothing.
 */
export function positions(targets: Target[], all: boolean): Map<string, { x: number; y: number }> {
	const seen = new Map<string, number>();
	const places = new Map<string, { x: number; y: number }>();

	for (const target of named(targets, all)) {
		const ordinal = (seen.get(target.name) ?? 0) + 1;
		seen.set(target.name, ordinal);
		places.set(`${target.name}#${ordinal}`, { x: target.cx, y: target.cy });
	}

	return places;
}

/** How far the layout moved: how many targets shifted, and the largest single shift. */
export function shifted(
	before: FrameRead,
	after: FrameRead,
	all: boolean
): { count: number; most: number } {
	const was = positions(before.targets, all);
	const now = positions(after.targets, all);

	let count = 0;
	let most = 0;

	for (const [key, from] of was) {
		const to = now.get(key);
		if (to === undefined) continue;

		/*
		 * A line's worth, the same floor `probe/probes.ts` uses on vertical overflow and for the
		 * same reason: sub-pixel rounding, a sticky row and a scrollbar appearing all shift a box
		 * by a few pixels, and a probe that calls those a state change is one nobody believes.
		 * The shifts that matter here are the ones a reflow makes — 596px, 782px.
		 */
		const distance = Math.max(Math.abs(to.x - from.x), Math.abs(to.y - from.y));
		if (distance > 12) {
			count += 1;
			most = Math.max(most, distance);
		}
	}

	return { count, most };
}

/**
 * Resolves one step's target, or throws with the candidates.
 *
 * The error is the useful half of this function. A name that matches nothing prints the ten
 * nearest; a name that matches several prints all of them. Either way the next command is a
 * copy-paste rather than a hunt through the source for what the control is actually called —
 * which is the round-trip this whole file exists to delete.
 */
function resolve(read: FrameRead, pattern: string, all: boolean): number {
	const visible = read.targets.flatMap((target, index) =>
		(all || target.visible) && target.name !== '' ? [{ target, index }] : []
	);
	const matches = matchTargets(
		visible.map(entry => entry.target.name),
		pattern
	);

	if (matches.length === 1) {
		const only = visible[matches[0] ?? -1];
		if (only !== undefined) return only.index;
	}

	if (matches.length === 0) {
		const nearest = visible
			.map(entry => `  ${entry.target.role}  "${entry.target.name}"`)
			.slice(0, 10);
		const globbed = pattern.includes('*')
			? `\n"${pattern}" is a glob and is anchored at both ends — "*${pattern.replace(/^\*|\*$/g, '')}*" matches anywhere.`
			: '';

		throw new Error(
			`nothing named "${pattern}" on this frame. ${visible.length} named targets, first ten:\n${nearest.join('\n')}${globbed}\n\nRun with --list for all of them.`
		);
	}

	const candidates = matches
		.map(position => visible[position])
		.filter((entry): entry is { target: Target; index: number } => entry !== undefined)
		.map(
			entry =>
				`  ${entry.target.role}  "${entry.target.name}"  at ${entry.target.x},${entry.target.y}`
		);

	throw new Error(
		`"${pattern}" matches ${matches.length} targets — name one exactly:\n${candidates.join('\n')}`
	);
}

async function perform(page: Page, step: Step, read: FrameRead, all: boolean): Promise<string> {
	switch (step.kind) {
		case 'click': {
			const index = resolve(read, step.target, all);
			const target = read.targets[index];
			await (await handleFor(page, index)).click({ timeout: 5000 });
			return `${target?.role ?? 'target'} "${target?.name ?? ''}"`;
		}
		case 'fill': {
			const index = resolve(read, step.target, all);
			const target = read.targets[index];
			await (await handleFor(page, index)).fill(step.value, { timeout: 5000 });
			return `${target?.role ?? 'field'} "${target?.name ?? ''}" ← "${step.value}"`;
		}
		case 'press':
			await page.keyboard.press(step.key);
			return `key ${step.key}`;
		case 'tab':
			for (let count = 0; count < step.times; count += 1) await page.keyboard.press('Tab');
			return `${step.times} Tab press${step.times === 1 ? '' : 'es'}`;
		case 'scroll':
			await page.evaluate(by => {
				const scroller = [...document.querySelectorAll<HTMLElement>('*')]
					.filter(element => {
						const style = getComputedStyle(element);
						return (
							(style.overflowY === 'auto' || style.overflowY === 'scroll') &&
							element.scrollHeight > element.clientHeight + 4
						);
					})
					.sort((a, b) => b.clientHeight - a.clientHeight)[0];
				if (scroller) scroller.scrollTop += by;
				else window.scrollBy(0, by);
			}, step.by);
			return `${step.by > 0 ? '+' : ''}${step.by}px`;
		case 'wait':
			await page.waitForTimeout(step.ms);
			return `${step.ms}ms`;
	}
}

/** The names a press brought and the names it took away. */
function turnover(
	before: FrameRead,
	after: FrameRead,
	all: boolean
): { was: Set<string>; now: Set<string>; appeared: string[]; gone: string[] } {
	const was = new Set(named(before.targets, all).map(target => target.name));
	const now = new Set(named(after.targets, all).map(target => target.name));

	return {
		was,
		now,
		appeared: [...now].filter(name => !was.has(name)),
		gone: [...was].filter(name => !now.has(name)),
	};
}

/**
 * Whether anything changed between two reads: content, layout, an open dialog or the appearance.
 *
 * The one question `drive` and `sweep` both ask of a press, so it is asked in one place.
 */
export function moved(before: FrameRead, after: FrameRead, all: boolean): boolean {
	const { appeared, gone } = turnover(before, after, all);

	return !(
		appeared.length === 0 &&
		gone.length === 0 &&
		before.chars === after.chars &&
		before.words === after.words &&
		before.boxes === after.boxes &&
		before.columns === after.columns &&
		before.dialog === after.dialog &&
		before.theme === after.theme &&
		shifted(before, after, all).count === 0
	);
}

export function report(before: FrameRead, after: FrameRead, all: boolean): string {
	const { was: namesBefore, now: namesAfter, appeared, gone } = turnover(before, after, all);

	const move = shifted(before, after, all);
	const held = positions(before.targets, all).size;

	const lines = [
		`  ${pad('text', 10)} ${delta(before.chars, after.chars)}  chars`,
		`  ${pad('boxes', 10)} ${delta(before.boxes, after.boxes)}  painted`,
		`  ${pad('targets', 10)} ${delta(namesBefore.size, namesAfter.size)}  named`,
	];

	/*
	 * Layout, reported whenever it moved. A state reached by clipping a column away changes no
	 * text at all: one round's evidence for deleting `collapsed` was "1 → 2 columns", which is
	 * this block and none of the three content lines above it.
	 */
	if (before.columns !== after.columns) {
		lines.push(`  ${pad('columns', 10)} ${delta(before.columns, after.columns)}  on the page`);
	}
	if (move.count > 0) {
		lines.push(
			`  ${pad('moved', 10)} ${String(move.count).padStart(6)} of ${held} targets shifted, largest ${move.most}px`
		);
	}
	if (before.extent.x !== after.extent.x || before.extent.right !== after.extent.right) {
		lines.push(
			`  ${pad('extent', 10)} ${before.extent.x}–${before.extent.right} → ${after.extent.x}–${after.extent.right}  px used`
		);
	}
	if (before.scrollY !== after.scrollY) {
		lines.push(`  ${pad('scroll', 10)} ${delta(before.scrollY, after.scrollY)}  px`);
	}
	if (before.dialog !== after.dialog) {
		lines.push(`  ${pad('dialog', 10)} ${before.dialog ?? 'none'} → ${after.dialog ?? 'none'}`);
	}
	if (before.theme !== after.theme) {
		lines.push(`  ${pad('theme', 10)} ${before.theme} → ${after.theme}`);
	}

	const quote = (names: string[]): string =>
		names
			.slice(0, 6)
			.map(name => `"${name.length > 44 ? `${name.slice(0, 43)}…` : name}"`)
			.join(', ') + (names.length > 6 ? `, and ${names.length - 6} more` : '');

	if (appeared.length > 0) lines.push(`  ${pad('appeared', 10)} ${quote(appeared)}`);
	if (gone.length > 0) lines.push(`  ${pad('gone', 10)} ${quote(gone)}`);

	const still = !moved(before, after, all);

	lines.push('');
	if (still) {
		lines.push('  RESULT: NOTHING MOVED — no content and no layout changed. Either the control');
		lines.push('  is inert, or it is not the one you meant. Check the name against --list.');
	} else {
		lines.push('  RESULT: the press reached a state.');
	}

	return lines.join('\n');
}

export async function drive(argv: Argv): Promise<number> {
	const widthName = (flag(argv, 'width') ?? 'desktop') as ViewportName;
	if (VIEWPORTS[widthName] === undefined) {
		throw new Error(`unknown width "${widthName}" — one of ${Object.keys(VIEWPORTS).join(', ')}`);
	}

	const token = argv.positional[0] ?? flag(argv, 'frame');
	if (token === undefined || token === '') {
		throw new Error('name a frame: workbench drive <id>:<state> [--list | --do "click:…"]');
	}

	const options: DriveOptions = {
		url: flag(argv, 'url') ?? host().url,
		token,
		width: widthName,
		steps: parseSteps(flag(argv, 'do') ?? ''),
		listOnly: bool(argv, 'list'),
		shot: flag(argv, 'shot'),
		motion: bool(argv, 'motion'),
		all: bool(argv, 'all'),
		theme: flag(argv, 'theme') === 'light' ? 'light' : 'dark',
	};

	if (options.steps.length === 0 && !options.listOnly && options.shot === undefined) {
		options.listOnly = true;
	}

	return withWorkbench(options.url, async browser => {
		const census = await readCensus(browser, options.url);
		const entry = selectOne(census, options.token);
		const viewport = VIEWPORTS[options.width];

		const context = await browser.newContext({
			viewport: { width: viewport.width, height: viewport.height },
		});
		const page = await context.newPage();

		try {
			/* `rate: 0` is off, not a stopped clock: every animation settled at its end state. */
			const address = frameUrl(
				entry.kind,
				entry.id,
				entry.variant,
				entry.chrome,
				options.motion ? {} : { rate: 0 },
				options.theme
			);
			await page.goto(`${options.url}${address}`, { waitUntil: 'load', timeout: 30_000 });
			await settle(page);

			const before = await page.evaluate(readFrame);

			process.stdout.write(
				`${entry.id} · ${entry.variant} · ${options.width} ${viewport.width}px\n${options.url}${address}\n\n`
			);

			if (options.listOnly) {
				const targets = named(before.targets, options.all);
				const roleWidth = Math.max(...targets.map(target => target.role.length), 4);
				for (const target of targets) {
					const where = `${String(target.x).padStart(4)},${String(target.y).padStart(4)}`;
					const state = target.disabled ? '  [disabled]' : '';
					process.stdout.write(
						`  ${pad(target.role, roleWidth)}  ${where}  "${target.name}"${state}\n`
					);
				}
				const hidden = before.targets.filter(target => !target.visible).length;
				process.stdout.write(
					`\n${targets.length} named target${targets.length === 1 ? '' : 's'}${hidden > 0 && !options.all ? ` · ${hidden} not drawn at this width (--all to include)` : ''}\n`
				);
				/*
				 * The list is a preface, not an alternative. Returning here made `--list --do` drop
				 * the steps in silence — a requested press that never happened and said nothing,
				 * which is the one failure this whole file exists to make impossible.
				 */
				if (options.steps.length > 0) process.stdout.write('\n');
				else if (options.shot === undefined) return 0;
			}

			let read = before;
			for (const [index, step] of options.steps.entries()) {
				const did = await perform(page, step, read, options.all);
				await paint(page);
				read = await page.evaluate(readFrame);
				process.stdout.write(`  step ${index + 1}  ${describeStep(step)}\n          → ${did}\n`);
			}

			if (options.steps.length > 0) {
				process.stdout.write(`\n${report(before, read, options.all)}\n`);
			}

			if (options.shot !== undefined) {
				await page.screenshot({ path: options.shot, fullPage: false });
				process.stdout.write(
					`\n  shot       ${options.shot} (${viewport.width}×${viewport.height})\n`
				);
			}

			return 0;
		} finally {
			await context.close();
		}
	});
}
