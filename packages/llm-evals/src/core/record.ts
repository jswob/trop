// Reads a run record: the JSON `promptfoo export eval <id>` writes (format version 3). Everything
// after a run (the integrity gate, the report) reads this file, never promptfoo's database or its
// summary line.
import type { FailureKind } from './outcome.ts';
import type { SampleRecord } from './provider.ts';

export interface RecordRow {
	/** the column's label */
	variant: string;
	/** the column's position */
	column: number;
	caseId: string;
	split: string;
	group?: string;
	/** scored and passed, scored and failed, or never scored (a technical error) */
	state: 'pass' | 'fail' | 'error';
	/** why a row was never scored; `unknown` when the error did not come from our provider */
	failure?: FailureKind | 'unknown';
	error?: string;
	scores: Record<string, number>;
	/** the scorer's line per sample */
	reason: string;
	cost: number;
	latencyMs: number;
	samples: SampleRecord[];
}

export interface RunRecord {
	evalId: string;
	description: string;
	tags: Record<string, string>;
	/** column labels, in column order */
	variants: string[];
	rows: RecordRow[];
}

type Json = Record<string, unknown>;
const obj = (value: unknown): Json =>
	typeof value === 'object' && value !== null ? (value as Json) : {};
const str = (value: unknown): string => (typeof value === 'string' ? value : '');
const num = (value: unknown): number => (typeof value === 'number' ? value : 0);

/**
 * The scorer's own lines. promptfoo replaces the top-level reason of a passing row with
 * "All assertions passed"; the assertion's own result keeps what the scorer wrote.
 */
function scorerReason(gradingResult: unknown): string {
	const grading = obj(gradingResult);
	const components = Array.isArray(grading.componentResults) ? grading.componentResults : [];
	return str(obj(components[0]).reason) || str(grading.reason);
}

function parseRow(raw: unknown): RecordRow {
	const row = obj(raw);
	const provider = obj(row.provider);
	const metadata = { ...obj(obj(row.response).metadata), ...obj(row.metadata) };
	const failureReason = num(row.failureReason);
	const state = failureReason === 2 ? 'error' : row.success === true ? 'pass' : 'fail';
	const error = str(row.error);
	const group = str(metadata.group);
	return {
		variant: str(provider.label) || str(provider.id),
		column: num(row.promptIdx),
		caseId: str(obj(row.vars).id),
		split: str(metadata.split),
		...(group ? { group } : {}),
		state,
		...(state === 'error'
			? { failure: (str(metadata.failure) || 'unknown') as RecordRow['failure'], error }
			: {}),
		scores: obj(row.namedScores) as Record<string, number>,
		reason: scorerReason(row.gradingResult),
		cost: num(row.cost),
		latencyMs: num(row.latencyMs),
		samples: Array.isArray(metadata.samples) ? (metadata.samples as SampleRecord[]) : [],
	};
}

export function parseRecord(json: unknown): RunRecord {
	const file = obj(json);
	const results = obj(file.results);
	if (results.version !== 3 || !Array.isArray(results.results)) {
		throw new Error('Not a promptfoo eval export of format version 3.');
	}
	const config = obj(file.config);
	const rows = results.results.map(parseRow);
	const byColumn = new Map([...rows].sort((a, b) => a.column - b.column).map(r => [r.variant, r]));
	const fromPrompts = (Array.isArray(results.prompts) ? results.prompts : [])
		.map(p => str(obj(p).provider))
		.filter(Boolean);
	return {
		evalId: str(file.evalId),
		description: str(config.description),
		tags: obj(config.tags) as Record<string, string>,
		// From the rows when there are any; a run that saved none still names its columns.
		variants: byColumn.size ? [...byColumn.keys()] : fromPrompts,
		rows,
	};
}
