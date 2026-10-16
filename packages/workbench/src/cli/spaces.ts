import type { FrameKind } from '../lib/frame-url';

/**
 * Space names as anything sensible spells them, resolved to the census `kind`.
 *
 * The workbench's own routes are `/views`, `/kit` and `/proto`; the census kind is `view`,
 * singular. One round lost a round-trip to `--space views` matching nothing and printing *"the
 * proto space is empty whenever nothing is being argued about"* — which reads like a true
 * statement about an empty space rather than a typo, so the run looked like it had passed.
 *
 * Both spellings resolve here, and a name that is neither is an error that names the three. A
 * silent empty result is the one outcome this must never produce.
 */
const ALIASES: Record<string, FrameKind> = {
	proto: 'proto',
	protos: 'proto',
	prototype: 'proto',
	prototypes: 'proto',
	view: 'view',
	views: 'view',
	place: 'view',
	places: 'view',
	kit: 'kit',
	kits: 'kit',
	component: 'kit',
	components: 'kit',
};

export const SPACES: FrameKind[] = ['proto', 'view', 'kit'];

export function resolveSpace(name: string): FrameKind {
	const kind = ALIASES[name.trim().toLowerCase()];
	if (kind === undefined) {
		throw new Error(
			`unknown space "${name}" — the three are ${SPACES.join(', ')} (plurals and "places" also resolve)`
		);
	}
	return kind;
}

export function resolveSpaces(names: string[]): FrameKind[] {
	return [...new Set(names.map(resolveSpace))];
}
