import { existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import path from 'node:path';

import { nameOf } from '../sources';
import { flag, type Argv } from './args';
import { engineName } from './engine';
import { host } from './host';
import {
	assertDrawing,
	protoTemplate,
	registerProto,
	thesisTemplate,
	unregisterProto,
} from './scaffold';

const protoIndex = (): string => path.join(host().root, 'proto/index.ts');

function protoFile(id: string): string {
	return path.join(host().root, 'proto', `${id}.tsx`);
}

/** Where a candidate's thesis numbers are written down, for `conform` to hold the frames against. */
export function thesisFile(id: string): string {
	return path.join(host().root, 'theses', `${id}.thesis.ts`);
}

function assertId(id: string | undefined): string {
	if (id === undefined || !/^[a-z][a-z0-9-]*$/.test(id)) {
		throw new Error(`name the candidate in kebab-case: workbench new split-rail --title "…"`);
	}
	return id;
}

/**
 * A candidate, scaffolded and registered.
 *
 * `--from` starts it from a board's drawing instead of the scaffold: the file the user approved at
 * step 2, copied in as it is. A drawing re-typed into a blank candidate is where a build used to
 * drift from what was approved.
 *
 * The restart note is not decoration. `proto/index.ts` enumerates its sources with an eager
 * `import.meta.glob`, which resolves at startup, so a brand-new file does not appear until the dev
 * server restarts — and `WORKBENCH.md` lists debugging its absence as a wasted half hour that has
 * actually been spent.
 */
export function newCandidate(argv: Argv): number {
	const id = assertId(argv.positional[0]);
	const title = flag(argv, 'title') ?? id;
	const thesis = flag(argv, 'thesis') ?? 'TODO — one line, and the axis it commits to.';

	const from = flag(argv, 'from');

	const file = protoFile(id);
	if (existsSync(file)) throw new Error(`proto/${id}.tsx already exists`);

	const source =
		from === undefined
			? protoTemplate(id, title, thesis, {
					app: host().name,
					theme: nameOf(host().theme.css, host()),
					engine: engineName(),
				})
			: assertDrawing(readFileSync(path.resolve(from), 'utf8'), id);

	writeFileSync(file, source, 'utf8');
	writeFileSync(protoIndex(), registerProto(readFileSync(protoIndex(), 'utf8'), id), 'utf8');

	const numbers = thesisFile(id);
	const wroteNumbers = !existsSync(numbers);
	if (wroteNumbers) {
		mkdirSync(path.dirname(numbers), { recursive: true });
		writeFileSync(numbers, thesisTemplate(id, thesis, engineName()), 'utf8');
	}

	process.stdout.write(
		`created  proto/${id}.tsx${from === undefined ? '' : `  from ${path.resolve(from)}`}\nregistered in proto/index.ts\n` +
			`${wroteNumbers ? 'created ' : 'kept    '} theses/${id}.thesis.ts\n\n` +
			`  /proto/${id}   ·   run "bun workbench frames --space proto" for its address\n\n` +
			`Write the thesis's numbers into theses/${id}.thesis.ts before anything is built:\n` +
			`  bun workbench conform ${id}   holds the frames against them.\n\n` +
			`Restart the workbench — proto/index.ts globs its sources eagerly, so a new file\n` +
			`does not appear until it does.\n`
	);
	return 0;
}

/**
 * A candidate, deleted.
 *
 * Step 9: *"Delete the losing candidates. The record is the bans file, not the files."* This makes
 * that one command instead of three edits, which is the difference between a rule that is followed
 * and a rule that is followed when there is time.
 */
export function rmCandidate(argv: Argv): number {
	const id = assertId(argv.positional[0]);
	const file = protoFile(id);

	const existed = existsSync(file);
	if (existed) rmSync(file);

	const numbers = thesisFile(id);
	const hadNumbers = existsSync(numbers);
	if (hadNumbers) rmSync(numbers);

	const before = readFileSync(protoIndex(), 'utf8');
	const after = unregisterProto(before, id);
	if (after !== before) writeFileSync(protoIndex(), after, 'utf8');

	if (!existed && after === before) throw new Error(`no candidate "${id}" — nothing to delete`);

	process.stdout.write(
		`${existed ? `deleted  proto/${id}.tsx\n` : ''}${hadNumbers ? `deleted  theses/${id}.thesis.ts\n` : ''}${after !== before ? 'unregistered from proto/index.ts\n' : ''}\n` +
			`What it taught belongs in BANS.md, not in a file:\n` +
			`  bun workbench ban add --section <n> --round <n> --title "…"\n`
	);
	return 0;
}
