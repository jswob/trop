import { createHash } from 'node:crypto';
import {
	cpSync,
	existsSync,
	mkdirSync,
	mkdtempSync,
	readFileSync,
	readdirSync,
	rmSync,
	writeFileSync,
} from 'node:fs';
import os from 'node:os';
import path from 'node:path';

import type { Browser, Route } from '@playwright/test';

import type { Appearance } from '../lib/appearance';

import { frameUrl } from '../lib/frame-url';
import { VIEWPORT_NAMES, VIEWPORTS, type ViewportName } from '../lib/viewports';
import { bool, flag, list, num, type Argv } from './args';
import { host } from './host';
import { selectFrames } from './select';
import {
	answerName,
	APPEARANCES,
	assertShotsDir,
	compareSets,
	diffLines,
	packAnswer,
	REMOTE_DIR,
	shotName,
	shotsNamed,
	shotsOf,
	steadied,
	unpackAnswer,
	type Answer,
	type Shot,
} from './shot-files';
import { resolveSpaces } from './spaces';
import { readCensus, settle, withWorkbench } from './workbench';

/** A set as it lies in a folder: each picture's name and the digest of its bytes. */
function readSet(dir: string): Map<string, string> {
	return new Map(
		readdirSync(dir)
			.filter(name => name.endsWith('.png'))
			.sort()
			.map(name => [
				name,
				createHash('sha256')
					.update(readFileSync(path.join(dir, name)))
					.digest('hex'),
			])
	);
}

/**
 * Answers every request a frame makes to another host, from the set's folder.
 *
 * The first time an address is asked for, the host answers and the answer is kept; every later
 * frame, in this set and in one taken against it, is given the same bytes. `live` collects the
 * addresses the host had to be asked for.
 */
function remoteAnswers(dir: string, live: Set<string>): (route: Route) => Promise<void> {
	const folder = path.join(dir, REMOTE_DIR);
	const asked = new Map<string, Promise<Answer>>();
	mkdirSync(folder, { recursive: true });

	const answerFor = (route: Route): Promise<Answer> => {
		const url = route.request().url();
		const file = path.join(folder, answerName(url));
		const known = asked.get(url);
		if (known !== undefined) return known;

		const answer = existsSync(file)
			? Promise.resolve(unpackAnswer(readFileSync(file, 'utf8')))
			: route.fetch().then(async response => {
					const fetched = {
						url,
						status: response.status(),
						headers: response.headers(),
						body: await response.body(),
					};
					live.add(url);
					writeFileSync(file, packAnswer(fetched), 'utf8');
					return unpackAnswer(packAnswer(fetched));
				});

		asked.set(url, answer);
		return answer;
	};

	return async route => {
		try {
			const { status, headers, body } = await answerFor(route);
			await route.fulfill({ status, headers, body });
		} catch {
			await route.abort();
		}
	};
}

/** A shot that could not be taken, and why. */
type Failure = { shot: Shot; reason: string };

/** How many more times a shot that differs from the earlier set is taken before it is called different. */
const RETAKES = 2;

/**
 * Takes every shot into `dir`, a few at a time, and returns the ones that could not be taken.
 *
 * Each is the whole page of the frame with its animations at their end state and its faces loaded,
 * so two runs over the same code write the same bytes.
 */
async function take(
	browser: Browser,
	url: string,
	shots: Shot[],
	dir: string,
	workers: number,
	live: Set<string>
): Promise<Failure[]> {
	const failed: Failure[] = [];
	const origin = new URL(url).origin;
	const answer = remoteAnswers(dir, live);
	let next = 0;

	const worker = async (): Promise<void> => {
		for (let index = next++; index < shots.length; index = next++) {
			const shot = shots[index];
			if (shot === undefined) continue;

			const viewport = VIEWPORTS[shot.width];
			const context = await browser.newContext({
				viewport: { width: viewport.width, height: viewport.height },
			});

			try {
				await context.route(
					asked => /^https?:$/.test(asked.protocol) && asked.origin !== origin,
					answer
				);
				const page = await context.newPage();
				const { entry } = shot;
				const address = frameUrl(
					entry.kind,
					entry.id,
					entry.variant,
					entry.chrome,
					{ rate: 0 },
					shot.theme
				);
				await page.goto(`${url}${address}`, { waitUntil: 'load', timeout: 30_000 });
				await settle(page);
				await page.evaluate(() => document.fonts.ready.then(() => null));
				await page.screenshot({
					path: path.join(dir, shotName(shot)),
					fullPage: true,
					animations: 'disabled',
				});
			} catch (error) {
				const reason = error instanceof Error ? error.message : String(error);
				failed.push({ shot, reason: reason.split('\n')[0] ?? '' });
			} finally {
				await context.close();
			}
		}
	};

	await Promise.all(Array.from({ length: Math.min(workers, shots.length) }, worker));
	return failed.sort((a, b) => shotName(a.shot).localeCompare(shotName(b.shot)));
}

/**
 * **A picture of every frame, and what changed since the last set of them.**
 *
 * `shots <folder>` writes one PNG of every frame at every width in both appearances, with motion
 * off. `shots --against <folder>` takes them again and names every file that is not the same,
 * byte for byte. It is the oracle for a step that must change nothing on screen, such as moving
 * the machinery or the kit to another package: the probes measure a floor and the sweep presses
 * controls, and neither would notice a stylesheet that stopped loading in one appearance.
 *
 * A run pointed at part of the set (`--space`, `--frames`, `--widths`, `--theme`) compares only
 * what it took. With `--against` and no folder of its own, the new set goes to a temporary folder
 * that is kept only when something differs.
 *
 * A set keeps what other hosts answered while it was taken, and a set taken against it starts from
 * those answers, so the two never differ by what a font host chose to send that minute. An address
 * the earlier set never asked for is fetched and named.
 *
 * A shot that could not be taken is tried once more, alone. A shot that differs from the earlier
 * set is taken again, alone, and is called different only when no take is the same: one that
 * matched later is printed as `unsteady` and fails nothing.
 */
export async function shots(argv: Argv): Promise<number> {
	const url = flag(argv, 'url') ?? host().url;
	const spaces = resolveSpaces(list(argv, 'space'));
	const wanted = list(argv, 'frames');
	const asked = list(argv, 'widths').filter((name): name is ViewportName =>
		VIEWPORT_NAMES.includes(name as ViewportName)
	);
	const widths = asked.length > 0 ? asked : [...VIEWPORT_NAMES];
	const one = flag(argv, 'theme');
	const themes: readonly Appearance[] =
		one === undefined ? APPEARANCES : [one === 'light' ? 'light' : 'dark'];
	const whole =
		spaces.length === 0 && wanted.length === 0 && asked.length === 0 && one === undefined;

	const against = flag(argv, 'against');
	if (bool(argv, 'against') && against === undefined) {
		throw new Error('--against takes the folder of the earlier set');
	}
	const before = against === undefined ? undefined : path.resolve(against);
	if (before !== undefined && !existsSync(before)) throw new Error(`no folder at ${before}`);

	const own = argv.positional[0];
	const scratch = before !== undefined && own === undefined;
	const dir = scratch
		? mkdtempSync(path.join(os.tmpdir(), 'shots-'))
		: assertShotsDir(own, host().repo);

	mkdirSync(dir, { recursive: true });
	if (readdirSync(dir).some(name => name.endsWith('.png'))) {
		throw new Error(`${dir} already holds pictures — name an empty folder`);
	}
	if (before !== undefined && existsSync(path.join(before, REMOTE_DIR))) {
		cpSync(path.join(before, REMOTE_DIR), path.join(dir, REMOTE_DIR), { recursive: true });
	}

	return withWorkbench(url, async browser => {
		const census = await readCensus(browser, url);
		const chosen = selectFrames(census, { frames: wanted, spaces });
		if (chosen.length === 0) {
			process.stdout.write('No frames to take.\n');
			return 1;
		}

		const set = shotsOf(chosen, widths, themes);
		const started = Date.now();
		const live = new Set<string>();
		const stalled = await take(browser, url, set, dir, num(argv, 'workers', 4), live);
		const failed = (
			await take(
				browser,
				url,
				stalled.map(failure => failure.shot),
				dir,
				1,
				live
			)
		).map(failure => `${shotName(failure.shot)}: ${failure.reason}`);
		const seconds = ((Date.now() - started) / 1000).toFixed(1);

		process.stdout.write(
			`  ${String(set.length - failed.length)} of ${String(set.length)} shots in ${seconds}s → ${dir}\n`
		);
		for (const line of failed) process.stdout.write(`  FAILED   ${line}\n`);
		if (before !== undefined) {
			for (const address of [...live].sort()) {
				process.stdout.write(`  fetched  ${address} (the earlier set never asked for it)\n`);
			}
		}

		if (before === undefined) {
			process.stdout.write(
				failed.length === 0
					? 'RESULT: set taken\n'
					: `RESULT: ${String(failed.length)} shot${failed.length === 1 ? '' : 's'} NOT TAKEN\n`
			);
			return failed.length === 0 ? 0 : 1;
		}

		const earlier = readSet(before);
		const first = compareSets(earlier, readSet(dir), { whole });
		let diff = first;
		for (let retake = 0; retake < RETAKES && diff.differs.length > 0; retake += 1) {
			await take(browser, url, shotsNamed(set, diff.differs), dir, 1, live);
			diff = compareSets(earlier, readSet(dir), { whole });
		}

		const changed = diff.differs.length + diff.missing.length + diff.added.length;
		process.stdout.write(`\n${diffLines(diff, steadied(first, diff)).join('\n')}\n`);

		if (changed === 0 && failed.length === 0) {
			if (scratch) rmSync(dir, { recursive: true, force: true });
			process.stdout.write(`RESULT: no difference from ${before}\n`);
			return 0;
		}

		process.stdout.write(
			`RESULT: ${String(changed)} shot${changed === 1 ? '' : 's'} NOT THE SAME as ${before}` +
				`${failed.length > 0 ? `, ${String(failed.length)} not taken` : ''} — the new set is kept at ${dir}\n`
		);
		return 1;
	});
}
