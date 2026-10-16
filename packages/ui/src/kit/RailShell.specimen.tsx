import { specimen } from '@jswob/design-workbench';
import {
	BookText,
	Check,
	CheckCheck,
	Compass,
	NotebookPen,
	Plus,
	Undo2,
	Users,
	X,
} from 'lucide-react';
import { useState } from 'react';
import { Button, Modal, PAGE_TOP, PageColumn, RailShell, SLUG_FLOOR, useHeld } from '@app/ui/kit';
import type { HeldSeed, RailEntry } from '@app/ui/kit';
import { SpecimenPage } from './SplitShell.specimen';

/** A shelf's own register, so the frame is drawn on content that is not the app. */
const REGISTER: (RailEntry & { lede: string })[] = [
	{
		id: 'reading',
		label: 'Reading',
		icon: BookText,
		fact: '3 open',
		lede: 'What is part-read, and where each one was left.',
	},
	{
		id: 'finding',
		label: 'Finding',
		icon: Compass,
		fact: 'idle',
		lede: 'A search over everything on the shelf.',
	},
	{
		id: 'lenders',
		label: 'Lenders',
		icon: Users,
		fact: '7',
		lede: 'Who has what, and since when.',
	},
	{
		id: 'notes',
		label: 'Notes',
		icon: NotebookPen,
		fact: '42',
		lede: 'Everything written in a margin, in one place.',
	},
];

const HOME = 'The shelf';

const ON_SHELF = 12;

/** The ground the slug floats over, with the margins the frame gives its own. */
const ground = (
	<div className={`${PAGE_TOP} ${SLUG_FLOOR} px-ui-gutter`}>
		{[
			'Twelve books stand on the shelf, in the order a hand reaches in.',
			'Three are part-read, and each says the page it was left on.',
			'Two are lent, and each says who has it.',
			'Forty-two notes are written in their margins.',
		].map(line => (
			<p
				key={line}
				className="border-b border-ui-rule py-2 text-ui-body text-ui-ink last:border-b-0"
			>
				{line}
			</p>
		))}
	</div>
);

/** What the Notes page is holding: a note being written, or the same note being saved. */
type Note = { fact: string; busy?: boolean };

const NOTE: Note = { fact: '2 unsaved' };

const SAVING: Note = { fact: 'Saving', busy: true };

/** One lender, opened inside Lenders. */
const ROSA = { id: 'rosa', name: 'Rosa Okafor', lede: 'What she has out, and since when.' };

/**
 * The shell, seeded and live: the frame's own hook owns where it is and what it holds, so every
 * state can be reached from another by pressing the slug.
 *
 * Lenders is the page with an errand, a detour and a record to open; Notes is the one that holds;
 * Finding is the one with a run. `running`, `picking` and `opened` seed those three.
 */
function Rail({
	seed,
	running = false,
	picking = false,
	opened = false,
}: {
	seed: HeldSeed<Note>;
	running?: boolean;
	picking?: boolean;
	opened?: boolean;
}) {
	const hand = useHeld(seed, { resting: 'reading' });
	const [lenders, setLenders] = useState(7);
	const [picked, setPicked] = useState<number | null>(picking ? 2 : null);
	const [inside, setInside] = useState(opened);

	const dest = REGISTER.find(entry => entry.id === hand.where.dest);
	const lending = dest?.id === 'lenders';
	const searching = running && dest?.id === 'finding';
	const record = lending && inside;
	const fact = lending ? String(lenders) : searching ? 'Searching' : (dest?.fact ?? '');

	return (
		<RailShell
			sections={REGISTER.map(entry =>
				entry.id === 'lenders' ? { ...entry, fact: String(lenders) } : entry
			)}
			section={dest?.id ?? ''}
			panels={[]}
			panel=""
			menu={hand.where.menu}
			phone={dest === undefined || hand.peeking || picked !== null ? 'list' : 'panel'}
			reading={
				dest === undefined
					? { place: HOME, fact: `${String(ON_SHELF)} on it` }
					: { place: dest.label, fact }
			}
			progress={searching ? 0.4 : undefined}
			home={HOME}
			onGo={hand.go}
			onSlug={hand.slug}
			onHome={hand.home}
			column={ground}
			columnClassName="lg:w-96 lg:shrink-0"
			held={
				hand.held === null
					? null
					: {
							name: 'A new note',
							fact: hand.held.fact,
							busy: hand.held.busy,
							peeking: hand.peeking,
							onPeek: hand.peek,
							icon: NotebookPen,
							cells: [
								{ id: 'discard', icon: Undo2, label: 'Discard the note', onPress: hand.stop },
								{ id: 'save', icon: Check, label: 'Save the note', onPress: hand.release },
							],
						}
			}
			detour={
				picked === null
					? null
					: {
							name: 'Lenders',
							icon: Users,
							fact:
								picked === 0 ? 'None picked' : `${String(picked)} of ${String(ON_SHELF)} picked`,
							onBack: () => setPicked(null),
							cells: [
								{
									id: 'all',
									icon: CheckCheck,
									label: 'Select all',
									onPress: () => setPicked(ON_SHELF),
								},
								{ id: 'none', icon: X, label: 'Clear all', onPress: () => setPicked(0) },
							],
						}
			}
			nested={
				record
					? {
							id: ROSA.id,
							name: ROSA.name,
							page: 'Lenders',
							icon: Users,
							onBack: () => setInside(false),
						}
					: null
			}
			errand={
				lending
					? { label: 'Add a lender', icon: Plus, onPress: () => setLenders(count => count + 1) }
					: null
			}
			overlay={
				hand.going === null ? null : (
					<Modal
						title="A new note"
						onClose={hand.stay}
						footer={
							<>
								<Button onClick={hand.discard}>Discard the note</Button>
								<Button weight="primary" onClick={hand.stay}>
									Keep writing
								</Button>
							</>
						}
					>
						<p className="flex items-start gap-1.5 text-ui-body leading-5 text-ui-red">
							<span aria-hidden>!</span>
							What is written in it will be lost.
						</p>
					</Modal>
				)
			}
		>
			{dest === undefined ? null : (
				<PageColumn
					head={
						record
							? { title: ROSA.name, lede: ROSA.lede, cut: true }
							: { title: dest.label, lede: dest.lede }
					}
					className="flex-1 lg:mx-0 lg:max-w-[34.5rem]"
				>
					<SpecimenPage />
				</PageColumn>
			)}
		</RailShell>
	);
}

const AT_REST = { ledger: true, menu: false, held: null };

export default specimen(RailShell, {
	kit: 'rail-shell',
	is: 'The frame below 1024px, where alone it is itself: one slug at the foot, and its register.',
	stands: 'screen',
	states: {
		ground: {
			label: 'On the ground',
			note: 'The mark, where you are, and one fact about it.',
			draw: () => <Rail seed={{ ...AT_REST, dest: null }} />,
		},
		page: {
			label: 'On a page',
			note: 'The arrow leaves. The rest of the bar raises the register.',
			draw: () => <Rail seed={{ ...AT_REST, dest: 'reading' }} />,
		},
		menu: {
			label: 'The register, raised',
			note: 'From the thumb up, each row with its fact. The page is dim.',
			draw: () => <Rail seed={{ ...AT_REST, dest: 'reading', menu: true }} />,
		},
		errand: {
			label: 'A page’s errand',
			note: 'One cell after the fact, under the thumb at every scroll.',
			draw: () => <Rail seed={{ ...AT_REST, dest: 'lenders' }} />,
		},
		running: {
			label: 'A run going',
			note: 'The top edge fills, and the work takes the fact’s slot.',
			draw: () => <Rail seed={{ ...AT_REST, dest: 'finding' }} running />,
		},
		held: {
			label: 'Holding',
			note: 'The hold’s row is the bar: the eye, one fact, a cell each.',
			draw: () => <Rail seed={{ ...AT_REST, dest: 'notes', held: NOTE }} />,
		},
		peeking: {
			label: 'Holding, at a peek',
			note: 'The ground on screen: the page’s glyph is the way back.',
			draw: () => <Rail seed={{ ...AT_REST, dest: 'notes', held: NOTE, peeking: true }} />,
		},
		busy: {
			label: 'Holding, its run going',
			note: 'The bed empties and the ink stays up. The fact says why.',
			draw: () => <Rail seed={{ ...AT_REST, dest: 'notes', held: SAVING }} />,
		},
		detour: {
			label: 'A detour',
			note: 'Sent to the ground for one thing: no bed, no guard.',
			draw: () => <Rail seed={{ ...AT_REST, dest: 'lenders' }} picking />,
		},
		nested: {
			label: 'A nested page',
			note: 'A level row on the bar: the record, and the glyph back.',
			draw: () => <Rail seed={{ ...AT_REST, dest: 'lenders' }} opened />,
		},
	},
	props: {
		sections: {
			is: 'The destinations: the register’s rows, and the left rail’s cells from 1024px.',
		},
		section: { is: 'Which destination is open, by id. Empty on the ground.', text: true },
		panels: { is: 'What a right rail would offer. None, and no right rail is drawn.' },
		panel: { is: 'Which panel is open, by id.', text: true },
		reading: { is: 'What the slug prints: where you are, and exactly one thing about it.' },
		progress: { is: 'How far through a run, 0 to 1. The slug’s top edge draws it.' },
		menu: { is: 'Whether the register is raised out of the slug.', toggle: true },
		phone: {
			is: 'Which of the two columns is on screen below 1024px.',
			options: { list: true, panel: true },
		},
		home: {
			is: 'What the ground is called. Given, a page’s bar grows the arrow back.',
			text: true,
		},
		onGo: { is: 'A destination picked, in a rail or in the raised register.' },
		onSlug: { is: 'The bar’s body tapped: raise or lower the register.' },
		onHome: { is: 'The arrow tapped. Always the ground, from any page.' },
		column: { is: 'The ground’s contents. The shell gives it the scroll and the rule.' },
		columnClassName: { is: 'The width the shell gives that column from 1024px.' },
		overlay: { is: 'A dialog over the whole frame.' },
		held: { is: 'What the open page holds and would lose. Its row replaces the bar.' },
		detour: { is: 'A page’s errand on the ground. Its row replaces the bar, with no bed.' },
		nested: { is: 'A record opened inside the page. The box grows a level row for it.' },
		errand: { is: 'The page’s own errand, as one cell on its resting bar.' },
		away: { is: 'The frame put away under a screen standing in its place.', toggle: true },
		children: { is: 'The open page’s contents.' },
	},
	holds: [
		'There is nothing across the top at all. The register is ordered from the thumb up.',
		'The foot prints where you are and exactly one thing about it, never two.',
		'The slug is inset by one gutter on every edge it comes near.',
		'Its box is ruled in ui-edge, never ui-rule, so it is told apart from what it floats over.',
		'The floor a column leaves is a sum: the gutter, the bar and one page margin, 94px.',
		'A raised register is lowered by tapping anywhere that is not it, or by Escape.',
	],
	refuses: [
		'A full-bleed foot bar, a navigation screen, or a foot divided into two equal targets.',
		'The mark as the way back. It stays on the ground, and an arrow does the leaving.',
		'The ground as a row in the raised register.',
		'A row marked active for a column that is not on screen.',
		'A bar half taken over by a hold. It is the resting bar or the hold’s row.',
		'A progress edge that is always there. It is drawn only while a run is going.',
	],
});
