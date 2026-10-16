import { existsSync, readFileSync, renameSync, writeFileSync } from 'node:fs';
import path from 'node:path';

import { flag, type Argv } from './args';
import { callersOf, exportedSymbols } from './callers';
import { host } from './host';
import { promoteToApp } from './promote-app';
import { pascalOf, unregisterProto } from './scaffold';

const kitIndex = (): string => path.join(host().root, 'kit/index.ts');
const protoIndex = (): string => path.join(host().root, 'proto/index.ts');

const EXPORT_LINE = /^export (?:type )?\{[\s\S]*?\} from '\.\/([^']+)';$/gm;

/**
 * Puts a kit export where the file already keeps them — sorted by module, case-insensitively.
 *
 * Returning the source unchanged when the module is already exported matters: a promotion that is
 * re-run must not append a second export line, and the second run is the common one because the
 * first usually stops on the caller count.
 */
export function addKitExport(
	source: string,
	module: string,
	values: string[],
	types: string[]
): string {
	if (new RegExp(`from '\\./${module}';`).test(source)) return source;

	const lines: string[] = [];
	if (values.length > 0) lines.push(`export { ${values.join(', ')} } from './${module}';`);
	if (types.length > 0) lines.push(`export type { ${types.join(', ')} } from './${module}';`);
	if (lines.length === 0) return source;

	const block = `${lines.join('\n')}\n`;
	const matches = [...source.matchAll(EXPORT_LINE)];

	const after = matches.filter(match => (match[1] ?? '').toLowerCase() < module.toLowerCase());
	const last = after[after.length - 1];

	if (last === undefined) {
		const first = matches[0];
		if (first === undefined) return `${source}${block}`;
		const at = first.index ?? 0;
		return `${source.slice(0, at)}${block}${source.slice(at)}`;
	}

	const at = (last.index ?? 0) + last[0].length + 1;
	return `${source.slice(0, at)}${block}${source.slice(at)}`;
}

/**
 * A settled prototype, moved out of the space that holds only what is unsettled.
 *
 * It does the mechanical half and **stops at the half that is design work**. It will not write the
 * specimen: a component's page owes what it is, every state, a line for every prop, its parts, what
 * it holds and what it refused, and a generated stub with those left empty is a component that
 * looks documented. Nor will it rebuild the callers, which is the other half of settling and is the
 * only thing that proves the contract.
 *
 * What it does do is print rule 8's number, read off the source, before anything moves.
 */
export function promote(argv: Argv): number {
	const to = flag(argv, 'to');
	if (to === 'app') return promoteToApp(argv);

	const id = argv.positional[0];
	if (id === undefined)
		throw new Error('name the candidate: workbench promote <id> --to kit|views');

	if (to !== 'kit' && to !== 'views') throw new Error('--to takes kit, views or app');

	const from = path.join(host().root, 'proto', `${id}.tsx`);
	if (!existsSync(from)) throw new Error(`no proto/${id}.tsx to promote`);

	const source = readFileSync(from, 'utf8');
	const { values, types } = exportedSymbols(source);
	const callers = callersOf(values, [`proto/${id}.tsx`]);

	process.stdout.write(`promoting proto/${id}.tsx → ${to}/\n\n`);
	process.stdout.write(`  exports   ${values.join(', ') || '(none)'}\n`);
	process.stdout.write(`  callers   ${callers.length === 0 ? 'none' : callers.join(', ')}\n`);

	if (callers.length < 2) {
		process.stdout.write(
			`\n  Rule 8 wants a second caller. One caller is a part of that screen, and a\n` +
				`  component with none has a contract nothing outside it has had to meet.\n` +
				`  Promoting at one is a thing the kit has done on request and the kit page\n` +
				`  prints the number rather than rounding it up — so if this is that, say so\n` +
				`  in its specimen.\n`
		);
	}

	if (flag(argv, 'dry') !== undefined || argv.flags['dry'] === true) return 0;

	const name = flag(argv, 'name') ?? (to === 'kit' ? pascalOf(id) : id);
	const target = path.join(host().root, to, `${name}.tsx`);
	if (existsSync(target)) throw new Error(`${to}/${name}.tsx already exists`);

	renameSync(from, target);
	writeFileSync(protoIndex(), unregisterProto(readFileSync(protoIndex(), 'utf8'), id), 'utf8');

	process.stdout.write(`\nmoved     ${to}/${name}.tsx\nunregistered from proto/index.ts\n`);

	if (to === 'kit') {
		const before = readFileSync(kitIndex(), 'utf8');
		const after = addKitExport(before, name, values, types);
		if (after !== before) {
			writeFileSync(kitIndex(), after, 'utf8');
			process.stdout.write(`exported  kit/index.ts\n`);
		}
	}

	const page =
		to === 'kit'
			? `  2  write its specimen beside it, kit/${name}.specimen.tsx, and list it in\n` +
				`     kit-docs/index.ts: what it is, every state, a line for every prop, its parts,\n` +
				`     what it holds and what it refused\n`
			: `  2  list it as a place in views/index.ts, so the bench can open it\n`;

	process.stdout.write(
		`\nThe rest is the round's, not the command's:\n` +
			`  1  rebuild the callers on it — a promoted thing with nothing drawn in it has not\n` +
			`     been tested, and the contract is only real once something outside it has met it\n` +
			`${page}` +
			`  3  delete the losing candidates — the record is BANS.md, not the files\n` +
			`  4  fold the round's bans:  bun workbench ban add --section <n> --round <n>\n` +
			`  5  bun workbench check\n`
	);

	return 0;
}
