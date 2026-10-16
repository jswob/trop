import { frameUrl } from '../lib/frame-url';
import { flag, list, type Argv } from './args';
import { host } from './host';
import { selectFrames } from './select';
import { resolveSpaces } from './spaces';
import { readCensus, withWorkbench } from './workbench';

function pad(value: string, width: number): string {
	return value.length >= width ? value : value + ' '.repeat(width - value.length);
}

/**
 * Every frame-able state with the address that opens it.
 *
 * It exists because an address built by hand is an address that can be wrong, and a wrong one
 * renders a frame that is not the one being reasoned about — which looks exactly like a frame that
 * did not change. `WORKBENCH.md` already says to build them with `frameUrl()` and never by hand;
 * this is `frameUrl()` with a terminal in front of it.
 */
export async function frames(argv: Argv): Promise<number> {
	const url = flag(argv, 'url') ?? host().url;
	const spaces = resolveSpaces(list(argv, 'space'));
	const wanted = list(argv, 'frames');

	const census = await withWorkbench(url, browser => readCensus(browser, url));
	const chosen = selectFrames(census, { frames: wanted, spaces });

	if (chosen.length === 0) {
		process.stdout.write(
			`No frames in [${wanted.length > 0 ? wanted.join(', ') : spaces.join(', ') || 'any space'}].\n`
		);
		return 1;
	}

	if (argv.flags['json'] !== undefined) {
		process.stdout.write(`${JSON.stringify(chosen, null, 2)}\n`);
		return 0;
	}

	const rows = chosen.map(entry => ({
		id: entry.id,
		variant: entry.variant,
		kind: entry.kind,
		address: frameUrl(entry.kind, entry.id, entry.variant, entry.chrome),
	}));

	const idWidth = Math.max(...rows.map(row => row.id.length), 2);
	const variantWidth = Math.max(...rows.map(row => row.variant.length), 5);

	process.stdout.write(`${pad('id', idWidth)}  ${pad('state', variantWidth)}  space    address\n`);
	for (const row of rows) {
		process.stdout.write(
			`${pad(row.id, idWidth)}  ${pad(row.variant, variantWidth)}  ${pad(row.kind, 7)}  ${row.address}\n`
		);
	}

	const ids = new Set(rows.map(row => row.id)).size;
	process.stdout.write(`\n${ids} frame${ids === 1 ? '' : 's'} · ${rows.length} states\n`);
	return 0;
}
