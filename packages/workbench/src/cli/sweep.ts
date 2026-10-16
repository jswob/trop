import path from 'node:path';

import type { Browser, ElementHandle, Page } from '@playwright/test';

import type { Appearance } from '../lib/appearance';

import type { FrameEntry } from '../lib/frame-census';
import { frameUrl } from '../lib/frame-url';
import { VIEWPORT_NAMES, VIEWPORTS, type ViewportName } from '../lib/viewports';
import { acceptBaseline, keyOf, loadBaseline } from '../probe/baseline';
import type { FrameReport, ProbeResult } from '../probe/report';
import { bool, flag, list, type Argv } from './args';
import { handleFor, moved, paint } from './drive';
import { host } from './host';
import { readFrame, type FrameRead, type Target } from './inspect';
import { selectFrames } from './select';
import { resolveSpaces } from './spaces';
import { readCensus, settle, withWorkbench } from './workbench';

/** The sweep's own held findings, apart from the probes': the two answer different questions. */

/** A run of alike controls longer than this is sampled: its first, its middle and its last. */
const LONG_RUN = 8;

export type Press = {
	/** Where it stands in the read that listed it. */
	index: number;
	/** Its name and which of that name it is, so it is found again after the frame reloads. */
	key: string;
	name: string;
	role: string;
};

export type Picked = {
	presses: Press[];
	/** How many alike controls were left out of long runs. */
	alike: number;
	/** How many pressable controls have no accessible name. */
	unnamed: number;
};

/**
 * Which controls of a frame get pressed.
 *
 * Everything drawn, enabled and not already standing on its answer. A long run of alike controls,
 * sixty rows of one list, is pressed at its first, middle and last: the sixty answers are one
 * answer, and pressing them all is what would make this too slow to run between edits.
 */
export function pick(targets: Target[], every: boolean): Picked {
	const seen = new Map<string, number>();
	const all: (Press & { like: string })[] = [];
	let unnamed = 0;

	targets.forEach((target, index) => {
		if (!target.visible || target.disabled || target.press !== 'press') return;
		if (target.name === '') {
			unnamed += 1;
			return;
		}

		const ordinal = (seen.get(target.name) ?? 0) + 1;
		seen.set(target.name, ordinal);
		all.push({
			index,
			key: `${target.name}#${String(ordinal)}`,
			name: target.name,
			role: target.role,
			like: target.like,
		});
	});

	const runs = new Map<string, number[]>();
	all.forEach((press, position) => {
		runs.set(press.like, [...(runs.get(press.like) ?? []), position]);
	});

	const left = new Set<number>();
	if (!every) {
		for (const run of runs.values()) {
			if (run.length <= LONG_RUN) continue;

			const kept = new Set([run[0], run[Math.floor(run.length / 2)], run[run.length - 1]]);
			for (const position of run) if (!kept.has(position)) left.add(position);
		}
	}

	return {
		presses: all
			.filter((_press, position) => !left.has(position))
			.map(({ index, key, name, role }) => ({ index, key, name, role })),
		alike: left.size,
		unnamed,
	};
}

/**
 * Whether a press scrolled something: the same controls, standing somewhere else on the glass.
 *
 * `moved` reads content coordinates, so that a click scrolling its own target into view is not a
 * reflow. A pointer whose whole job is to scroll a column changes nothing else, and read that way
 * it moved nothing. The frame is read again once the control is in view, so what shifts between
 * that read and the next is the press's own doing.
 */
export function scrolled(before: Target[], after: Target[]): boolean {
	const places = (targets: Target[]): Map<string, number> => {
		const seen = new Map<string, number>();
		const found = new Map<string, number>();
		for (const target of targets) {
			if (!target.visible || target.name === '') continue;
			const ordinal = (seen.get(target.name) ?? 0) + 1;
			seen.set(target.name, ordinal);
			found.set(`${target.name}#${String(ordinal)}`, target.y);
		}
		return found;
	};

	const now = places(after);
	for (const [key, y] of places(before)) {
		const to = now.get(key);
		if (to !== undefined && Math.abs(to - y) > 12) return true;
	}
	return false;
}

/** What one press did, as the page reported it. */
export type PressFacts = {
	name: string;
	/** Whether content, layout, a dialog, the appearance or the address changed. */
	moved: boolean;
	/** Whether the keyboard was left on the page body. */
	focusOnBody: boolean;
	/** How far the page scrolls sideways afterwards, where it did not before. */
	sideways: number;
	/** What the page threw while it was pressed. */
	errors: string[];
	/** What it opened, and how Escape went: nothing, when it opened nothing. */
	opened: { closed: boolean; focusBack: boolean } | null;
};

/**
 * What is wrong with one press, in words. Nothing, for a press that did its job.
 *
 * These are the faults thirty-three independent reviews kept finding by hand: a control that does
 * nothing, a keyboard dropped on the page, a menu Escape will not close, a page that overflows only
 * once something has been pressed. None of them needs a judgement, so none of them needs a reader.
 */
export function judgePress(facts: PressFacts): string[] {
	const findings: string[] = [];
	const said = `"${facts.name}"`;

	if (!facts.moved) findings.push(`${said} moved nothing`);
	if (facts.opened === null && facts.focusOnBody) {
		findings.push(`${said} left the keyboard on the page body`);
	}
	if (facts.sideways > 0) {
		findings.push(`${said} made the page scroll sideways by ${String(facts.sideways)}px`);
	}
	for (const error of facts.errors) findings.push(`${said} threw: ${error}`);

	if (facts.opened !== null && !facts.opened.closed) {
		findings.push(`${said} opened something Escape does not close`);
	} else if (facts.opened !== null && !facts.opened.focusBack) {
		findings.push(`${said} opened something, and closing it did not put the keyboard back on it`);
	}

	return findings;
}

/**
 * Text wider than the box it is set in, **inside the frame document**.
 *
 * A date that needs 86px in an 80px column passes every other probe: the page does not scroll and
 * nothing is covered. A box that scrolls, or one that ends its text in an ellipsis, chose to.
 */
export function textFits(): string[] {
	const findings: string[] = [];

	for (const element of document.querySelectorAll<HTMLElement>('body *')) {
		if (findings.length >= 8) break;

		const text = [...element.childNodes]
			.filter(node => node.nodeType === Node.TEXT_NODE)
			.map(node => node.textContent ?? '')
			.join(' ')
			.replace(/\s+/g, ' ')
			.trim();
		if (text === '') continue;

		const style = getComputedStyle(element);
		if (style.display === 'inline' || style.visibility === 'hidden') continue;
		if (style.overflowX === 'auto' || style.overflowX === 'scroll') continue;
		if (style.textOverflow === 'ellipsis') continue;

		const box = element.getBoundingClientRect();
		if (box.width < 8 || box.height < 4) continue;

		const over = element.scrollWidth - element.clientWidth;
		if (over <= 1) continue;

		findings.push(
			`"${text.length > 40 ? `${text.slice(0, 39)}…` : text}" is ${String(element.scrollWidth)}px wide in a ${String(element.clientWidth)}px box`
		);
	}

	return findings;
}

/** What the page says about itself around a press: where the keyboard is, and what stands open. */
function standing(): { focusOnBody: boolean; popups: number; sideways: number } {
	const active = document.activeElement;
	const popups = [
		...document.querySelectorAll<HTMLElement>(
			'[aria-modal="true"], dialog[open], [role="dialog"], [role="menu"], [role="listbox"]'
		),
	].filter(element => {
		const box = element.getBoundingClientRect();
		return box.width > 0 && box.height > 0;
	}).length;

	return {
		focusOnBody: active === null || active === document.body,
		popups,
		sideways: Math.max(
			0,
			document.documentElement.scrollWidth - document.documentElement.clientWidth
		),
	};
}

type Swept = FrameReport & { pressed: number; alike: number; unreached: string[] };

async function open(page: Page, address: string): Promise<FrameRead> {
	await page.goto(address, { waitUntil: 'load', timeout: 30_000 });
	await settle(page);
	return page.evaluate(readFrame);
}

async function pressOne(
	page: Page,
	address: string,
	handle: ElementHandle<HTMLElement>,
	press: Press,
	errors: string[]
): Promise<PressFacts | null> {
	const reach = await handle.evaluate(element => {
		const hitAt = (): Element | null => {
			const box = element.getBoundingClientRect();
			return document.elementFromPoint(box.x + box.width / 2, box.y + box.height / 2);
		};

		/* A control already under the hand is pressed where it stands: scrolling to a pinned pointer moves the column it points into. */
		let hit = hitAt();
		if (hit === null || !element.contains(hit)) {
			element.scrollIntoView({ block: 'center', inline: 'center' });
			hit = hitAt();
		}
		return hit !== null && element.contains(hit);
	});
	if (!reach) return null;

	/*
	 * The frame is read again here, with the control already scrolled to. Bringing a control into
	 * view can draw or clear a pointer at the column's edge, and against a read taken before the
	 * scroll that was charged to the press: one inert control read as moved on one run in two.
	 */
	await paint(page);
	const before = await page.evaluate(readFrame);
	const was = await page.evaluate(standing);
	const thrown = errors.length;

	try {
		await handle.click({ timeout: 1500 });
	} catch {
		return null;
	}

	await paint(page);

	if (page.url() !== address) {
		await settle(page);
		const landed = await page.evaluate(standing);
		return {
			name: press.name,
			moved: true,
			focusOnBody: landed.focusOnBody,
			sideways: was.sideways === 0 ? landed.sideways : 0,
			errors: errors.slice(thrown),
			opened: null,
		};
	}

	const stirred = (read: FrameRead): boolean =>
		moved(before, read, false) || scrolled(before.targets, read.targets);

	/*
	 * A press that seems to have moved nothing is read once more, a little later. A pointer scrolls
	 * its column smoothly, and a frame after the press the column has not got anywhere yet.
	 */
	let after = await page.evaluate(readFrame);
	if (!stirred(after)) {
		await page.waitForTimeout(400);
		after = await page.evaluate(readFrame);
	}
	const now = await page.evaluate(standing);

	/*
	 * A hand-off that waits on an answer is still a hand-off: sign-in gives the keyboard back to its
	 * field when the server refuses, a second after the press. Only a keyboard still on `<body>`
	 * after that is one nobody took.
	 */
	if (now.focusOnBody) {
		now.focusOnBody = await page
			.waitForFunction(
				() => document.activeElement !== null && document.activeElement !== document.body,
				undefined,
				{ timeout: 1000 }
			)
			.then(
				() => false,
				() => true
			);
	}
	let opened: PressFacts['opened'] = null;

	if (now.popups > was.popups) {
		await page.keyboard.press('Escape');
		await paint(page);
		const shut = await page.evaluate(standing);
		const back = await page.evaluate(readFrame);
		const focusBack = await handle
			.evaluate(element => !element.isConnected || document.activeElement === element)
			.catch(() => true);
		/* Closed is the frame back as it stood. An overlay may stay mounted, undrawn, after it shuts. */
		opened = { closed: shut.popups <= was.popups || !moved(before, back, false), focusBack };
	}

	return {
		name: press.name,
		moved: stirred(after),
		focusOnBody: now.focusOnBody,
		sideways: was.sideways === 0 ? now.sideways : 0,
		errors: errors.slice(thrown),
		opened,
	};
}

async function sweepFrame(
	browser: Browser,
	base: string,
	entry: FrameEntry,
	widthName: ViewportName,
	theme: Appearance,
	every: boolean
): Promise<Swept> {
	const viewport = VIEWPORTS[widthName];
	const context = await browser.newContext({
		viewport: { width: viewport.width, height: viewport.height },
	});
	const page = await context.newPage();
	const errors: string[] = [];
	page.on('pageerror', error => errors.push(error.message.split('\n')[0] ?? ''));

	try {
		const address = `${base}${frameUrl(entry.kind, entry.id, entry.variant, entry.chrome, { rate: 0 }, theme)}`;
		let read = await open(page, address);
		const text = await page.evaluate(textFits);
		const picked = pick(read.targets, every);

		const findings: string[] = errors.map(error => `the frame threw as it opened: ${error}`);
		if (picked.unnamed > 0) {
			findings.push(
				`${String(picked.unnamed)} control${picked.unnamed === 1 ? ' has' : 's have'} no name`
			);
		}

		const unreached: string[] = [];
		let pressed = 0;
		let fresh = true;

		for (const press of picked.presses) {
			/*
			 * Every press is made on a frame just opened. A press that moves nothing on the glass
			 * can still change what the frame holds: `Add a role` in a pinned frame ends its hold,
			 * and the `Discard` pressed after it then read as inert when it is not.
			 */
			if (!fresh) read = await open(page, address);
			fresh = false;

			const find = () =>
				pick(read.targets, true).presses.find(other => other.key === press.key)?.index;
			const index = find();
			if (index === undefined) {
				unreached.push(press.name);
				fresh = true;
				continue;
			}

			/*
			 * A source file saved while the sweep runs reloads the frame, and the control read a
			 * moment ago is gone with it. The frame is opened once more before the press is given up.
			 */
			const handle = await handleFor(page, index).catch(async () => {
				read = await open(page, address);
				const again = find();
				return again === undefined ? null : handleFor(page, again).catch(() => null);
			});
			if (handle === null) {
				unreached.push(press.name);
				fresh = true;
				continue;
			}

			const facts = await pressOne(page, address, handle, press, errors);
			if (facts === null) {
				/* Nothing was pressed, so the frame is as it was opened. */
				unreached.push(press.name);
				fresh = true;
				continue;
			}

			pressed += 1;
			findings.push(...judgePress(facts));
		}

		const results: ProbeResult[] = [
			{
				probe: 'press',
				ok: findings.length === 0,
				note: `${String(pressed)} pressed`,
				findings,
			},
			{
				probe: 'text-fits',
				ok: text.length === 0,
				note:
					text.length === 0 ? 'every text in its box' : `${String(text.length)} wider than its box`,
				findings: text,
			},
		];

		return {
			id: entry.id,
			variant: entry.variant,
			width: viewport.width,
			widthName,
			results,
			pressed,
			alike: picked.alike,
			unreached,
		};
	} finally {
		await context.close();
	}
}

/**
 * Sets the findings the baseline holds apart from the new ones, one finding at a time.
 *
 * The probes hold a result whole or not at all, which suits a probe with one finding a frame. A
 * swept frame has a dozen, most of them a pinned state's inert controls, and one new fault among
 * them must not print the other eleven as failing again.
 */
export function hold(
	reports: FrameReport[],
	keys: readonly string[]
): { reports: FrameReport[]; held: number; resolved: string[] } {
	const known = new Set(keys);
	const seen = new Set<string>();
	let held = 0;

	const split = reports.map(report => ({
		...report,
		results: report.results.flatMap(result => {
			const old = result.findings.filter(finding => {
				const key = keyOf(report, result.probe, finding);
				seen.add(key);
				return known.has(key);
			});
			if (old.length === 0) return [result];

			held += old.length;
			const fresh = result.findings.filter(finding => !old.includes(finding));
			return [
				...(fresh.length > 0 ? [{ ...result, ok: false, findings: fresh }] : []),
				{ ...result, ok: true, findings: old },
			];
		}),
	}));

	const swept = new Set(
		reports.map(report => `${report.id}·${report.variant}·${report.widthName}`)
	);
	const resolved = keys.filter(
		key => swept.has(key.split('·').slice(0, 3).join('·')) && !seen.has(key)
	);

	return { reports: split, held, resolved };
}

/** The reports with only the findings a pattern matches: what a partial `--accept` writes down. */
export function only(reports: FrameReport[], pattern: RegExp): FrameReport[] {
	return reports.map(report => ({
		...report,
		results: report.results.map(result => {
			const findings = result.findings.filter(finding => pattern.test(finding));
			return { ...result, ok: findings.length === 0, findings };
		}),
	}));
}

/**
 * The findings, each said once with every state and width it was found in.
 *
 * One fault in a frame's chrome shows in every state at every width, and a line a frame turned two
 * places into a hundred and forty-five lines. A finding the baseline holds is printed only when
 * everything is asked for.
 */
export function grouped(
	reports: FrameReport[],
	widths: readonly string[],
	everything: boolean
): string[] {
	type Found = { held: boolean; at: [state: string, width: string][] };
	const byId = new Map<string, Map<string, Found>>();

	for (const report of reports) {
		for (const result of report.results) {
			if (result.ok && !everything) continue;

			for (const finding of result.findings) {
				const findings = byId.get(report.id) ?? new Map<string, Found>();
				const entry: Found = findings.get(finding) ?? { held: result.ok, at: [] };
				entry.at.push([report.variant, report.widthName]);
				findings.set(finding, entry);
				byId.set(report.id, findings);
			}
		}
	}

	const lines: string[] = [];
	for (const [id, findings] of byId) {
		lines.push(id);

		for (const [finding, { held, at }] of findings) {
			const states = [...new Set(at.map(([state]) => state))];
			const found = [...new Set(at.map(([, width]) => width))];
			const square = at.length === states.length * found.length;

			lines.push(`  ${held ? 'held' : 'FAIL'}  ${finding}`);
			lines.push(
				square
					? `          ${states.join(', ')} · ${found.length === widths.length && widths.length > 1 ? 'every width' : found.join(', ')}`
					: `          ${at.map(([state, width]) => `${state} at ${width}`).join(', ')}`
			);
		}
	}

	return lines;
}

/**
 * **Every control of a frame, pressed at every width, and only what went wrong printed.**
 *
 * It is `drive` asked of the whole frame at once. A reviewer used to do this by hand, one control
 * and one script at a time, and it was the slowest part of a pass that took eleven minutes at the
 * median. What it looks for needs no judgement: a press that moves nothing, a keyboard left on the
 * page body, something Escape does not close or that does not hand the keyboard back, a page that
 * scrolls sideways afterwards, a press that throws, text wider than its box, a control with no
 * name.
 *
 * What it cannot see is whether the thing that moved was the right thing. That is a reader's.
 *
 * Findings that are a design rather than a defect are written down with `--accept`, as the
 * probes' are, so the command fails only on what is new.
 */
export async function sweep(argv: Argv): Promise<number> {
	const url = flag(argv, 'url') ?? host().url;
	const theme: Appearance = flag(argv, 'theme') === 'light' ? 'light' : 'dark';
	const everything = bool(argv, 'all');
	const every = bool(argv, 'every');
	const accept = bool(argv, 'accept');
	const asked = list(argv, 'widths').filter((name): name is ViewportName =>
		VIEWPORT_NAMES.includes(name as ViewportName)
	);
	const widths = asked.length > 0 ? asked : [...VIEWPORT_NAMES];
	const frames = argv.positional.flatMap(token => token.split(',')).filter(Boolean);
	const spaces = resolveSpaces(list(argv, 'space').length > 0 ? list(argv, 'space') : ['proto']);

	if (accept && theme === 'light') {
		throw new Error('--accept rewrites the dark baseline; run it without --theme light.');
	}

	return withWorkbench(url, async browser => {
		const census = await readCensus(browser, url);
		const entries = selectFrames(census, { frames, spaces });

		if (entries.length === 0) {
			process.stdout.write(
				`Nothing to sweep in [${frames.length > 0 ? frames.join(', ') : spaces.join(', ')}]. Name a frame: workbench sweep <id>[:<state>], or a space with --space views.\n`
			);
			return 1;
		}

		const jobs = entries.flatMap(entry => widths.map(widthName => ({ entry, widthName })));
		const raw = new Array<Swept>(jobs.length);
		let next = 0;

		const worker = async (): Promise<void> => {
			for (let index = next++; index < jobs.length; index = next++) {
				const job = jobs[index];
				if (job === undefined) continue;
				raw[index] = await sweepFrame(browser, url, job.entry, job.widthName, theme, every);
			}
		};

		await Promise.all(Array.from({ length: Math.min(4, jobs.length) }, worker));

		const existing = loadBaseline(host().baselines.sweep);

		if (accept) {
			/*
			 * `--only` accepts the findings a pattern matches and leaves the rest failing. Findings
			 * already held on these frames are kept, so accepting one cause does not drop another.
			 */
			const pattern = flag(argv, 'only');
			const chosen = pattern === undefined ? raw : only(raw, new RegExp(pattern));
			const before = hold(raw, existing?.keys ?? []).reports.map(report => ({
				...report,
				results: report.results.filter(result => result.ok && result.findings.length > 0),
			}));
			const kept = [...chosen, ...before].map(report => ({
				...report,
				results: report.results.map(result => ({ ...result, ok: result.findings.length === 0 })),
			}));

			const baseline = acceptBaseline(
				kept,
				existing,
				'Sweep findings read, understood and expected to keep occurring. See PROBES.md.',
				host().baselines.sweep
			);
			process.stdout.write(
				`${grouped(chosen, widths, false).join('\n')}\n\nbaseline accepted · ${String(baseline.keys.length)} findings held · ${path.relative(process.cwd(), host().baselines.sweep)}\n`
			);
			return 0;
		}

		const { reports, held, resolved } = bool(argv, 'full')
			? { reports: raw as FrameReport[], held: 0, resolved: [] }
			: hold(raw, existing?.keys ?? []);
		const body = grouped(reports, widths, everything);
		if (body.length > 0) process.stdout.write(`${body.join('\n')}\n\n`);

		const pressed = raw.reduce((sum, report) => sum + report.pressed, 0);
		const alike = raw.reduce((sum, report) => sum + report.alike, 0);
		const unreached = raw.reduce((sum, report) => sum + report.unreached.length, 0);
		const failing = reports.flatMap(report =>
			report.results.flatMap(result => (result.ok ? [] : result.findings))
		).length;

		const asides = [
			...(alike > 0 ? [`${String(alike)} alike left out, --every presses them`] : []),
			...(unreached > 0 ? [`${String(unreached)} out of reach`] : []),
		];
		process.stdout.write(
			`${String(raw.length)} frame${raw.length === 1 ? '' : 's'} swept · ${String(pressed)} press${pressed === 1 ? '' : 'es'}${asides.length > 0 ? ` (${asides.join(' · ')})` : ''}${held > 0 ? ` · ${String(held)} held by the baseline` : ''}\n`
		);

		if (resolved.length > 0) {
			process.stdout.write(
				`\n${String(resolved.length)} held finding${resolved.length === 1 ? '' : 's'} no longer occur${resolved.length === 1 ? 's' : ''}. Re-accept to prune:\n${resolved.map(key => `  ${key}`).join('\n')}\n`
			);
		}

		process.stdout.write(
			failing > 0
				? `RESULT: ${String(failing)} FINDING${failing === 1 ? '' : 'S'}\n`
				: 'RESULT: sweep clean\n'
		);
		return failing > 0 ? 1 : 0;
	});
}
