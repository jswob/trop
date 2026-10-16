import type { Page } from '@playwright/test';

import { flag, type Argv } from './args';
import { host } from './host';
import { settle, withWorkbench } from './workbench';

type Row = string[];

function table(rows: Row[]): string {
	const widths = rows.reduce<number[]>(
		(sizes, row) => row.map((cell, index) => Math.max(sizes[index] ?? 0, cell.length)),
		[]
	);

	return rows
		.map(row =>
			row
				.map((cell, index) => cell.padEnd(widths[index] ?? 0))
				.join('  ')
				.trimEnd()
		)
		.join('\n');
}

const cells = (page: Page, row: string, cell: string): Promise<Row[]> =>
	page.$$eval(
		row,
		(rows, inner) =>
			rows.map(element =>
				[...element.querySelectorAll(inner)].map(part => (part.textContent ?? '').trim())
			),
		cell
	);

/**
 * A kit component's page, read back from the bench.
 *
 * `kit` lists the components. `kit <id>` prints one: what it is, its states, its props and how many
 * files call it. `--state <id>` shows that state alone, and `--parts` adds what the page measures
 * for each named part, at `--width` and in `--theme`.
 *
 * It asks the page rather than the source, as every measurement here does: the figures printed are
 * the ones on the screen, so a thing said about the screen can be checked from a terminal.
 */
export async function kit(argv: Argv): Promise<number> {
	const url = flag(argv, 'url') ?? host().url;
	const id = argv.positional[0];

	return withWorkbench(url, async browser => {
		const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });

		if (id === undefined) {
			await page.goto(`${url}/kit`, { waitUntil: 'load' });
			await settle(page);
			const rows = await cells(page, '.wb-list-row', ':scope > *');

			process.stdout.write(
				`${table(rows.map(([name = '', line = '', callers = '']) => [name, callers, line]))}\n`
			);
			process.stdout.write(
				`\n${String(rows.length)} components · bun workbench kit <id> for one\n`
			);
			return 0;
		}

		const state = flag(argv, 'state');
		const search = new URLSearchParams();
		if (state !== undefined) {
			search.set('variants', state);
			search.set('at', state);
		}
		if (argv.flags['parts'] !== undefined) search.set('parts', 'on');
		const width = flag(argv, 'width');
		if (width !== undefined) search.set('width', width);
		if (flag(argv, 'theme') === 'light') search.set('theme', 'light');

		const query = search.toString();
		await page.goto(`${url}/kit/${id}${query === '' ? '' : `?${query}`}`, {
			waitUntil: 'load',
		});
		await settle(page);
		await page.waitForTimeout(argv.flags['parts'] !== undefined ? 1500 : 300);

		const name = await page.textContent('.wb-name');
		const is = await page.textContent('.wb-desc');
		const read = await page.textContent('.wb-readout');
		const states = await cells(
			page,
			'.wb-flow .wb-cap, .wb-grid .wb-cap',
			'.wb-ix, .wb-capname, .wb-note'
		);
		const props = await cells(page, '.wb-prow', ':scope > *');
		const parts = await cells(page, '.wb-part:not(.wb-part-head)', ':scope > *');
		const partsOf = await page.textContent('.wb-parts .wb-h').catch(() => null);
		const callers = await page.textContent('.wb-callers').catch(() => null);

		if (props.length === 0) {
			process.stdout.write(`${name ?? id}\n${is ?? ''}\n\nNo specimen is declared for it yet.\n`);
			return 1;
		}

		process.stdout.write(`${name ?? id}\n${is ?? ''}\n${read ?? ''}\n\n`);
		process.stdout.write(`${table([['', 'state', 'why it is listed'], ...states])}\n\n`);
		if (parts.length > 0) {
			if (partsOf !== null) process.stdout.write(`${partsOf}\n`);
			process.stdout.write(
				`${table([['', 'part', 'what it is', 'box', 'padding', 'type', 'ink on its ground'], ...parts])}\n\n`
			);
		} else if (argv.flags['parts'] !== undefined) {
			process.stdout.write('It names no parts.\n\n');
		}
		process.stdout.write(`${table([['prop', 'takes', ''], ...props])}\n\n${callers ?? ''}\n`);
		process.stdout.write(`${page.url().replace(url, '')}\n`);
		return 0;
	});
}

/** The system's tokens, read back from the bench's tokens page. */
export async function tokens(argv: Argv): Promise<number> {
	const url = flag(argv, 'url') ?? host().url;

	return withWorkbench(url, async browser => {
		const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });
		await page.goto(`${url}/tokens`, { waitUntil: 'load' });
		await settle(page);

		const colours = await page.$$eval('.wb-chip', chips =>
			chips.map(chip => {
				const [name, used] = [...(chip.querySelectorAll('.wb-meta')[0]?.children ?? [])].map(
					part => part.textContent ?? ''
				);
				const [dark, light] = [...chip.querySelectorAll('input')].map(box => box.value);
				const contrast = chip.querySelectorAll('.wb-meta')[1]?.lastElementChild?.textContent ?? '';

				return [name ?? '', dark ?? '', light ?? '', contrast, used ?? ''];
			})
		);
		const rest = await page.$$eval('.wb-type, .wb-space', rows =>
			rows.map(row => [
				row.querySelector('b')?.textContent ?? '',
				row.querySelector('input')?.value ?? row.querySelector('.wb-wide')?.textContent ?? '',
				[...row.querySelectorAll('span')].find(span => /files?$/.test(span.textContent ?? ''))
					?.textContent ?? '',
			])
		);

		process.stdout.write(
			`${table([['colour', 'dark', 'light', 'on its ground', 'used by'], ...colours])}\n\n`
		);
		process.stdout.write(`${table([['token', 'value', 'used by'], ...rest])}\n\n`);
		process.stdout.write(`${String(colours.length + rest.length)} tokens\n`);
		return 0;
	});
}
