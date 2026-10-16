import { existsSync, readFileSync, writeFileSync } from 'node:fs';

import type { FrameReport } from './report';

export type Baseline = {
	accepted: string;
	note: string;
	/** One key per finding that is known, understood and expected to keep occurring. */
	keys: string[];
};

/**
 * **A probe that cries wolf is one nobody runs, which is the same as not having it.**
 *
 * `PROBES.md` says that about the four filters inside the runner, and then the runner grew a
 * fourteenth failing frame that is not a defect. A round settled a floating slug at the foot of
 * the column; the hit-test probe is *right* that it covers whatever is under it at that scroll
 * position, and the file says so in as many words — *"a floating object covering column content is
 * a reading, not a finding"*. So the views space exits non-zero on a clean tree, prints 35 lines
 * that have already been adjudicated, and every run needs a person to re-read them and conclude
 * nothing.
 *
 * A baseline is the written form of that adjudication. The number to watch, as `PROBES.md` puts
 * it, *"is whether it grew"* — so that is what the exit code tracks. Findings held here still
 * print; they simply stop failing the command, and a held finding that stops occurring is reported
 * too, because a baseline nobody prunes is a second thing that quietly stops being true.
 */
export function keyOf(
	report: Pick<FrameReport, 'id' | 'variant' | 'widthName'>,
	probe: string,
	finding: string
): string {
	return `${report.id}·${report.variant}·${report.widthName}·${probe}·${finding}`;
}

export function keysOf(reports: FrameReport[]): string[] {
	return reports.flatMap(report =>
		report.results.flatMap(result =>
			result.ok ? [] : result.findings.map(finding => keyOf(report, result.probe, finding))
		)
	);
}

/** Which frame·state·width a key belongs to, so a partial run never calls the rest resolved. */
function scopeOfKey(key: string): string {
	return key.split('·').slice(0, 3).join('·');
}

export type Held = {
	/** The reports, with findings the baseline holds moved out of the exit code. */
	reports: FrameReport[];
	held: number;
	/** Keys the baseline holds that no longer occur on a frame this run actually probed. */
	resolved: string[];
};

export function applyBaseline(reports: FrameReport[], baseline: Baseline | null): Held {
	if (baseline === null) return { reports, held: 0, resolved: [] };

	const known = new Set(baseline.keys);
	let held = 0;

	const annotated = reports.map(report => ({
		...report,
		results: report.results.map(result => {
			if (result.ok || result.findings.length === 0) return result;

			const keys = result.findings.map(finding => keyOf(report, result.probe, finding));
			if (!keys.every(key => known.has(key))) return result;

			held += keys.length;
			return { ...result, ok: true, note: `${result.note} · held by the baseline` };
		}),
	}));

	const probed = new Set(
		reports.map(report => `${report.id}·${report.variant}·${report.widthName}`)
	);
	const seen = new Set(keysOf(reports));
	const resolved = baseline.keys.filter(key => probed.has(scopeOfKey(key)) && !seen.has(key));

	return { reports: annotated, held, resolved };
}

export function loadBaseline(file: string): Baseline | null {
	if (!existsSync(file)) return null;
	return JSON.parse(readFileSync(file, 'utf8')) as Baseline;
}

/**
 * Writes the findings of the frames just probed, keeping the rest of the baseline intact.
 *
 * Accepting a subset must never silently drop what a narrower run did not look at — that would
 * turn `--frames one:state --accept` into a way to erase everything else's adjudication.
 */
export function acceptBaseline(
	reports: FrameReport[],
	existing: Baseline | null,
	note: string,
	file: string
): Baseline {
	const probed = new Set(
		reports.map(report => `${report.id}·${report.variant}·${report.widthName}`)
	);
	const kept = (existing?.keys ?? []).filter(key => !probed.has(scopeOfKey(key)));

	const baseline: Baseline = {
		accepted: new Date().toISOString().slice(0, 10),
		note,
		keys: [...new Set([...kept, ...keysOf(reports)])].sort(),
	};

	writeFileSync(file, `${JSON.stringify(baseline, null, 2)}\n`, 'utf8');
	return baseline;
}
