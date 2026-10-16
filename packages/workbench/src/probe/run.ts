import { writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import process from 'node:process';

import type { Browser, Page } from '@playwright/test';

import { bool, flag, list, type Argv } from '../cli/args';
import { host } from '../cli/host';
import { selectFrames } from '../cli/select';
import { resolveSpaces } from '../cli/spaces';
import { readCensus, settle, withWorkbench } from '../cli/workbench';
import type { Appearance } from '../lib/appearance';
import type { FrameEntry } from '../lib/frame-census';

import { frameUrl, type FrameKind } from '../lib/frame-url';
import { VIEWPORT_NAMES, VIEWPORTS, type ViewportName } from '../lib/viewports';
import { acceptBaseline, applyBaseline, loadBaseline } from './baseline';
import {
	contrast,
	hScroll,
	hitTest,
	installFocusBaselines,
	notEmpty,
	readFocusState,
	scopeOf,
	vOverflow,
} from './probes';
import { failures, formatReport, summarise, type FrameReport, type ProbeResult } from './report';

type Options = {
	url: string;
	spaces: FrameKind[];
	frames: string[];
	widths: ViewportName[];
	accept: boolean;
	full: boolean;
	quiet: boolean;
	/** The appearance every frame is probed in. The baseline is dark's, and only dark accepts. */
	theme: Appearance;
};

function optionsOf(argv: Argv): Options {
	const widths = list(argv, 'widths').filter((name): name is ViewportName =>
		VIEWPORT_NAMES.includes(name as ViewportName)
	);
	const asked = list(argv, 'space');

	return {
		url: flag(argv, 'url') ?? host().url,
		spaces: bool(argv, 'all') ? [] : resolveSpaces(asked.length > 0 ? asked : ['proto']),
		frames: list(argv, 'frames'),
		widths: widths.length > 0 ? widths : [...VIEWPORT_NAMES],
		accept: bool(argv, 'accept'),
		full: bool(argv, 'full'),
		quiet: bool(argv, 'quiet'),
		theme: flag(argv, 'theme') === 'light' ? 'light' : 'dark',
	};
}

/**
 * Focus visibility, driven with real Tab presses.
 *
 * `.focus()` does not set `:focus-visible`, so a probe that focuses programmatically measures a
 * state the keyboard never produces. Tab is the only way to ask the real question.
 *
 * The budget is twice the target count because Tab leaves the document for the browser's own
 * chrome and comes back, so a pass is not one press per target. It stops as soon as every
 * target has been reached rather than spending the budget.
 */
async function focusProbe(page: Page): Promise<ProbeResult> {
	const count = await page.evaluate(installFocusBaselines);
	if (count === 0) {
		return { probe: 'focus-visible', ok: true, note: 'no focusable targets', findings: [] };
	}

	const seen = new Set<string>();
	const invisible: string[] = [];

	for (let step = 0; step < count * 2 + 2 && seen.size < count; step += 1) {
		await page.keyboard.press('Tab');
		const state = await page.evaluate(readFocusState);
		if (!state || state.index === null || seen.has(state.index)) continue;
		seen.add(state.index);
		if (!state.changed && invisible.length < 5)
			invisible.push(`${state.label} shows nothing on focus`);
	}

	const unreachable = count - seen.size;

	return {
		probe: 'focus-visible',
		ok: invisible.length === 0 && unreachable === 0,
		note: `${seen.size}/${count} targets reached by Tab, ${invisible.length} with no visible focus`,
		findings: [
			...invisible,
			...(unreachable > 0
				? [`${unreachable} enabled target(s) never received focus from Tab`]
				: []),
		],
	};
}

async function probeFrame(
	browser: Browser,
	base: string,
	entry: FrameEntry,
	widthName: ViewportName,
	theme: Appearance
): Promise<FrameReport> {
	const viewport = VIEWPORTS[widthName];
	const context = await browser.newContext({
		viewport: { width: viewport.width, height: viewport.height },
	});
	const page = await context.newPage();

	try {
		/* `rate: 0` is off, not a stopped clock: every animation settled at its end state. */
		const address = frameUrl(entry.kind, entry.id, entry.variant, entry.chrome, { rate: 0 }, theme);
		await page.goto(`${base}${address}`, { waitUntil: 'load', timeout: 30_000 });
		await settle(page);

		const results: ProbeResult[] = [
			await page.evaluate(hScroll),
			await page.evaluate(vOverflow),
			await page.evaluate(hitTest),
			await page.evaluate(notEmpty),
			await focusProbe(page),
			await page.evaluate(contrast),
		];

		return { id: entry.id, variant: entry.variant, width: viewport.width, widthName, results };
	} finally {
		await context.close();
	}
}

/**
 * `probe` — the measured floor on every frame asked for: sideways scroll, overflow, hit-testing,
 * visible focus and contrast. Findings the baseline holds are left out; `--accept` holds what is
 * printed, once it has been read.
 */
export async function probe(argv: Argv): Promise<number> {
	const options = optionsOf(argv);
	if (options.accept && options.theme === 'light') {
		process.stderr.write('--accept rewrites the dark baseline; run it without --theme light.\n');
		return 1;
	}

	const floor = host().baselines.probes;

	return withWorkbench(options.url, async browser => {
		const census = await readCensus(browser, options.url);
		const targets = selectFrames(census, options);

		if (targets.length === 0) {
			const where =
				options.frames.length > 0 ? options.frames.join(', ') : options.spaces.join(', ');
			process.stdout.write(
				`Nothing to probe in [${where}]. The proto space is empty whenever nothing is being argued about — name frames with --frames, or a space with --space views.\n`
			);
			return 0;
		}

		/*
		 * Four at a time. Each frame gets its own browser context, so they were already isolated and
		 * the loop was serial only because it was written as one — 33 frames × 3 widths at roughly
		 * half a second each is the largest single wait in a round, and the round pays it on every
		 * iteration. Four is a floor on contention rather than a ceiling on speed: the probes read
		 * layout, and enough concurrent contexts start to measure each other's scheduling.
		 */
		const jobs = targets.flatMap(entry => options.widths.map(widthName => ({ entry, widthName })));
		const raw: FrameReport[] = new Array<FrameReport>(jobs.length);
		let next = 0;

		const worker = async (): Promise<void> => {
			for (let index = next++; index < jobs.length; index = next++) {
				const job = jobs[index];
				if (job === undefined) continue;
				raw[index] = await probeFrame(
					browser,
					options.url,
					job.entry,
					job.widthName,
					options.theme
				);
			}
		};

		await Promise.all(Array.from({ length: Math.min(4, jobs.length) }, worker));

		const existing = loadBaseline(floor);

		if (options.accept) {
			const note = 'Findings read, understood and expected to keep occurring. See PROBES.md.';
			const baseline = acceptBaseline(raw, existing, note, floor);
			process.stdout.write(
				`${formatReport(raw)}\n\nbaseline accepted · ${baseline.keys.length} findings held · ${path.relative(process.cwd(), floor)}\n`
			);
			return 0;
		}

		const { reports, held, resolved } = applyBaseline(raw, options.full ? null : existing);

		/*
		 * The whole report goes to a file whichever way it is printed.
		 *
		 * Both sides of one A/B run lost a full re-run — about two minutes each — to piping this
		 * through `tail`, which discards the head of the report and, because a pipe returns the
		 * last command's status, the exit code as well. A file that is always written makes a
		 * truncated pipe a nuisance instead of a repeat, and the verdict below is printed as a word
		 * so it survives the pipe that swallowed the status.
		 */
		const file = path.join(os.tmpdir(), `workbench-probe-${Date.now()}.txt`);
		writeFileSync(file, `${formatReport(reports)}\n`, 'utf8');

		if (!options.quiet) process.stdout.write(`${formatReport(reports)}\n`);

		const failing = failures(reports, scopeOf);
		process.stdout.write(`${summarise(reports, scopeOf)}`);
		if (held > 0) process.stdout.write(` · ${held} held by the baseline`);
		process.stdout.write(`\nfull report: ${file}\n`);

		if (resolved.length > 0) {
			process.stdout.write(
				`\n${resolved.length} baselined finding${resolved.length === 1 ? '' : 's'} no longer occur${resolved.length === 1 ? 's' : ''} — re-accept to prune:\n${resolved.map(key => `  ${key}`).join('\n')}\n`
			);
		}

		process.stdout.write(
			failing > 0 ? `RESULT: ${failing} FAILING the floor\n` : 'RESULT: floor clean\n'
		);

		return failing > 0 ? 1 : 0;
	});
}
