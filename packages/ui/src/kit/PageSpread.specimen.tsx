import { specimen } from '@jswob/design-workbench';
import { Users } from 'lucide-react';
import { useState } from 'react';
import type { ReactNode } from 'react';
import { PageColumn, PageSection, PageSpread, PageUnit } from '@app/ui/kit';
import type { NestedPage, PageHeading, SectionBreak } from '@app/ui/kit';
import { pageHeadIn, useKeyboardKept } from '@app/ui/kit/keyboard';
import { SpecimenPage } from './SplitShell.specimen';

const HEAD: PageHeading = {
	title: 'What is on the shelf',
	lede: 'Twelve of them, what state each is in, and who has the ones that are out.',
};

/**
 * The page as the frame sets it: a spread from 1024px, and below it the same head in one column
 * with the slug's room at the foot. The spread is never drawn under the width it needs.
 */
function Page({
	head = HEAD,
	flow,
	nested = null,
	children,
}: {
	head?: PageHeading;
	flow: string;
	nested?: NestedPage | null;
	children: ReactNode;
}) {
	return (
		<>
			<div className="hidden min-h-0 flex-1 lg:flex">
				<PageSpread head={head} flow={flow} nested={nested}>
					{children}
				</PageSpread>
			</div>
			<div className="flex min-h-0 flex-1 px-ui-gutter lg:hidden">
				<PageColumn head={head}>{children}</PageColumn>
			</div>
		</>
	);
}

/** A section with nothing to divide: one block, at the default policy. */
function Lines({ title, aside, lines }: { title: string; aside?: string; lines: string[] }) {
	return (
		<PageSection title={title} aside={aside}>
			{lines.map(line => (
				<p key={line} className="pt-3 text-ui-body leading-relaxed text-ui-ink-2">
					{line}
				</p>
			))}
		</PageSection>
	);
}

const opening = (
	<Lines
		title="How it stands"
		aside="12 of them"
		lines={[
			'The order is the order they stand in, because that is the order a hand reaches in.',
			'A book that is out keeps its place, and the gap is the fact worth seeing.',
		]}
	/>
);

const closing = (
	<Lines
		title="What comes next"
		lines={[
			'Two are promised for the winter, and neither has a place on the shelf yet.',
			'A book is given its place when it arrives, not when it is ordered.',
		]}
	/>
);

/** A section with a grain: ten entries a reader needs kept whole. */
const YEAR: { month: string; line: string }[] = [
	{ month: 'January', line: 'Two finished, and one given up at page 40.' },
	{ month: 'February', line: 'Nothing finished. Three started.' },
	{ month: 'March', line: 'The long one, at last.' },
	{ month: 'April', line: 'Two lent, and one of them came back.' },
	{ month: 'May', line: 'A month of rereading.' },
	{ month: 'June', line: 'One finished on a train, in a day.' },
	{ month: 'July', line: 'Away. The shelf did not move.' },
	{ month: 'August', line: 'Four short ones.' },
	{ month: 'September', line: 'One bought, one borrowed, neither opened.' },
	{ month: 'October', line: 'The borrowed one, read first.' },
];

function Year({ breaks }: { breaks: SectionBreak }) {
	return (
		<PageSection title="The year so far" aside={`${String(YEAR.length)} months`} breaks={breaks}>
			{YEAR.map(entry => (
				<PageUnit key={entry.month}>
					<div className="pb-3">
						<span className="block pt-3 text-ui-lead text-ui-ink">{entry.month}</span>
						<span className="block text-ui-body text-ui-ink-3">{entry.line}</span>
					</div>
				</PageUnit>
			))}
		</PageSection>
	);
}

/**
 * Lenders, with one lender opened inside it. The way back closes her and leaves the page, and the
 * keyboard lands on the page's head, as the frame lands it for a press that takes its own control.
 */
function Nested() {
	const [inside, setInside] = useState(true);
	useKeyboardKept(() => pageHeadIn(document));
	const rosa: NestedPage = {
		id: 'rosa',
		name: 'Rosa Okafor',
		page: 'Lenders',
		icon: Users,
		onBack: () => setInside(false),
	};

	return (
		<Page
			flow={inside ? rosa.id : 'lenders'}
			nested={inside ? rosa : null}
			head={
				inside
					? { title: rosa.name, lede: 'What she has out, and since when.', cut: true }
					: { title: 'Lenders', lede: 'Who has what, and since when.' }
			}
		>
			<SpecimenPage />
		</Page>
	);
}

export default specimen(PageSpread, {
	kit: 'page-spread',
	is: 'A page set as a spread: one flow, in as many columns of the measure as the glass takes.',
	stands: 'screen',
	states: {
		spread: {
			label: 'A second column',
			note: 'The flow ran past the foot, so a second column started.',
			draw: () => (
				<Page flow="spread">
					<SpecimenPage />
				</Page>
			),
		},
		short: {
			label: 'Shorter than the glass',
			note: 'One column, and no rule: there is nothing to stand between.',
			draw: () => <Page flow="short">{opening}</Page>,
		},
		long: {
			label: 'Longer than its columns',
			note: 'Every column grows together, and the page scrolls.',
			draw: () => (
				<Page flow="long">
					<SpecimenPage />
					{opening}
					<Year breaks="units" />
					{closing}
				</Page>
			),
		},
		whole: {
			label: 'A section moved whole',
			note: 'It will not fit under the first, so it moves entire.',
			draw: () => (
				<Page flow="whole">
					{opening}
					<Year breaks="whole" />
				</Page>
			),
		},
		units: {
			label: 'A section cut between units',
			note: 'The same flow: the rule falls between two entries.',
			draw: () => (
				<Page flow="units">
					{opening}
					<Year breaks="units" />
				</Page>
			),
		},
		nested: {
			label: 'A nested page',
			note: 'One 48px step in: the page’s glyph back, and its thread.',
			draw: () => <Nested />,
		},
	},
	props: {
		head: { is: 'What the page calls itself, and the one line under it.' },
		flow: { is: 'Which page this is: a token that changes when the destination does.' },
		edge: {
			is: 'Which side the scroll thumb rides: the rail’s side, wherever the rail is.',
			options: { left: true, right: true },
		},
		className: { is: 'The room it takes in its row. All of it, unless set.' },
		nested: { is: 'A record opened inside the page. Given, the flow is set one 48px step in.' },
		children: { is: 'The flow: the page’s sections, in order.' },
	},
	holds: [
		'A column is set at the measure, 32rem, and the block at exactly as many as fit.',
		'One 32px margin is spent on every edge: beside every column and rule, above and below.',
		'The first column fills to the foot of the glass, and a second starts with what is left.',
		'The head is the first block of the flow, drawn the same however many columns there are.',
		'A section says for itself what the rule may cut: all of it, or between its named units.',
		'Below 1024px it is PageColumn: the same head, one column, the slug’s room at the foot.',
	],
	refuses: [
		'Balanced columns: a four-row page was cut into two two-row columns over an empty glass.',
		'A column count as a setting, or a mode that switches into two.',
		'Columns marching off the right-hand edge into a sideways scroll.',
		'A break policy written over the contents by tag name.',
		'A section that breaks and protects nothing inside it.',
		'A head spanning the columns, or a gutter on the scroller.',
	],
});
