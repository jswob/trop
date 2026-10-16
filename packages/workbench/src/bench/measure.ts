/** A colour as the page computes it: red, green and blue to 255, and how opaque. */
export type Rgba = [number, number, number, number];

/** Reads a colour in any form a browser computes one to, or a stylesheet's hex. */
export function readColor(value: string): Rgba | null {
	const text = value.trim().toLowerCase();

	const hex = /^#([0-9a-f]{3}|[0-9a-f]{6})$/.exec(text)?.[1];
	if (hex !== undefined) {
		const full = hex.length === 3 ? [...hex].map(digit => digit + digit).join('') : hex;
		const whole = parseInt(full, 16);
		return [(whole >> 16) & 255, (whole >> 8) & 255, whole & 255, 1];
	}

	if (!/^(rgba?|color)\(/.test(text)) return null;

	const [r, g, b, a] = (text.match(/[\d.]+/g) ?? []).map(Number);
	if (r === undefined || g === undefined || b === undefined) return null;

	const scale = text.startsWith('color(') ? 255 : 1;
	return [r * scale, g * scale, b * scale, a ?? 1];
}

/** One colour laid over an opaque one. */
export function over(top: Rgba, under: Rgba): Rgba {
	const mix = (index: 0 | 1 | 2): number => top[index] * top[3] + under[index] * (1 - top[3]);
	return [mix(0), mix(1), mix(2), 1];
}

function luminance([r, g, b]: Rgba): number {
	const channel = (value: number): number => {
		const part = value / 255;
		return part <= 0.03928 ? part / 12.92 : ((part + 0.055) / 1.055) ** 2.4;
	};

	return 0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b);
}

/** The WCAG contrast of an ink on the ground it sits on. A see-through ink is laid over it first. */
export function contrastOf(ink: Rgba, ground: Rgba): number {
	const a = luminance(over(ink, ground));
	const b = luminance(ground);

	return (Math.max(a, b) + 0.05) / (Math.min(a, b) + 0.05);
}

export function toHex([r, g, b]: Rgba): string {
	return `#${[r, g, b].map(value => Math.round(value).toString(16).padStart(2, '0')).join('')}`;
}

export type PartReading = {
	name: string;
	/** The letter its outline carries on the frame. */
	tag: string;
	/** Its box, from the specimen's own corner. */
	x: number;
	y: number;
	width: number;
	height: number;
	/** Top, right, bottom, left. Nothing when it has none. */
	padding: [number, number, number, number] | null;
	/** Its type size and weight. Nothing for a glyph. */
	type: string | null;
	edge: string | null;
	ink: string;
	ground: string;
	contrast: number;
};

const WHITE: Rgba = [255, 255, 255, 1];
const TAGS = 'abcdefghijklmnopqrstuvwxyz';
const round = (value: number): number => Math.round(value * 10) / 10;

/** What a part stands on: every ground behind it, laid over each other up to the first opaque one. */
function groundOf(element: Element): Rgba {
	const view = element.ownerDocument.defaultView;
	const layers: Rgba[] = [];

	for (
		let node: Element | null = element;
		node !== null && view !== null;
		node = node.parentElement
	) {
		const colour = readColor(view.getComputedStyle(node).backgroundColor);
		if (colour === null || colour[3] === 0) continue;

		layers.push(colour);
		if (colour[3] === 1) break;
	}

	return layers.reduceRight((under, layer) => over(layer, under), WHITE);
}

/** What is drawn rather than set: a shape takes its ink from `fill` and `stroke`, never `color`. */
const SHAPES = ['rect', 'path', 'circle', 'ellipse', 'line', 'polygon', 'polyline'];

/**
 * The ink a part shows. An empty box shows its placeholder, so that is the ink read on it; a drawn
 * shape shows its fill, or its stroke where it has none.
 */
function inkOf(element: Element, view: Window): string {
	if (SHAPES.includes(element.tagName.toLowerCase())) {
		const style = view.getComputedStyle(element);
		const paint = [style.fill, style.stroke].find(
			value => typeof value === 'string' && readColor(value) !== null
		);
		if (paint !== undefined) return paint;
	}

	const placeholder =
		element.hasAttribute('placeholder') && 'value' in element && element.value === '';

	return view.getComputedStyle(element, placeholder ? '::placeholder' : undefined).color;
}

/**
 * Reads the named parts of a specimen off the live page.
 *
 * Nothing here is typed in by anyone: a box is the element's, a colour is the one the page
 * computed, and the contrast is worked out from those. It is what replaces a drawing of the parts,
 * and it is read the same way whoever asks, from a frame on the bench or from a terminal.
 */
export function measureParts(
	root: Element,
	parts: readonly { name: string; select: string }[]
): PartReading[] {
	const view = root.ownerDocument.defaultView;
	if (view === null) return [];

	const origin = root.getBoundingClientRect();
	const readings: PartReading[] = [];

	for (const part of parts) {
		const element = root.querySelector(part.select);
		if (element === null) continue;

		const box = element.getBoundingClientRect();
		const style = view.getComputedStyle(element);
		const ground = groundOf(element);
		const ink = over(readColor(inkOf(element, view)) ?? [0, 0, 0, 1], ground);
		const pad = [style.paddingTop, style.paddingRight, style.paddingBottom, style.paddingLeft].map(
			side => Math.round(parseFloat(side) || 0)
		) as [number, number, number, number];
		const edge = parseFloat(style.borderTopWidth) || 0;

		readings.push({
			name: part.name,
			tag: TAGS[readings.length] ?? '?',
			x: round(box.left - origin.left),
			y: round(box.top - origin.top),
			width: round(box.width),
			height: round(box.height),
			padding: pad.some(side => side > 0) ? pad : null,
			type:
				element.tagName.toLowerCase() === 'svg' || SHAPES.includes(element.tagName.toLowerCase())
					? null
					: `${String(parseFloat(style.fontSize))}px, ${style.fontWeight}`,
			edge: edge > 0 ? `${String(edge)}px` : null,
			ink: toHex(ink),
			ground: toHex(ground),
			contrast: Math.round(contrastOf(ink, ground) * 100) / 100,
		});
	}

	return readings;
}
