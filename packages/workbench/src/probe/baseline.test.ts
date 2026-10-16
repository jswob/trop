import { describe, expect, it } from 'vitest';

import { applyBaseline, keyOf, keysOf, type Baseline } from './baseline';
import type { FrameReport } from './report';

function report(id: string, variant: string, widthName: string, findings: string[]): FrameReport {
	return {
		id,
		variant,
		widthName,
		width: 1440,
		results: [
			{ probe: 'h-scroll', ok: true, note: '0', findings: [] },
			{
				probe: 'hit-test',
				ok: findings.length === 0,
				note: `${findings.length} covered`,
				findings,
			},
		],
	};
}

const COVERED = 'button "Open" is covered by div.absolute';
const OTHER = 'button "Close" is covered by div.absolute';

function baselineOf(keys: string[]): Baseline {
	return { accepted: '2026-09-20', note: 'adjudicated', keys };
}

describe('keysOf', () => {
	it('keys a finding by the frame, the state, the width and the probe', () => {
		expect(keysOf([report('a', 'b', 'mobile', [COVERED])])).toEqual([
			`a·b·mobile·hit-test·${COVERED}`,
		]);
	});

	it('keys nothing for a passing probe', () => {
		expect(keysOf([report('a', 'b', 'mobile', [])])).toEqual([]);
	});

	it('keeps the same finding on two widths apart', () => {
		const keys = keysOf([
			report('a', 'b', 'mobile', [COVERED]),
			report('a', 'b', 'desktop', [COVERED]),
		]);

		expect(new Set(keys).size).toBe(2);
	});
});

describe('applyBaseline', () => {
	/**
	 * A round settled a floating slug at the foot of the column, and the hit-test probe is right
	 * that it covers what is under it. `PROBES.md`: *a floating object covering column content is a
	 * reading, not a finding* — and *the number to watch is whether it grew*. Held findings stop
	 * failing the command; they do not stop being printed.
	 */
	it('holds a finding the baseline already adjudicated', () => {
		const reports = [report('a', 'b', 'mobile', [COVERED])];
		const held = applyBaseline(reports, baselineOf([keyOf(reports[0]!, 'hit-test', COVERED)]));

		expect(held.held).toBe(1);
		expect(held.reports[0]?.results[1]?.ok).toBe(true);
		expect(held.reports[0]?.results[1]?.note).toContain('held by the baseline');
	});

	it('fails a result the moment one of its findings is new', () => {
		const reports = [report('a', 'b', 'mobile', [COVERED, OTHER])];
		const held = applyBaseline(reports, baselineOf([keyOf(reports[0]!, 'hit-test', COVERED)]));

		expect(held.reports[0]?.results[1]?.ok).toBe(false);
		expect(held.held).toBe(0);
	});

	it('reports a held finding that no longer occurs, so the baseline can be pruned', () => {
		const held = applyBaseline(
			[report('a', 'b', 'mobile', [])],
			baselineOf([`a·b·mobile·hit-test·${COVERED}`])
		);

		expect(held.resolved).toEqual([`a·b·mobile·hit-test·${COVERED}`]);
	});

	/**
	 * A narrow run must never call the rest of the space resolved. `--frames one:state` looks at
	 * one frame; everything the baseline holds for the others is simply not evidence either way.
	 */
	it('says nothing about frames this run did not probe', () => {
		const held = applyBaseline(
			[report('a', 'b', 'mobile', [])],
			baselineOf([`z·y·desktop·hit-test·${COVERED}`])
		);

		expect(held.resolved).toEqual([]);
	});

	it('changes nothing at all when there is no baseline', () => {
		const reports = [report('a', 'b', 'mobile', [COVERED])];
		const held = applyBaseline(reports, null);

		expect(held.reports).toBe(reports);
		expect(held.held).toBe(0);
	});

	it('leaves a failure with no findings failing — there is nothing to key it by', () => {
		const bare: FrameReport = {
			id: 'a',
			variant: 'b',
			widthName: 'mobile',
			width: 390,
			results: [{ probe: 'not-empty', ok: false, note: '0 chars', findings: [] }],
		};

		expect(applyBaseline([bare], baselineOf([])).reports[0]?.results[0]?.ok).toBe(false);
	});
});
