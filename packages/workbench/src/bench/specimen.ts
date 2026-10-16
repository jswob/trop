import { createElement, type ComponentType, type ReactNode } from 'react';

import type { Subject } from '../lib/subject';

type NotPlayed = { options?: undefined; choices?: undefined; text?: undefined; toggle?: undefined };

/**
 * How a hand may turn one prop from the bench, if it may at all.
 *
 * `options` is for a prop that takes a short list of words, and it must name **every** one of
 * them: the compiler refuses a word the prop does not take and refuses a list with one missing.
 * `choices` names values that are not words — an icon, a number. `text` and `toggle` are offered
 * only to a prop whose type is a string or a boolean. A prop that is a function, an element or a
 * fixture is described and left alone.
 */
type Play<V> =
	| NotPlayed
	| { choices: Readonly<Record<string, V>> }
	| (string extends NonNullable<V> ? { text: true } : never)
	| (boolean extends NonNullable<V> ? { toggle: true } : never)
	| (string extends NonNullable<V>
			? never
			: [Extract<NonNullable<V>, string>] extends [never]
				? never
				: { options: { readonly [K in Extract<NonNullable<V>, string>]: true } });

export type PropDoc<V> = { is: string } & Play<V>;

type StateInput<P> = {
	label: string;
	/** Why the state is listed. */
	note: string;
} & ({ args: Partial<P> } | { draw: () => ReactNode });

export type SpecimenInput<P> = {
	/** The kit entry it is the page of. */
	kit: string;
	/** What it is. */
	is: string;
	/** A component is drawn at its own size unless it needs a whole screen to be itself. */
	stands?: 'content' | 'screen';
	/** The measure it is given, for one that takes its container's. */
	width?: number;
	/** What every state is drawn with, under what the state declares. */
	base?: Partial<P>;
	states: Record<string, StateInput<P>>;
	/** A line for **every** prop the component takes. One left out does not compile. */
	props: { [K in keyof P]-?: PropDoc<P[K]> };
	/** Its parts, by selector, so the bench can measure them without the component knowing. */
	parts?: Record<string, { select: string; is: string }>;
	holds: string[];
	refuses: string[];
};

export type SpecimenState = {
	id: string;
	label: string;
	note: string;
	/** Drawn whole by the specimen, so nothing turned from the bar reaches it. */
	drawn: boolean;
};

export type SpecimenProp = {
	name: string;
	is: string;
	kind: 'options' | 'choices' | 'text' | 'toggle' | 'none';
	/** The values it can be turned to, by name. Empty for text, and for a prop nobody turns. */
	names: string[];
};

export type SpecimenPart = { name: string; select: string; is: string };

/**
 * One component's page, as data.
 *
 * The bench draws every component page from one of these, the same way, so a page is filled in
 * rather than composed. It is also what a tool reads: the states have addresses, the props have
 * names and values, and the parts have selectors.
 */
export type Specimen = {
	/** The kit entry's id. */
	id: string;
	/** The staged subject its states are the variants of. */
	subjectId: string;
	is: string;
	stands: 'content' | 'screen';
	/** Its states are a subject staged somewhere else, so the bench stages none for it. */
	staged: boolean;
	width?: number;
	states: SpecimenState[];
	props: SpecimenProp[];
	parts: SpecimenPart[];
	holds: string[];
	refuses: string[];
	/** What each state declares for every prop that can be turned, by the value's name. */
	declared: Record<string, Record<string, string>>;
	/** One state, with what the address turned drawn over what the state declares. */
	render: (stateId: string, turned: Record<string, string>) => ReactNode;
};

type Doc = {
	is: string;
	options?: object;
	choices?: Record<string, unknown>;
	text?: true;
	toggle?: true;
};

const TOGGLE = ['off', 'on'];

function describe(name: string, doc: Doc): SpecimenProp {
	if (doc.options !== undefined) {
		return { name, is: doc.is, kind: 'options', names: Object.keys(doc.options) };
	}
	if (doc.choices !== undefined) {
		return { name, is: doc.is, kind: 'choices', names: Object.keys(doc.choices) };
	}
	if (doc.text === true) return { name, is: doc.is, kind: 'text', names: [] };
	if (doc.toggle === true) return { name, is: doc.is, kind: 'toggle', names: TOGGLE };

	return { name, is: doc.is, kind: 'none', names: [] };
}

/** The value a name stands for, or nothing when the prop cannot be turned to it. */
function valueOf(prop: SpecimenProp, doc: Doc, name: string): { value: unknown } | undefined {
	switch (prop.kind) {
		case 'options':
			return prop.names.includes(name) ? { value: name } : undefined;
		case 'choices':
			return doc.choices !== undefined && name in doc.choices
				? { value: doc.choices[name] }
				: undefined;
		case 'toggle':
			return TOGGLE.includes(name) ? { value: name === 'on' } : undefined;
		case 'text':
			// Emptied, it goes back to the component's own default rather than to an empty string.
			return { value: name === '' ? undefined : name };
		case 'none':
			return undefined;
	}
}

/** The name of the value a state draws a prop with. Empty where neither says. */
function nameOf(prop: SpecimenProp, doc: Doc, value: unknown): string {
	switch (prop.kind) {
		case 'options':
			return typeof value === 'string' ? value : '';
		case 'choices':
			return Object.entries(doc.choices ?? {}).find(([, choice]) => choice === value)?.[0] ?? '';
		case 'toggle':
			return value === true ? 'on' : 'off';
		case 'text':
			return typeof value === 'string' ? value : '';
		case 'none':
			return '';
	}
}

/**
 * Declares a component's page.
 *
 * Typed against the component itself: a state's `args` are its props, `props` must hold a line
 * for every one of them, and a prop's `options` must be every word it takes. Rename a prop and
 * this stops compiling, which is the check the written breakdowns never had.
 */
export function specimen<P extends object>(
	component: ComponentType<P>,
	input: SpecimenInput<P>
): Specimen {
	const docs = input.props as Record<string, Doc>;
	const props = Object.entries(docs).map(([name, doc]) => describe(name, doc));
	const states = Object.entries(input.states);
	const base = (input.base ?? {}) as Record<string, unknown>;
	const argsOf = (id: string): Record<string, unknown> | undefined => {
		const state = input.states[id];
		return state !== undefined && 'args' in state
			? { ...base, ...(state.args as Record<string, unknown>) }
			: undefined;
	};

	return {
		id: input.kit,
		subjectId: `kit-${input.kit}`,
		is: input.is,
		stands: input.stands ?? 'content',
		staged: false,
		...(input.width === undefined ? {} : { width: input.width }),
		states: states.map(([id, state]) => ({
			id,
			label: state.label,
			note: state.note,
			drawn: 'draw' in state,
		})),
		props,
		parts: Object.entries(input.parts ?? {}).map(([name, part]) => ({ name, ...part })),
		holds: input.holds,
		refuses: input.refuses,
		declared: Object.fromEntries(
			states.map(([id]) => {
				const args = argsOf(id);

				return [
					id,
					args === undefined
						? {}
						: Object.fromEntries(
								props
									.filter(prop => prop.kind !== 'none')
									.map(prop => [
										prop.name,
										nameOf(prop, docs[prop.name] ?? { is: '' }, args[prop.name]),
									])
							),
				];
			})
		),
		render: (stateId, turned) => {
			const state = input.states[stateId];
			if (state === undefined) return null;
			if ('draw' in state) return state.draw();

			const overrides: Record<string, unknown> = {};
			for (const prop of props) {
				const name = turned[prop.name];
				const doc = docs[prop.name];
				if (name === undefined || doc === undefined) continue;

				const found = valueOf(prop, doc, name);
				if (found !== undefined) overrides[prop.name] = found.value;
			}

			return createElement(component, { ...argsOf(stateId), ...overrides } as P);
		},
	};
}

/**
 * The page of a component that is already staged as frames of its own: a shell that needs a whole
 * screen and a fixture to be itself. Its states and their words are the staged subject's.
 */
export function specimenOf(
	subject: Subject,
	input: {
		kit: string;
		props: Record<string, string>;
		holds: string[];
		refuses: string[];
	}
): Specimen {
	return {
		id: input.kit,
		subjectId: subject.id,
		is: subject.description,
		stands: 'screen',
		staged: true,
		states: subject.variants.map(variant => ({
			id: variant.id,
			label: variant.label,
			note: variant.note ?? '',
			drawn: true,
		})),
		props: Object.entries(input.props).map(([name, is]) => ({ name, is, kind: 'none', names: [] })),
		parts: [],
		holds: input.holds,
		refuses: input.refuses,
		declared: {},
		render: stateId => subject.variants.find(variant => variant.id === stateId)?.render() ?? null,
	};
}

/** What a state declares for every prop that can be turned, by the value's name. */
export function valueNames(spec: Specimen, stateId: string): Record<string, string> {
	return spec.declared[stateId] ?? {};
}

/** The props the address turned: `weight:primary,running:Saving`. */
export function parseTurned(value: unknown): Record<string, string> {
	if (typeof value !== 'string') return {};

	const turned: Record<string, string> = {};
	for (const pair of value.split(',')) {
		const at = pair.indexOf(':');
		if (at <= 0) continue;

		turned[pair.slice(0, at)] = decodeURIComponent(pair.slice(at + 1));
	}

	return turned;
}

/** The turned props as the address carries them. Nothing, when nothing is turned. */
export function formatTurned(turned: Record<string, string>): string | undefined {
	const pairs = Object.entries(turned).map(
		([name, value]) => `${name}:${encodeURIComponent(value)}`
	);

	return pairs.length > 0 ? pairs.join(',') : undefined;
}

/** One prop turned. Turned back to what the state declares, it is forgotten. */
export function turn(
	turned: Record<string, string>,
	declared: Record<string, string>,
	name: string,
	value: string
): Record<string, string> {
	const rest = Object.fromEntries(Object.entries(turned).filter(([other]) => other !== name));

	return declared[name] === value ? rest : { ...rest, [name]: value };
}
