import { render } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { formatTurned, parseTurned, specimen, turn, valueNames } from './specimen';

type Weight = 'primary' | 'normal' | 'quiet';

function Plate({
	children,
	weight = 'normal',
	running = null,
	disabled = false,
	icon,
}: {
	children: string;
	weight?: Weight;
	running?: string | null;
	disabled?: boolean;
	icon?: 'plus' | 'cross';
}) {
	return (
		<button type="button" data-weight={weight} data-icon={icon} disabled={disabled}>
			{running ?? children}
		</button>
	);
}

const plate = specimen(Plate, {
	kit: 'plate',
	is: 'A thing to press.',
	base: { children: 'Assign', weight: 'normal' },
	states: {
		rest: { label: 'At rest', note: 'The middle rung.', args: {} },
		running: {
			label: 'Running',
			note: 'Its run is going.',
			args: { weight: 'primary', running: 'Saving' },
		},
	},
	props: {
		children: { is: 'The word.' },
		weight: {
			is: 'How much surface it owns.',
			options: { primary: true, normal: true, quiet: true },
		},
		running: { is: 'The word while it runs.', text: true },
		disabled: { is: 'Off.', toggle: true },
		icon: { is: 'A glyph.', choices: { none: undefined, plus: 'plus' } },
	},
	parts: { plate: { select: 'button', is: 'The ground.' } },
	holds: ['Emphasis is area.'],
	refuses: ['A coloured rung.'],
});

describe('specimen', () => {
	it('keeps the states in the order they are written, each with its words', () => {
		expect(plate.states.map(state => [state.id, state.label, state.note])).toEqual([
			['rest', 'At rest', 'The middle rung.'],
			['running', 'Running', 'Its run is going.'],
		]);
	});

	it('says how each prop can be turned', () => {
		expect(plate.props.map(prop => [prop.name, prop.kind, prop.names])).toEqual([
			['children', 'none', []],
			['weight', 'options', ['primary', 'normal', 'quiet']],
			['running', 'text', []],
			['disabled', 'toggle', ['off', 'on']],
			['icon', 'choices', ['none', 'plus']],
		]);
	});

	it('draws a state with the props it declares over the ones every state shares', () => {
		const { getByRole } = render(<>{plate.render('running', {})}</>);

		expect(getByRole('button').textContent).toBe('Saving');
		expect(getByRole('button').dataset['weight']).toBe('primary');
	});

	it('draws what the address turned over what the state declares', () => {
		const { getByRole } = render(
			<>{plate.render('running', { weight: 'quiet', running: '', disabled: 'on', icon: 'plus' })}</>
		);
		const button = getByRole('button');

		expect(button.dataset['weight']).toBe('quiet');
		expect(button.textContent).toBe('Assign');
		expect(button).toHaveProperty('disabled', true);
		expect(button.dataset['icon']).toBe('plus');
	});

	it('ignores a turn it cannot read, and a prop nobody can turn', () => {
		const { getByRole } = render(
			<>{plate.render('rest', { weight: 'loud', children: 'Hacked', icon: 'star' })}</>
		);

		expect(getByRole('button').dataset['weight']).toBe('normal');
		expect(getByRole('button').textContent).toBe('Assign');
	});

	it('names the parts and keeps what it holds and refuses', () => {
		expect(plate.parts).toEqual([{ name: 'plate', select: 'button', is: 'The ground.' }]);
		expect(plate.holds).toEqual(['Emphasis is area.']);
		expect(plate.refuses).toEqual(['A coloured rung.']);
	});
});

describe('a state the specimen draws whole', () => {
	const drawn = specimen(Plate, {
		kit: 'plate',
		is: 'A thing to press.',
		states: {
			rest: { label: 'At rest', note: 'The middle rung.', args: { children: 'Assign' } },
			pair: {
				label: 'A pair',
				note: 'Two of them, side by side.',
				draw: () => (
					<>
						<Plate>Keep</Plate>
						<Plate weight="primary">Remove</Plate>
					</>
				),
			},
		},
		props: {
			children: { is: 'The word.' },
			weight: {
				is: 'How much surface it owns.',
				options: { primary: true, normal: true, quiet: true },
			},
			running: { is: 'The word while it runs.' },
			disabled: { is: 'Off.' },
			icon: { is: 'A glyph.' },
		},
		holds: [],
		refuses: [],
	});

	it('says so, since nothing turned from the bar reaches it', () => {
		expect(drawn.states.map(state => [state.id, state.drawn])).toEqual([
			['rest', false],
			['pair', true],
		]);
	});

	it('is drawn as written whatever the address turned', () => {
		const { getAllByRole } = render(<>{drawn.render('pair', { weight: 'quiet' })}</>);

		expect(getAllByRole('button').map(button => button.dataset['weight'])).toEqual([
			'normal',
			'primary',
		]);
	});
});

describe('valueNames', () => {
	it('names what a state declares for every prop that can be turned', () => {
		expect(valueNames(plate, 'running')).toEqual({
			weight: 'primary',
			running: 'Saving',
			disabled: 'off',
			icon: 'none',
		});
	});

	it('names the component’s own default where the state says nothing', () => {
		expect(valueNames(plate, 'rest')).toMatchObject({ running: '', disabled: 'off' });
	});
});

describe('the turned props in the address', () => {
	it('reads a list of name and value pairs', () => {
		expect(parseTurned('weight:primary,running:Saving%20now')).toEqual({
			weight: 'primary',
			running: 'Saving now',
		});
	});

	it('reads nothing from nothing, and skips what is not a pair', () => {
		expect(parseTurned(undefined)).toEqual({});
		expect(parseTurned('weight,,:x')).toEqual({});
	});

	it('writes them back the way it reads them, with a comma or a colon in a value kept', () => {
		const turned = { weight: 'quiet', running: 'One, two: three' };

		expect(parseTurned(formatTurned(turned))).toEqual(turned);
	});

	it('writes nothing for nothing turned', () => {
		expect(formatTurned({})).toBeUndefined();
	});

	it('forgets a prop turned back to what the state declares', () => {
		const declared = valueNames(plate, 'rest');

		expect(turn({ weight: 'quiet' }, declared, 'weight', 'normal')).toEqual({});
		expect(turn({}, declared, 'weight', 'primary')).toEqual({ weight: 'primary' });
		expect(turn({ weight: 'quiet' }, declared, 'disabled', 'on')).toEqual({
			weight: 'quiet',
			disabled: 'on',
		});
	});
});
