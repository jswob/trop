import { existsSync, readdirSync } from 'node:fs';
import path from 'node:path';

import type { Browser } from '@playwright/test';

import type { Appearance } from '../lib/appearance';

import type { FrameEntry } from '../lib/frame-census';
import { frameUrl } from '../lib/frame-url';
import { VIEWPORT_NAMES, VIEWPORTS, type ViewportName } from '../lib/viewports';
import { bool, flag, list, type Argv } from './args';
import { host } from './host';
import { asks, judge, type Ask, type FrameReading, type PartReading, type Thesis } from './thesis';
import { readCensus, settle, withWorkbench } from './workbench';

const theses = (): string => path.join(host().root, 'theses');
const SUFFIX = '.thesis.ts';

/**
 * Reads every part a thesis names, **inside the frame document**.
 *
 * Self-contained, as `readFrame` is: Playwright sends it across by `toString()`. A drawing with no
 * box is the hidden width's copy of a responsive frame and is left out, and a selector the page
 * cannot parse is handed back by name rather than thrown from inside the page.
 */
export function readParts(wanted: Ask[]): { parts: Record<string, PartReading[]>; bad: string[] } {
	const parts: Record<string, PartReading[]> = {};
	const bad: string[] = [];

	for (const ask of wanted) {
		let found: HTMLElement[];
		try {
			found = [...document.querySelectorAll<HTMLElement>(ask.select)];
		} catch {
			bad.push(`${ask.name}: ${ask.select}`);
			continue;
		}

		parts[ask.name] = found.flatMap(element => {
			const box = element.getBoundingClientRect();
			if (box.width <= 0 || box.height <= 0) return [];

			const range = document.createRange();
			range.selectNodeContents(element);
			const tops = new Set(
				[...range.getClientRects()]
					.filter(rect => rect.width > 0 && rect.height > 0)
					.map(rect => Math.round(rect.top))
			);
			const text = (element.innerText ?? element.textContent ?? '').replace(/\s+/g, ' ').trim();
			const computed = getComputedStyle(element);

			/* A side is room when the part draws nothing there: no ground, and no edge on that side. */
			const bare =
				computed.backgroundImage === 'none' &&
				/^(transparent|rgba\(0, 0, 0, 0\))$/.test(computed.backgroundColor);
			const roomOn = (side: 'Top' | 'Right' | 'Bottom' | 'Left'): number =>
				bare && (Number.parseFloat(computed[`border${side}Width`]) || 0) === 0
					? Number.parseFloat(computed[`padding${side}`]) || 0
					: 0;

			return [
				{
					width: box.width,
					height: box.height,
					top: box.top,
					right: box.right,
					bottom: box.bottom,
					left: box.left,
					lines: text === '' ? 0 : tops.size,
					chars: text.length,
					room: {
						top: roomOn('Top'),
						right: roomOn('Right'),
						bottom: roomOn('Bottom'),
						left: roomOn('Left'),
					},
					style: Object.fromEntries(
						ask.props.map(prop => [prop, computed.getPropertyValue(prop).trim()])
					),
				},
			];
		});
	}

	return { parts, bad };
}

async function loadThesis(id: string): Promise<Thesis> {
	const file = path.join(theses(), `${id}${SUFFIX}`);
	if (!existsSync(file)) {
		throw new Error(
			`no thesis file for "${id}" — its numbers go in theses/${id}${SUFFIX}, written when the thesis is approved`
		);
	}

	const module = (await import(file)) as { numbers?: Thesis };
	if (module.numbers === undefined) {
		throw new Error(`theses/${id}${SUFFIX} exports no \`numbers\``);
	}
	if (module.numbers.id !== id) {
		throw new Error(`theses/${id}${SUFFIX} is a thesis for "${module.numbers.id}", not "${id}"`);
	}

	return module.numbers;
}

function written(): string[] {
	if (!existsSync(theses())) return [];

	return readdirSync(theses())
		.filter(name => name.endsWith(SUFFIX))
		.map(name => name.slice(0, -SUFFIX.length))
		.sort();
}

async function readFrames(
	browser: Browser,
	url: string,
	entries: FrameEntry[],
	widths: ViewportName[],
	wanted: Ask[],
	theme: Appearance
): Promise<FrameReading[]> {
	const jobs = entries.flatMap(entry => widths.map(width => ({ entry, width })));
	const readings = new Array<FrameReading>(jobs.length);
	let next = 0;

	const worker = async (): Promise<void> => {
		for (let index = next++; index < jobs.length; index = next++) {
			const job = jobs[index];
			if (job === undefined) continue;

			const viewport = VIEWPORTS[job.width];
			const context = await browser.newContext({
				viewport: { width: viewport.width, height: viewport.height },
			});

			try {
				const page = await context.newPage();
				const { entry } = job;
				const address = frameUrl(
					entry.kind,
					entry.id,
					entry.variant,
					entry.chrome,
					{ rate: 0 },
					theme
				);
				await page.goto(`${url}${address}`, { waitUntil: 'load', timeout: 30_000 });
				await settle(page);

				const read = await page.evaluate(readParts, wanted);
				if (read.bad.length > 0) {
					throw new Error(`the page cannot read these selectors:\n  ${read.bad.join('\n  ')}`);
				}

				readings[index] = { state: entry.variant, width: job.width, parts: read.parts };
			} finally {
				await context.close();
			}
		}
	};

	await Promise.all(Array.from({ length: Math.min(4, jobs.length) }, worker));
	return readings;
}

function pad(value: string, width: number): string {
	return value.length >= width ? value : value + ' '.repeat(width - value.length);
}

/**
 * **The thesis's numbers, held against the live frames.**
 *
 * This is the conformance half of what an independent reviewer used to do by hand: thirty-three
 * review runs spent a quarter of their time writing a measuring script and throwing it away, and
 * a quarter of what they found was a number the thesis had stated (a row 69px against a declared
 * 68, a gap of 16 where the board drew 8). The numbers are written once, when the thesis is
 * approved and before anything is built, so the check does not depend on whoever built it.
 *
 * It prints what was not kept and one line of counts. `--all` prints what held as well, with the
 * figure measured, for a report that has to show both.
 */
export async function conform(argv: Argv): Promise<number> {
	const url = flag(argv, 'url') ?? host().url;
	const theme: Appearance = flag(argv, 'theme') === 'light' ? 'light' : 'dark';
	const everything = bool(argv, 'all');
	const asked = list(argv, 'widths').filter((name): name is ViewportName =>
		VIEWPORT_NAMES.includes(name as ViewportName)
	);
	const widths = asked.length > 0 ? asked : [...VIEWPORT_NAMES];

	const ids = argv.positional.length > 0 ? argv.positional : written();
	if (ids.length === 0) {
		process.stdout.write('No thesis files in theses/. Nothing to hold the frames against.\n');
		return 1;
	}

	const theses = await Promise.all(ids.map(loadThesis));

	return withWorkbench(url, async browser => {
		const census = await readCensus(browser, url);
		let missed = 0;

		for (const thesis of theses) {
			const entries = census.filter(entry => entry.id === thesis.id);
			if (entries.length === 0) {
				throw new Error(
					`no frame with id "${thesis.id}" — run "workbench frames" for the list, and restart the workbench if the candidate is new`
				);
			}

			process.stdout.write(`${thesis.id} · ${thesis.bet}\n`);

			if (thesis.checks.length === 0) {
				process.stdout.write(
					`  FAIL  no numbers written yet. They go in theses/${thesis.id}${SUFFIX} before the build.\n\n`
				);
				missed += 1;
				continue;
			}

			const readings = await readFrames(browser, url, entries, widths, asks(thesis), theme);
			const verdicts = judge(thesis, readings);
			const failing = verdicts.filter(verdict => !verdict.ok);
			const widest = Math.max(...verdicts.map(verdict => verdict.what.length), 4);

			for (const verdict of verdicts) {
				if (verdict.ok && !everything) continue;

				if (verdict.ok) {
					process.stdout.write(
						`  pass  ${pad(verdict.what, widest)}  ${verdict.said} said · ${verdict.held} measured in ${String(verdict.measured)}\n`
					);
					continue;
				}

				process.stdout.write(`  FAIL  ${pad(verdict.what, widest)}  ${verdict.said} said\n`);
				for (const miss of verdict.misses.slice(0, 8)) process.stdout.write(`          ${miss}\n`);
				if (verdict.misses.length > 8) {
					process.stdout.write(`          and ${String(verdict.misses.length - 8)} more\n`);
				}
			}

			process.stdout.write(
				`  ${String(entries.length)} state${entries.length === 1 ? '' : 's'} × ${String(widths.length)} width${widths.length === 1 ? '' : 's'} · ${String(verdicts.length)} number${verdicts.length === 1 ? '' : 's'} · ${failing.length === 0 ? 'all kept' : `${String(failing.length)} not kept`}\n\n`
			);
			missed += failing.length;
		}

		process.stdout.write(
			missed === 0
				? 'RESULT: thesis kept\n'
				: `RESULT: ${String(missed)} thesis number${missed === 1 ? '' : 's'} NOT KEPT\n`
		);
		return missed === 0 ? 0 : 1;
	});
}
