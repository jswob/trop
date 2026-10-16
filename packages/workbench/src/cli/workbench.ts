import { spawn, type ChildProcess } from 'node:child_process';
import path from 'node:path';
import process from 'node:process';

import { chromium, type Browser, type Page } from '@playwright/test';

import { CENSUS_KEY, type FrameEntry } from '../lib/frame-census';
import { host } from './host';

/**
 * Getting a browser onto a frame — the part every node-side tool here needs and none of them
 * should own a second copy of.
 *
 * `probe/run.ts` had all of this inline first. It moved when `drive` needed the same four things
 * (is it up, start it if not, wait for paint, read the census off `window`), because two copies of
 * "wait for the page to be there" is how one of them quietly stops waiting long enough.
 */

/** Whether the workbench is already up. A round usually has it running; a CI job does not. */
export async function reachable(url: string): Promise<boolean> {
	try {
		const response = await fetch(url, { signal: AbortSignal.timeout(1500) });
		return response.ok;
	} catch {
		return false;
	}
}

/** Starts the workbench the way `serve` does, as a process of its own, and waits for it to answer. */
export async function startWorkbench(url: string): Promise<ChildProcess> {
	const child = spawn(
		process.execPath,
		[path.join(import.meta.dirname, 'main.ts'), '--config', host().file, 'serve'],
		{ cwd: host().app.root, stdio: 'ignore' }
	);

	const deadline = Date.now() + 60_000;
	while (Date.now() < deadline) {
		if (await reachable(url)) return child;
		await new Promise(resolve => setTimeout(resolve, 400));
	}

	child.kill();
	throw new Error(`the workbench did not come up at ${url} within 60s`);
}

/**
 * Let the page paint before measuring it.
 *
 * A measurement taken in the same batch as the navigation before it reads a frame that is not
 * there yet — the same trap that makes a screenshot come back black with the DOM fully rendered
 * underneath. It is the capture that is early, not the page that is broken.
 */
export async function settle(page: Page): Promise<void> {
	await page.waitForFunction(
		() => (document.getElementById('root')?.childElementCount ?? 0) > 0,
		undefined,
		{ timeout: 15_000 }
	);
	await page.evaluate(
		() =>
			new Promise(resolve =>
				requestAnimationFrame(() => requestAnimationFrame(() => resolve(null)))
			)
	);
}

/** Every frame-able variant, read off the workbench's own `window` rather than re-derived. */
export async function readCensus(browser: Browser, url: string): Promise<FrameEntry[]> {
	const page = await browser.newPage();
	try {
		await page.goto(url, { waitUntil: 'load', timeout: 30_000 });
		return (await page.evaluate(key => window[key as typeof CENSUS_KEY], CENSUS_KEY)) ?? [];
	} finally {
		await page.close();
	}
}

/**
 * Runs `body` against a browser and a live workbench, starting one only if it has to and killing
 * only what it started. A workbench the round already has open is never taken down under it.
 */
export async function withWorkbench<T>(
	url: string,
	body: (browser: Browser) => Promise<T>
): Promise<T> {
	let spawned: ChildProcess | undefined;
	if (!(await reachable(url))) {
		process.stdout.write(`workbench not up — starting it at ${url}\n`);
		spawned = await startWorkbench(url);
	}

	const browser = await chromium.launch();
	try {
		return await body(browser);
	} finally {
		await browser.close();
		spawned?.kill();
	}
}
