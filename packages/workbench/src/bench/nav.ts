export type NavPlace = { id: string; title: string; states: number };

export type NavInput = {
	/** The place every page opens over, if it has been drawn, and what the app calls it. */
	ground?: NavPlace & { label: string };
	destinations: { id: string; label: string; places: NavPlace[] }[];
	kit: { id: string; name: string }[];
	protos: { id: string; title: string }[];
};

export type NavRow = {
	label: string;
	/** Absent on a destination that only holds its places. */
	to: string | undefined;
	/** States of a place, or components of the kit. */
	count?: number;
	children?: NavRow[];
};

export type NavGroup = { title: string; rows: NavRow[]; empty?: string };

const placeRow = (place: NavPlace, label = place.title): NavRow => ({
	label,
	to: `/views/${place.id}`,
	count: place.states,
});

/**
 * The sidebar: the four spaces, and under Views the app's own destinations.
 *
 * Views opens with every place on one page, as Kit opens with every component.
 *
 * A destination with one place is a row under the destination's name, because that is what the
 * app calls it. One with several folds, and its places are the rows. A destination nobody has
 * drawn is not a row: a nav lists places you can go.
 */
export function navGroups(input: NavInput): NavGroup[] {
	const places =
		(input.ground ? 1 : 0) + input.destinations.reduce((sum, dest) => sum + dest.places.length, 0);
	const views: NavRow[] = [
		{ label: 'Places', to: '/views', count: places },
		...(input.ground ? [placeRow(input.ground, input.ground.label)] : []),
		...input.destinations.flatMap((dest): NavRow[] => {
			const [first, ...rest] = dest.places;

			if (first === undefined) return [];
			if (rest.length === 0) return [placeRow(first, dest.label)];

			return [
				{ label: dest.label, to: undefined, children: dest.places.map(place => placeRow(place)) },
			];
		}),
	];

	return [
		{ title: 'Tokens', rows: [{ label: 'Tokens', to: '/tokens' }] },
		{
			title: 'Kit',
			rows: [
				{
					label: 'Components',
					to: '/kit',
					count: input.kit.length,
					children: input.kit.map(entry => ({
						label: kitName(entry.name),
						to: `/kit/${entry.id}`,
					})),
				},
				{ label: 'Stress page', to: '/kit/stress' },
			],
		},
		{ title: 'Views', rows: views },
		{
			title: 'Proto',
			rows: input.protos.map(proto => ({ label: proto.title, to: `/proto/${proto.id}` })),
			...(input.protos.length === 0 ? { empty: 'Nothing unsettled' } : {}),
		},
	];
}

export type JumpEntry = {
	/** What the list prints and what a query is matched against, with `id`. */
	label: string;
	id: string;
	/** Where it belongs, printed after the name. */
	hint: string;
	to: string;
};

/** A component's name as the bench prints it: its several exports joined with commas. */
export function kitName(name: string): string {
	return name.replaceAll(' · ', ', ');
}

/** Everything the jump list can open: every place, then every kit component. */
export function jumpEntries(input: NavInput): JumpEntry[] {
	const place =
		(hint: string) =>
		(entry: NavPlace): JumpEntry => ({
			label: entry.title,
			id: entry.id,
			hint: `${hint}, ${String(entry.states)} ${entry.states === 1 ? 'state' : 'states'}`,
			to: `/views/${entry.id}`,
		});

	return [
		...(input.ground ? [place(input.ground.label)(input.ground)] : []),
		...input.destinations.flatMap(dest => dest.places.map(place(dest.label))),
		...input.kit.map(entry => ({
			label: kitName(entry.name),
			id: entry.id,
			hint: 'Kit',
			to: `/kit/${entry.id}`,
		})),
	];
}

/** The entries a typed query leaves, matched on any part of a name or an id. */
export function matchJump(entries: readonly JumpEntry[], query: string): JumpEntry[] {
	const needle = query.trim().toLowerCase();
	if (needle === '') return [...entries];

	return entries.filter(
		entry => entry.label.toLowerCase().includes(needle) || entry.id.toLowerCase().includes(needle)
	);
}
