import type { Subject } from '@jswob/design-workbench';
import { specimenOf } from '@jswob/design-workbench';
import { BookText, Compass, Hash, Settings2, Sparkles, Users } from 'lucide-react';
import { PageSection, SplitShell, useSplit } from '@app/ui/kit';
import type { PageHeading, SplitDest, SplitWhere } from '@app/ui/kit';

/**
 * `kit/SplitShell`, drawn on content that is not the app.
 *
 * The specimen is deliberately **not** any of the app's own screens. A kit component's specimen
 * that needs one screen's data to render is a screen's component wearing a kit's name, and the
 * first thing this page has to prove is that the frame is free of whatever errand it is drawn for.
 *
 * So the ground here is a run of ruled rows, the pages are set prose, and the register is six
 * destinations of a bookshelf. Everything visible is the shell: the split, the rail on it, the
 * page margins, the spread's column arithmetic, the slug below `lg`.
 *
 * The page is long enough on purpose. With the ground collapsed at 1440 the flow runs past the foot
 * of the first column, so the specimen shows a second one arriving — which is the thing that cannot
 * be demonstrated with three lines of placeholder text.
 */

const DEST: SplitDest[] = [
	{
		id: 'reading',
		label: 'Reading',
		icon: BookText,
		fact: '3 open',
		blurb: 'What is part-read, and where each one was left.',
		group: 'shelf',
	},
	{
		id: 'finding',
		label: 'Finding',
		icon: Compass,
		fact: 'idle',
		blurb: 'A search over everything on the shelf, and what it turned up.',
		group: 'shelf',
	},
	{
		id: 'tags',
		label: 'Tags',
		icon: Hash,
		fact: '18',
		blurb: 'How the shelf is cut up, and what each cut holds.',
		group: 'shelf',
	},
	{
		id: 'lenders',
		label: 'Lenders',
		icon: Users,
		fact: '7',
		blurb: 'Who has what, and since when.',
		group: 'people',
	},
	{
		id: 'notes',
		label: 'Notes',
		icon: Sparkles,
		fact: '42',
		blurb: 'Everything written in a margin, gathered in one place.',
		group: 'people',
	},
	{
		id: 'settings',
		label: 'Settings',
		icon: Settings2,
		fact: '',
		blurb: 'The account, and what the shelf assumes when nobody says otherwise.',
		group: 'you',
	},
];

const RESTING = 'reading';

/** The ground: ruled rows at a real pitch, so the ledger has a rhythm to be read against. */
const SHELF: { title: string; hand: string; at: string }[] = [
	{ title: 'The Making of the English Landscape', hand: 'Hoskins', at: 'p. 140' },
	{ title: 'A Pattern Language', hand: 'Alexander', at: 'p. 612' },
	{ title: 'The Elements of Typographic Style', hand: 'Bringhurst', at: 'p. 44' },
	{ title: 'Seeing Like a State', hand: 'Scott', at: 'p. 88' },
	{ title: 'The Timeless Way of Building', hand: 'Alexander', at: 'returned' },
	{ title: 'Ways of Seeing', hand: 'Berger', at: 'p. 7' },
	{ title: 'The Death and Life of Great American Cities', hand: 'Jacobs', at: 'p. 233' },
	{ title: 'Understanding Comics', hand: 'McCloud', at: 'lent — Rosa' },
	{ title: 'The Visual Display of Quantitative Information', hand: 'Tufte', at: 'p. 51' },
	{ title: 'Notes on the Synthesis of Form', hand: 'Alexander', at: 'p. 19' },
	{ title: 'The Craftsman', hand: 'Sennett', at: 'unopened' },
	{ title: 'How Buildings Learn', hand: 'Brand', at: 'p. 176' },
];

function Shelf() {
	return (
		<div className="max-w-[27rem]">
			{SHELF.map(row => (
				<div
					key={row.title}
					className="flex items-baseline gap-3 border-b border-ui-rule py-2 last:border-b-0"
				>
					<span className="min-w-0 flex-1 truncate text-ui-body text-ui-ink">{row.title}</span>
					<span className="shrink-0 text-ui-body text-ui-ink-3">{row.hand}</span>
					<span className="w-20 shrink-0 text-right text-ui-body text-ui-ink-2">{row.at}</span>
				</div>
			))}
		</div>
	);
}

/**
 * Set prose at the measure, in sections, because a section is what the spread breaks between.
 *
 * Exported as `SpecimenPage` so the spread's specimen draws the **same flow** its shell does. Two
 * specimen texts would make the two pages look like two components, and the whole claim of the
 * spread is that the frame hands it a document and asks no questions about it.
 *
 * Every part is a `kit/PageSection` at the default `whole`: prose has no grain to declare, so there
 * are no units in it and the rule falls between the sections.
 */
const PAGE_BODY: { head: string; body: string[] }[] = [
	{
		head: 'What is on it',
		body: [
			'Twelve of them are part-read, which the shelf reports as a position rather than as a percentage: a page number is a place you can go back to, and a percentage is a number about a number.',
			'Nothing here is sorted by when it was added. The order is the order they stand in, because that is the order a hand reaches in.',
		],
	},
	{
		head: 'What is lent',
		body: [
			'A lent copy keeps its row and says who has it. It is not moved to a second list — a shelf with a hole in it is the fact worth seeing, and a list of absences is a list nobody opens.',
			'The date is left off on purpose. Whoever has it knows, and the row is not the place to start a negotiation.',
		],
	},
	{
		head: 'What is written in them',
		body: [
			'Margin notes are gathered under Notes rather than shown here, because a note belongs to a page and a page is not the shelf. What the shelf says is that there are some.',
			'A note that has been written twice about the same page is one note. The shelf does not count restatements, and neither should anything reading from it.',
		],
	},
	{
		head: 'What it refuses',
		body: [
			'No cover art. A grid of covers is a grid of pictures, and the thing being scanned is a run of titles set in one face at one size.',
			'No star rating. The shelf records what happened to a book, not what was thought of it, and the two get confused the moment they are drawn on one row.',
			'No progress bar beside the page number. The number is already the measure; a bar is the same fact drawn a second time, in a shape that implies a finish line the reader never agreed to.',
		],
	},
];

export function SpecimenPage() {
	return (
		<>
			{PAGE_BODY.map(part => (
				<PageSection key={part.head} title={part.head}>
					{part.body.map(line => (
						<p key={line} className="pt-3 text-ui-body leading-relaxed text-ui-ink-2">
							{line}
						</p>
					))}
				</PageSection>
			))}
		</>
	);
}

function headOf(id: string | null): PageHeading | null {
	const dest = DEST.find(entry => entry.id === id);
	if (dest === undefined) return null;
	return { title: dest.label, lede: dest.blurb };
}

/**
 * The shell, seeded and live.
 *
 * Every cell works, which is most of what there is to check about a frame: the logo collapses and
 * restores the ground, a destination's cell opens and closes its page, and the slug raises the
 * register below `lg`. `useSplit` supplies all of it — this specimen is the smallest honest caller
 * the shell has, and if it needed anything the views need, the shell would be wrong.
 */
function Specimen({ seed }: { seed: SplitWhere }) {
	const split = useSplit(seed, { resting: RESTING });
	const head = headOf(split.where.dest);
	const dest = DEST.find(entry => entry.id === split.where.dest);

	return (
		<SplitShell
			destinations={DEST}
			where={split.where}
			home="The shelf"
			reading={
				dest === undefined
					? { place: 'The shelf', fact: '12 on it' }
					: { place: dest.label, fact: dest.fact ?? '' }
			}
			head={head}
			flow={split.where.dest ?? 'ground'}
			ledger={<Shelf />}
			onGo={split.go}
			onLogo={split.toggleLedger}
			onSlug={split.slug}
			onHome={split.home}
		>
			<SpecimenPage />
		</SplitShell>
	);
}

function frame(seed: SplitWhere) {
	return () => <Specimen seed={seed} />;
}

/**
 * The shell's own frames, registered so each one has a URL.
 *
 * A frame needs a viewport to be a frame — `h-screen` measures the window — so a specimen cannot
 * be drawn inline the way `MonthDay` can. Each of these is an address the bench loads in a frame
 * at true pixel width, which is the same machinery `/views` uses and the same address a screenshot
 * would use.
 */
export const splitShellFrames: Subject = {
	id: 'kit-split-shell',
	title: 'SplitShell — specimens',
	description: 'The frame: the ledger on the left, one destination on the right, the rail between.',
	chrome: 'bare',
	variants: [
		{
			id: 'both',
			label: 'Both halves',
			note: 'The ground on the left, a page on the right.',
			render: frame({ ledger: true, dest: 'reading', menu: false }),
		},
		{
			id: 'collapsed',
			label: 'The whole glass',
			note: 'The ground folded away; the page has the glass.',
			render: frame({ ledger: false, dest: 'reading', menu: false }),
		},
		{
			id: 'ground',
			label: 'Nothing open',
			note: 'No page: the rail stands at the right edge.',
			render: frame({ ledger: true, dest: null, menu: false }),
		},
		{
			id: 'undrawn',
			label: 'Another destination',
			note: 'A page whose head is there before its contents.',
			render: frame({ ledger: true, dest: 'tags', menu: false }),
		},
		{
			id: 'menu',
			label: 'The register, raised',
			note: 'Below lg: the destinations raised from the slug.',
			render: frame({ ledger: true, dest: null, menu: true }),
		},
	],
};

export default specimenOf(splitShellFrames, {
	kit: 'split-shell',
	props: {
		destinations: 'The cells of the rail.',
		where: 'Which halves are open. Controlled: it takes the state, it does not own it.',
		user: 'Who is signed in, for the account cell.',
		reading: 'What each cell of the rail prints at rest.',
		home: 'What the ground is called.',
		head: 'The open page’s own name and line.',
		flow: 'How the page’s columns are set.',
		centred: 'A page that stands in the middle of the glass.',
		ledger: 'What the left half draws.',
		overlay: 'What is laid over both halves.',
		held: 'What the open destination is holding, and would lose.',
		children: 'The open page’s body.',
	},
	holds: [
		'One rail, on the split, with three positions.',
		'The screen always keeps at least one half.',
		'Neither column is titled.',
		'Both halves spend the same page margin.',
	],
	refuses: [
		'A second rail.',
		'A header band: its 44px goes back to the content.',
		'A control that closes both halves.',
		'A wipe when a page gives width back.',
	],
});
