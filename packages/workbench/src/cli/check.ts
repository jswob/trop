import { spawnSync } from 'node:child_process';
import path from 'node:path';
import process from 'node:process';

import { censusReport } from '../probe/census';
import { bool, list, type Argv } from './args';
import { host } from './host';
import { SKILL_LOCK, SKILL_SRC, skillStaleness } from './skill';

type Gate = { name: string; detail: string; ok: boolean; seconds: number };

function run(name: string, cwd: string, command: string, args: string[]): Gate {
	const started = Date.now();
	const result = spawnSync(command, args, { cwd, encoding: 'utf8' });
	const seconds = (Date.now() - started) / 1000;
	const output = `${result.stdout ?? ''}${result.stderr ?? ''}`;

	return {
		name,
		ok: result.status === 0,
		seconds,
		detail: output.trim().split('\n').slice(-40).join('\n'),
	};
}

/** The state census as a gate, read in this process. Left out when the config names none. */
function censusGate(): Gate[] {
	const started = Date.now();
	const report = censusReport(host());
	if (report === null) return [];

	return [
		{
			name: 'census',
			ok: report.ok,
			seconds: (Date.now() - started) / 1000,
			detail: report.text.trim().split('\n').slice(-40).join('\n'),
		},
	];
}

/**
 * The repo's copy of the skill as a gate: it has to be the engine's skill as it stands. Left out
 * when the config names no copy.
 */
function skillGate(): Gate[] {
	const reasons = skillStaleness();
	if (reasons === null) return [];

	return [
		{
			name: 'skill',
			ok: reasons.length === 0,
			seconds: 0,
			detail:
				reasons.length === 0
					? ''
					: `The skill's copy is stale:\n${reasons.map(reason => `  ${reason}`).join('\n')}\nMake the change in the engine's skill/ folder, then: bun workbench skill (or bun toolkit sync)`,
		},
	];
}

/**
 * Where a round may write, relative to the repository: the design root, the settled spaces and
 * stylesheets, the skill when the round is working *on the skill* (its source in the engine, its
 * copy in the repo and the toolkit's lock), and whatever was named with `--allow`.
 */
function allowedPaths(argv: Argv): string[] {
	const { repo, root, settled, skill } = host();
	const within = (file: string): string => path.relative(repo, file).split(path.sep).join('/');

	return [
		`${within(root)}/`,
		...settled.spaces.map(space => `${within(path.join(settled.root, space))}/`),
		...settled.styles.map(style => within(path.join(settled.root, style))),
		...(bool(argv, 'skill') && skill !== undefined
			? [`${within(skill)}/`, `${within(SKILL_SRC)}/`, SKILL_LOCK]
			: []),
		...list(argv, 'allow'),
	];
}

/**
 * The app's gates, the census and the blast radius, scoped to the design root.
 *
 * A project's own lint and test commands answer for the whole repository, and the inner loop of a
 * round pays for that over and over to learn about one folder. The config names the same checks
 * narrowed to the design root and the settled spaces, and this runs them.
 *
 * **This does not replace the close-the-session gate.** The skill ends a round on the project's own
 * lint and test commands, and it should: a design change can break a shared type, and only the
 * whole-repo run would know. This is the fast one for while the round is still moving.
 */
export function check(argv: Argv): number {
	const gates: Gate[] = [
		...host().gates.map(gate => run(gate.name, gate.cwd, gate.run[0] ?? '', gate.run.slice(1))),
		...censusGate(),
		...skillGate(),
	];

	const allowed = allowedPaths(argv);

	const status = spawnSync('git', ['status', '--porcelain'], {
		cwd: host().repo,
		encoding: 'utf8',
	});
	const outside = (status.stdout ?? '')
		.split('\n')
		.map(line => line.slice(3).trim())
		.filter(Boolean)
		.filter(file => !allowed.some(prefix => file.startsWith(prefix)));

	gates.push({
		name: 'blast',
		ok: outside.length === 0,
		seconds: 0,
		detail:
			outside.length === 0
				? ''
				: `Rule 0: never write outside the design root. Changed outside ${allowed.join(', ')}:\n${outside.map(file => `  ${file}`).join('\n')}`,
	});

	const total = gates.reduce((sum, gate) => sum + gate.seconds, 0);

	for (const gate of gates) {
		const mark = gate.ok ? 'ok  ' : 'FAIL';
		const timing = gate.seconds > 0 ? `${gate.seconds.toFixed(1)}s` : '';
		process.stdout.write(`  ${mark}  ${gate.name.padEnd(8)} ${timing.padStart(6)}\n`);
		if (!gate.ok && gate.detail !== '') {
			process.stdout.write(
				`${gate.detail
					.split('\n')
					.map(line => `        ${line}`)
					.join('\n')}\n`
			);
		}
	}

	const failed = gates.filter(gate => !gate.ok);
	process.stdout.write(
		`\n  ${total.toFixed(1)}s · ${failed.length === 0 ? 'clean' : `${failed.length} failing: ${failed.map(gate => gate.name).join(', ')}`}\n`
	);

	if (failed.length === 0) {
		process.stdout.write(
			`\n  Before calling a round settled, run the project's own lint and tests too — this is\n` +
				`  the design root only, and a shared type breaks somewhere this never looks.\n`
		);
	}

	return failed.length === 0 ? 0 : 1;
}
