import { specimen } from '@jswob/design-workbench';
import { Mark } from '@app/ui/kit';

export default specimen(Mark, {
	kit: 'mark',
	is: 'The app’s glyph, a placeholder: a filled square beside an open one.',
	states: {
		rail: {
			label: 'In the chrome',
			note: '34 by 16, as the rail draws it.',
			args: { size: 'rail' },
		},
		display: {
			label: 'Display',
			note: 'Three times the rail, on the one centred screen.',
			args: { size: 'display' },
		},
	},
	props: {
		size: {
			is: 'Which of its two sizes. Defaults to rail.',
			options: { rail: true, display: true },
		},
	},
	parts: {
		glyph: { select: 'svg', is: 'The whole mark, named Logo.' },
		square: { select: 'rect', is: 'The filled square: a square of ink.' },
		open: { select: 'rect[fill="none"]', is: 'The open square: an edge and no fill.' },
	},
	holds: [
		'A filled square beside an open one, until the app draws its own glyph in the same box.',
		'Squares and hard corners.',
		'Rail is the mark in the chrome, 34 by 16.',
		'Display is three times the rail, 102 by 48, the one place it is drawn large.',
	],
	refuses: [
		'A lock on the logo to say something is held.',
		'An empty rail holding only the mark, as the signed-out screen.',
		'Hue or a radius.',
	],
});
