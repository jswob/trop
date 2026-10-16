import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';

import type { Argv } from '../cli/args';
import { host } from '../cli/host';
import type { ResolvedConfig } from '../config';
import { designSources } from '../sources';
import { formatCensus, conflicts, stateCensus, unionMembers } from './state-census';

/**
 * Every state the system has and every hue each file paints it in, read off the source.
 *
 * The design root's kit, views and proto, and the settled kit and views that moved out of them: a
 * state painted from a settled file and from a design root file is still painted from two places.
 * `null` when the app's config names no census, so there is nothing to read.
 */
export function censusReport(config: ResolvedConfig): { text: string; ok: boolean } | null {
	if (config.census === undefined) return null;

	const { states: declaredIn, type, hues } = config.census;
	const files = designSources(config, ['kit', 'views', 'proto']).filter(
		([file]) => !file.includes('.test.') && path.basename(file) !== 'index.ts'
	);

	/* The union wherever it lives now: the design root until it settles, the app after. */
	const typesFile = declaredIn.find(file => existsSync(file));
	const states = unionMembers(typesFile === undefined ? '' : readFileSync(typesFile, 'utf8'), type);

	if (states.length === 0) {
		return { text: `No ${type} union found in kit/types.ts — nothing to census.`, ok: true };
	}

	const census = stateCensus(files, states, hues);
	return { text: formatCensus(census), ok: conflicts(census).length === 0 };
}

/** `census` — prints the state census and fails on a state painted in two hues. */
export function census(_argv: Argv): number {
	const report = censusReport(host());
	if (report === null) {
		process.stdout.write('No census in the workbench config — nothing to read.\n');
		return 0;
	}

	process.stdout.write(`${report.text}\n`);
	return report.ok ? 0 : 1;
}
