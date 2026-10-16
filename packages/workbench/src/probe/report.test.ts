import { describe, expect, it } from 'vitest';

import { failures, formatReport, summarise, type FrameReport, type ProbeScope } from './report';

const scopeOf = (probe: string): ProbeScope => (probe === 'contrast' ? 'settle' : 'floor');

const report = (overrides: Partial<FrameReport> = {}): FrameReport => ({
	id: 'split-rail',
	variant: 'both-open',
	width: 390,
	widthName: 'mobile',
	results: [{ probe: 'h-scroll', ok: true, note: 'over by 0', findings: [] }],
	...overrides,
});

describe('formatReport', () => {
	it('prints the measurement whether it passed or not', () => {
		const output = formatReport([report()]);

		expect(output).toContain('split-rail · both-open · mobile 390px');
		expect(output).toContain('over by 0');
	});

	it('marks a failure and indents its findings', () => {
		const output = formatReport([
			report({
				results: [
					{ probe: 'h-scroll', ok: false, note: 'over by 47', findings: ['div.row right=437'] },
				],
			}),
		]);

		expect(output).toContain('FAIL');
		expect(output).toContain('div.row right=437');
	});

	it('says so rather than printing an empty table', () => {
		expect(formatReport([])).toBe('No frames probed.');
	});
});

describe('failures', () => {
	it('counts a frame that fails a floor probe', () => {
		const failing = report({
			results: [{ probe: 'h-scroll', ok: false, note: 'over by 47', findings: [] }],
		});

		expect(failures([failing, report()], scopeOf)).toBe(1);
	});

	it('does not fail the command on a settle-scope probe', () => {
		const settle = report({
			results: [{ probe: 'contrast', ok: false, note: '1 under AA', findings: [] }],
		});

		expect(failures([settle], scopeOf)).toBe(0);
	});
});

describe('summarise', () => {
	it('reports a clean floor', () => {
		expect(summarise([report()], scopeOf)).toBe('1 frame probed · floor clean');
	});

	it('reports how many frames are failing', () => {
		const failing = report({
			results: [{ probe: 'hit-test', ok: false, note: '1 covered', findings: [] }],
		});

		expect(summarise([failing, report()], scopeOf)).toBe('2 frames probed · 1 failing the floor');
	});
});
