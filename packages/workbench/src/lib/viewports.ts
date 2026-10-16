/**
 * The widths a subject can be reviewed at. The app declares no breakpoints, so
 * these change the space a layout is given rather than which CSS applies.
 * Heights are the matching device heights, so a frame reads as a screen.
 */
export const VIEWPORTS = {
	mobile: { label: 'Mobile', width: 390, height: 844 },
	tablet: { label: 'Tablet', width: 834, height: 1112 },
	desktop: { label: 'Desktop', width: 1440, height: 900 },
} as const;

export const VIEWPORT_NAMES = [
	'mobile',
	'tablet',
	'desktop',
] as const satisfies readonly (keyof typeof VIEWPORTS)[];

export type ViewportName = (typeof VIEWPORT_NAMES)[number];

export const DEFAULT_VIEWPORT: ViewportName = 'desktop';
