/** What one probe found on one frame at one width. */
export type ProbeResult = {
	probe: string;
	/** `false` is a failure the round has to answer for; see `scope` for how loudly. */
	ok: boolean;
	/** The measurement, in the probe's own units. Always printed, pass or fail. */
	note: string;
	/** One line per offending element. Empty on a pass. */
	findings: string[];
};

export type FrameReport = {
	id: string;
	variant: string;
	width: number;
	widthName: string;
	results: ProbeResult[];
};

/**
 * `floor` probes run on every frame at every width and a failure fails the command.
 * `settle` probes are the ones only asked before something is promoted — they print,
 * and they do not set the exit code.
 */
export type ProbeScope = 'floor' | 'settle';

const BAR = '─'.repeat(78);

function pad(value: string, width: number): string {
	return value.length >= width ? value : value + ' '.repeat(width - value.length);
}

/** Every frame that failed at least one probe of the given scope. */
export function failures(reports: FrameReport[], scopeOf: (probe: string) => ProbeScope): number {
	return reports.filter(report =>
		report.results.some(result => !result.ok && scopeOf(result.probe) === 'floor')
	).length;
}

/**
 * The report as the round pastes it into its own write-up: one block per frame, every
 * measurement printed whether it passed or not.
 *
 * Both numbers belong in the report — a probe run only after a fix proves the current
 * state, not the change — so nothing here is filtered to failures.
 */
export function formatReport(reports: FrameReport[]): string {
	if (reports.length === 0) return 'No frames probed.';

	const blocks = reports.map(report => {
		const head = `${report.id} · ${report.variant} · ${report.widthName} ${report.width}px`;
		const rows = report.results.map(result => {
			const mark = result.ok ? 'pass' : 'FAIL';
			const detail =
				result.findings.length > 0 ? `\n      ${result.findings.join('\n      ')}` : '';
			return `  ${pad(mark, 5)} ${pad(result.probe, 22)} ${result.note}${detail}`;
		});
		return [head, ...rows].join('\n');
	});

	return [BAR, ...blocks, BAR].join('\n');
}

/** One line per frame, for the end of the round's report. */
export function summarise(reports: FrameReport[], scopeOf: (probe: string) => ProbeScope): string {
	const failed = failures(reports, scopeOf);
	const frames = reports.length;
	const counted = `${frames} frame${frames === 1 ? '' : 's'} probed`;

	return failed === 0 ? `${counted} · floor clean` : `${counted} · ${failed} failing the floor`;
}
