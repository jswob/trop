import { describe, expect, it } from 'vitest';

import { specimen } from './specimen';

function Plate(props: {
	children: string;
	weight?: 'primary' | 'quiet';
	busy?: boolean;
	count?: number;
}) {
	return props.children;
}

const words = { kit: 'plate', is: 'A thing.', holds: [], refuses: [] };
const lines = {
	children: { is: 'The word.' },
	weight: { is: 'Its weight.' },
	busy: { is: 'Whether it is busy.' },
	count: { is: 'A number.' },
};
const states = { rest: { label: 'At rest', note: 'Nothing.', args: {} } };

/**
 * What the compiler refuses. Each `@ts-expect-error` fails the type check the day the line under it
 * starts compiling, which is the day a specimen could drift from its component unnoticed.
 */
describe('a specimen is checked against its component', () => {
	it('compiles when every prop has a line', () => {
		expect(specimen(Plate, { ...words, states, props: lines }).props).toHaveLength(4);
	});

	it('refuses a prop left without a line', () => {
		specimen(Plate, {
			...words,
			states,
			// @ts-expect-error -- `count` has no line
			props: { children: lines.children, weight: lines.weight, busy: lines.busy },
		});
		expect(true).toBe(true);
	});

	it('refuses a line for a prop the component does not take', () => {
		// @ts-expect-error -- `colour` is not a prop
		specimen(Plate, { ...words, states, props: { ...lines, colour: { is: 'A hue.' } } });
		expect(true).toBe(true);
	});

	it('refuses a list of options with one missing, and one with a word the prop does not take', () => {
		specimen(Plate, {
			...words,
			states,
			// @ts-expect-error -- `quiet` is missing
			props: { ...lines, weight: { is: 'Its weight.', options: { primary: true } } },
		});
		specimen(Plate, {
			...words,
			states,
			props: {
				...lines,
				// @ts-expect-error -- `loud` is not a weight
				weight: { is: 'Its weight.', options: { primary: true, quiet: true, loud: true } },
			},
		});
		expect(true).toBe(true);
	});

	it('refuses a switch on a prop that is not a boolean, and a text box on one that is not a string', () => {
		specimen(Plate, {
			...words,
			states,
			// @ts-expect-error -- `count` is a number
			props: { ...lines, count: { is: 'A number.', toggle: true } },
		});
		// @ts-expect-error -- `busy` is a boolean
		specimen(Plate, { ...words, states, props: { ...lines, busy: { is: 'Busy.', text: true } } });
		expect(true).toBe(true);
	});

	it('refuses a state drawn with a prop the component does not take', () => {
		specimen(Plate, {
			...words,
			props: lines,
			// @ts-expect-error -- `tone` is not a prop
			states: { rest: { label: 'At rest', note: 'Nothing.', args: { tone: 'warm' } } },
		});
		expect(true).toBe(true);
	});
});
