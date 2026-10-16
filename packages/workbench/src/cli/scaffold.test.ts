import { describe, expect, it } from 'vitest';

import {
	assertDrawing,
	camelOf,
	pascalOf,
	protoTemplate,
	registerProto,
	thesisTemplate,
	unregisterProto,
} from './scaffold';

const EMPTY = `import { parseProposedShape } from '../lib/proposed-shape';
import type { Subject } from '../lib/subject';

/** Every Tier 2 prototype. */
export const PROTOS: Subject[] = [];

export function protoById(id: string) {
	return PROTOS.find(proto => proto.id === id);
}
`;

describe('naming', () => {
	it('turns a kebab id into the names the two spaces use', () => {
		expect(camelOf('split-rail')).toBe('splitRail');
		expect(pascalOf('split-rail')).toBe('SplitRail');
		expect(camelOf('rules')).toBe('rules');
		expect(pascalOf('page-spread-2')).toBe('PageSpread2');
	});
});

describe('registerProto', () => {
	it('adds the import and the array member', () => {
		const next = registerProto(EMPTY, 'split-rail');

		expect(next).toContain(`import { splitRail } from './split-rail';`);
		expect(next).toContain('export const PROTOS: Subject[] = [splitRail];');
	});

	it('appends beside a candidate that is already there', () => {
		const next = registerProto(registerProto(EMPTY, 'one'), 'two-up');

		expect(next).toContain('export const PROTOS: Subject[] = [one, twoUp];');
	});

	/** Lint sorts sibling imports by path, and nothing runs lint between `new` and `check`. */
	it('puts the import where lint sorts it, among the candidates already there', () => {
		const next = registerProto(
			registerProto(registerProto(EMPTY, 'person-tally'), 'person-bar'),
			'person-spread'
		);

		expect(next.match(/^import \{ \w+ \} from '\.\/[\w-]+';$/gm)).toEqual([
			"import { personBar } from './person-bar';",
			"import { personSpread } from './person-spread';",
			"import { personTally } from './person-tally';",
		]);
	});

	/**
	 * `BUILD.md` requires one thread to do the registering because three agents editing one file is
	 * a race. Idempotence is what makes that a convenience rather than a rule: running it twice
	 * must not append a second import, whoever ran it.
	 */
	it('is idempotent', () => {
		const once = registerProto(EMPTY, 'split-rail');

		expect(registerProto(once, 'split-rail')).toBe(once);
	});

	it('refuses a file with no prototype list rather than writing a broken one', () => {
		expect(() => registerProto('export const OTHER = [];', 'x')).toThrow(/no .export const PROTOS/);
	});
});

describe('unregisterProto', () => {
	it('undoes exactly what registering did', () => {
		expect(unregisterProto(registerProto(EMPTY, 'split-rail'), 'split-rail')).toBe(EMPTY);
	});

	it('leaves the other candidates alone', () => {
		const both = registerProto(registerProto(EMPTY, 'one'), 'two');
		const left = unregisterProto(both, 'one');

		expect(left).toContain('export const PROTOS: Subject[] = [two];');
		expect(left).not.toContain(`from './one'`);
	});

	it('does nothing to a file that never held it', () => {
		expect(unregisterProto(EMPTY, 'never-existed')).toBe(EMPTY);
	});
});

/** The app a candidate is scaffolded for. */
const ON = { app: 'Ledger', theme: 'src/ui/theme.css', engine: '@acme/workbench' };

describe('protoTemplate', () => {
	it('declares the PROPOSED_SHAPE that proto.test asserts every candidate has', () => {
		expect(protoTemplate('split-rail', 'Split rail', 'The rail stands on the split.', ON)).toMatch(
			/export type PROPOSED_SHAPE/
		);
	});

	it('exports the subject under the name the registry imports', () => {
		expect(protoTemplate('split-rail', 'Split rail', 'bet', ON)).toContain(
			'export const splitRail: Subject = {'
		);
	});

	it('carries the thesis into the file rather than leaving it in the brief', () => {
		expect(protoTemplate('x', 'X', 'Status colour carries the hierarchy.', ON)).toContain(
			'**The bet.** Status colour carries the hierarchy.'
		);
	});

	it('writes a title and a bet with an apostrophe as strings that still parse', () => {
		const source = protoTemplate('x', 'The meter’s band', "Share as the meter's band.", ON);

		expect(source).toContain(`title: 'The meter’s band',`);
		expect(source).toContain(`description: 'Share as the meter\\'s band.',`);
	});

	it('imports nothing but the engine, so a new candidate cannot reach a backend', () => {
		const source = protoTemplate('x', 'X', 'bet', ON);
		const imported = [...source.matchAll(/from '([^']+)'/g)].map(match => match[1]);

		expect(imported).toEqual([ON.engine]);
	});
});

describe('assertDrawing', () => {
	const drawing = protoTemplate('person-tally', 'A · Tally', 'The count beside the seat.', ON);

	it('takes a board’s drawing that is already the candidate it will become', () => {
		expect(assertDrawing(drawing, 'person-tally')).toBe(drawing);
	});

	it('refuses a drawing the registry could not import under that id', () => {
		expect(() => assertDrawing(drawing, 'person-bar')).toThrow(/export const personBar/);
	});

	it('refuses a drawing whose subject answers to another id', () => {
		const renamed = drawing.replace("id: 'person-tally'", "id: 'tally'");

		expect(() => assertDrawing(renamed, 'person-tally')).toThrow(/id: 'person-tally'/);
	});
});

describe('thesisTemplate', () => {
	it('exports the numbers under the name the conform verb reads', () => {
		const source = thesisTemplate('split-rail', 'The rail stands on the split.', '@acme/workbench');

		expect(source).toContain(`import { thesis } from '@acme/workbench/thesis';`);
		expect(source).toContain('export const numbers = thesis({');
		expect(source).toContain(`id: 'split-rail',`);
	});

	it('writes a bet with an apostrophe as a string that still parses, and starts with no checks', () => {
		const source = thesisTemplate('x', "Share as the meter's band.", '@acme/workbench');

		expect(source).toContain(`bet: 'Share as the meter\\'s band.',`);
		expect(source).toContain('checks: [],');
	});
});
