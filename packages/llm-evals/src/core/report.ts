// The verdict on a run: what each variant scored, how sure that is at this sample size, and how
// the variants compare case by case. promptfoo's viewer shows the answers; these are the numbers
// it cannot work out, because it counts rows one column at a time.
import { integrityLine, type Integrity } from './integrity.ts';
import type { RecordRow, RunRecord } from './record.ts';
import { outcomesIn } from './scorer.ts';
import { median, pairedDifference, wilson, type Difference } from './stats.ts';
import type { Outcome } from './task.ts';

export interface VariantSummary {
	variant: string;
	/** scored cases; technical errors are left out, never counted as wrong */
	cases: number;
	/** mean share of samples per case */
	correct: number;
	fellBack: number;
	wrong: number;
	invalid: number;
	/** cases with at least one wrong sample, and the 95% interval of that share */
	everWrong: { cases: number; low: number; high: number };
	/** cases where every sample gave the same answer */
	allAgree: number;
	/** the task's own step scores: mean over the cases that report them */
	steps: Record<string, { mean: number; cases: number }>;
	cost: number;
	medianMs: number;
}

export interface CaseDifference {
	caseId: string;
	/** what the key wants, from the scorer's reason */
	want: string;
	base: { outcomes: Outcome[]; shown: string[] };
	other: { outcomes: Outcome[]; shown: string[] };
	/** other minus base, in share of correct samples */
	delta: number;
}

export interface Comparison {
	base: string;
	other: string;
	/** cases both variants scored */
	cases: number;
	/** other minus base, share of correct samples per case */
	difference: Difference;
	otherBetter: number;
	baseBetter: number;
	differing: CaseDifference[];
}

export interface Report {
	evalId: string;
	description: string;
	tags: Record<string, string>;
	integrity: Integrity;
	variants: VariantSummary[];
	comparisons: Comparison[];
}

const CORE_SCORES = new Set([
	'correct',
	'fell_back',
	'wrong',
	'invalid',
	'ever_wrong',
	'all_agree',
]);
const mean = (values: number[]) =>
	values.length ? values.reduce((a, b) => a + b, 0) / values.length : 0;

function summarise(variant: string, rows: RecordRow[]): VariantSummary {
	const scored = rows.filter(r => r.state !== 'error' && 'correct' in r.scores);
	const of = (name: string) => mean(scored.map(r => r.scores[name] ?? 0));
	const everWrong = scored.filter(r => r.scores.ever_wrong === 1).length;
	const [low, high] = wilson(everWrong, scored.length);
	const steps: VariantSummary['steps'] = {};
	for (const name of new Set(scored.flatMap(r => Object.keys(r.scores)))) {
		if (CORE_SCORES.has(name) || name.endsWith('_pct')) continue;
		const reported = scored.filter(r => name in r.scores);
		steps[name] = { mean: mean(reported.map(r => r.scores[name] ?? 0)), cases: reported.length };
	}
	return {
		variant,
		cases: scored.length,
		correct: of('correct'),
		fellBack: of('fell_back'),
		wrong: of('wrong'),
		invalid: of('invalid'),
		everWrong: { cases: everWrong, low, high },
		allAgree: scored.filter(r => r.scores.all_agree === 1).length,
		steps,
		cost: rows.reduce((sum, r) => sum + r.cost, 0),
		medianMs: median(scored.map(r => r.latencyMs / Math.max(1, r.samples.length))),
	};
}

const side = (row: RecordRow) => ({
	outcomes: outcomesIn(row.reason),
	shown: row.samples.map(s => s.shown),
});

function compare(base: string, other: string, rows: RecordRow[]): Comparison {
	const scoredBy = (variant: string) =>
		new Map(
			rows
				.filter(r => r.variant === variant && r.state !== 'error' && 'correct' in r.scores)
				.map(r => [r.caseId, r])
		);
	const baseRows = scoredBy(base);
	const otherRows = scoredBy(other);
	const differences: number[] = [];
	const differing: CaseDifference[] = [];
	for (const [caseId, a] of baseRows) {
		const b = otherRows.get(caseId);
		if (!b) continue;
		const delta = (b.scores.correct ?? 0) - (a.scores.correct ?? 0);
		differences.push(delta);
		if (delta !== 0) {
			const want = /key says (.*)$/m.exec(a.reason)?.[1] ?? '';
			differing.push({ caseId, want, base: side(a), other: side(b), delta });
		}
	}
	differing.sort((x, y) => x.delta - y.delta || x.caseId.localeCompare(y.caseId));
	return {
		base,
		other,
		cases: differences.length,
		difference: pairedDifference(differences),
		otherBetter: differing.filter(d => d.delta > 0).length,
		baseBetter: differing.filter(d => d.delta < 0).length,
		differing,
	};
}

/** `vs` is the column the others are compared with; the first column (the champion) by default. */
export function buildReport(record: RunRecord, integrity: Integrity, vs?: string): Report {
	const base = vs ?? record.variants[0];
	if (base !== undefined && !record.variants.includes(base)) {
		throw new Error(`No column "${base}" in this run. Columns: ${record.variants.join(', ')}`);
	}
	return {
		evalId: record.evalId,
		description: record.description,
		tags: record.tags,
		integrity,
		variants: record.variants.map(v =>
			summarise(
				v,
				record.rows.filter(r => r.variant === v)
			)
		),
		comparisons:
			base === undefined
				? []
				: record.variants.filter(v => v !== base).map(v => compare(base, v, record.rows)),
	};
}

const pct = (share: number, digits = 1) => `${(share * 100).toFixed(digits)}%`;
const points = (share: number) => `${share >= 0 ? '+' : '−'}${Math.abs(share * 100).toFixed(1)}`;
const MARK: Record<Outcome, string> = { correct: '✓', fell_back: '↩', wrong: '✕', invalid: '⚠' };
const LEGEND =
	'One mark per sample: ✓ correct · ↩ fell back (the safe answer where the key wanted more) · ✕ wrong · ⚠ invalid. Different answers are separated by |.';

function table(rows: string[][]): string {
	const widths = (rows[0] ?? []).map((_cell, i) => Math.max(...rows.map(r => (r[i] ?? '').length)));
	return rows
		.map(r =>
			r
				.map((cell, i) => cell.padEnd(widths[i] ?? 0))
				.join('  ')
				.trimEnd()
		)
		.join('\n');
}

/** Whether a difference can be told from zero at this sample size, in plain words. */
export function verdictOf(difference: Difference): string {
	if (difference.low > 0 || difference.high < 0) return 'a real difference at this sample size';
	return 'too small to tell apart at this sample size';
}

export function renderReport(report: Report): string {
	const out: string[] = [];
	const tags = Object.entries(report.tags)
		.map(([k, v]) => `${k} ${v}`)
		.join(' · ');
	out.push(report.description, tags, `Integrity: ${integrityLine(report.integrity)}`, '');
	out.push(
		table([
			[
				'variant',
				'cases',
				'correct',
				'fell back',
				'wrong',
				'cases ever wrong (95%)',
				'all agree',
				'answers cost',
				'per answer',
			],
			...report.variants.map(v => [
				v.variant,
				String(v.cases),
				pct(v.correct),
				pct(v.fellBack),
				pct(v.wrong),
				`${v.everWrong.cases} (${pct(v.everWrong.low)}–${pct(v.everWrong.high)})`,
				String(v.allAgree),
				`$${v.cost.toFixed(4)}`,
				`${(v.medianMs / 1000).toFixed(1)} s`,
			]),
		])
	);
	const steps = report.variants.flatMap(v =>
		Object.entries(v.steps).map(
			([name, s]) => `${v.variant}: ${name} ${pct(s.mean)} of ${s.cases} cases`
		)
	);
	if (steps.length)
		out.push('', 'Step scores (each over the cases where it can happen):', ...steps);
	for (const c of report.comparisons) {
		const d = c.difference;
		out.push(
			'',
			`${c.other}  vs  ${c.base}`,
			`${points(d.mean)} points correct (95% interval ${points(d.low)} to ${points(d.high)}): ${verdictOf(d)}.`,
			`They differ on ${c.differing.length} of ${c.cases} cases: ${c.other} better on ${c.otherBetter}, worse on ${c.baseBetter}.`
		);
		if (c.differing.length) {
			const marks = (s: CaseDifference['base']) =>
				`${s.outcomes.map(o => MARK[o]).join('')} ${[...new Set(s.shown)].join(' | ')}`;
			out.push(
				table([
					['case', 'key says', c.base, c.other],
					...c.differing.map(x => [x.caseId, x.want, marks(x.base), marks(x.other)]),
				]),
				LEGEND
			);
		}
	}
	return out.join('\n');
}

/** For a run with no answer key behind it (`try`): what each variant answered, case by case. */
export function renderAnswers(record: RunRecord): string {
	const cases = [...new Set(record.rows.map(r => r.caseId))];
	const cell = (caseId: string, variant: string) => {
		const row = record.rows.find(r => r.caseId === caseId && r.variant === variant);
		if (!row) return '';
		if (row.state === 'error') return `⚠ ${row.failure ?? 'error'}`;
		const shown = row.samples.map(s => s.shown);
		return new Set(shown).size === 1 ? (shown[0] ?? '') : shown.join(' | ');
	};
	return table([
		['case', ...record.variants],
		...cases.map(c => [c, ...record.variants.map(v => cell(c, v))]),
	]);
}
