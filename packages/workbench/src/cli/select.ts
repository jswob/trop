import type { FrameEntry } from '../lib/frame-census';
import type { FrameKind } from '../lib/frame-url';

/**
 * Which frames a command was pointed at.
 *
 * `id` takes every variant that id has; `id:variant` takes the one. An empty `spaces` means every
 * space, which is what `--all` asks for.
 */
export function selectFrames(
	census: FrameEntry[],
	options: { frames: string[]; spaces: FrameKind[] }
): FrameEntry[] {
	if (options.frames.length > 0) {
		return census.filter(entry =>
			options.frames.some(token => {
				const [id, variant] = token.split(':');
				return (
					entry.id === id && (variant === undefined || variant === '' || entry.variant === variant)
				);
			})
		);
	}

	if (options.spaces.length === 0) return census;
	return census.filter(entry => options.spaces.includes(entry.kind));
}

/**
 * The one frame a command that drives needs, and an error naming the near misses when the token
 * does not resolve to exactly one.
 *
 * Driving is not probing: a probe run over four frames is four reports, but a press applies to one
 * screen, and "which of these did I press" is not a question a report can answer afterwards.
 */
export function selectOne(census: FrameEntry[], token: string): FrameEntry {
	const [id, variant] = token.split(':');
	const byId = census.filter(entry => entry.id === id);

	if (byId.length === 0) {
		const near = census
			.map(entry => entry.id)
			.filter((value, index, all) => all.indexOf(value) === index)
			.filter(value => value.includes(id ?? '') || (id ?? '').includes(value))
			.slice(0, 8);
		throw new Error(
			`no frame with id "${id ?? ''}"${near.length > 0 ? ` — did you mean ${near.join(', ')}?` : ''}\nRun "workbench frames" for the list.`
		);
	}

	if (variant === undefined || variant === '') {
		if (byId.length === 1 && byId[0] !== undefined) return byId[0];
		throw new Error(
			`"${id ?? ''}" has ${byId.length} states — name one: ${byId.map(entry => `${entry.id}:${entry.variant}`).join(', ')}`
		);
	}

	const exact = byId.find(entry => entry.variant === variant);
	if (exact === undefined) {
		throw new Error(
			`"${id ?? ''}" has no state "${variant}" — it has ${byId.map(entry => entry.variant).join(', ')}`
		);
	}

	return exact;
}
