import type { ComponentType, ReactNode } from 'react';

import type { Specimen } from '../bench/specimen';

import type { Appearance } from './appearance';
import type { Subject } from './subject';

/**
 * The browser's half of the binding between the workbench and the app it draws.
 *
 * An app is bound by two files. `workbench.config.ts` is the terminal's half: plain data, naming
 * files. `workbench.setup.tsx` is this one: the app's own modules, handed over. The workbench
 * imports nothing of the app's by name, so what it draws and what it draws it in are whatever this
 * file gives it.
 */

/** One of the app's destinations: a heading the places under it are listed beneath. */
export type Section = {
	id: string;
	label: string;
};

/** A place the app has drawn, and the section it is listed under. `null` is the ground. */
export type PlaceSubject = Subject & { dest: string | null };

/**
 * One component of the kit, as its page knows it.
 *
 * It declares its files and the names it exports. **Who calls it is counted from the source**,
 * never listed here (`lib/kit-callers`): a hand-kept list is the one thing that can quietly stop
 * being true.
 */
export type KitEntry = {
	/** URL-safe; it is the `$component` path param. */
	id: string;
	name: string;
	/** Its own files, as the source scan names them. Never counted as callers of itself. */
	files: string[];
	/** The exported names to look for when counting callers. */
	symbols: string[];
	/** What it is, in one line. */
	line: string;
	/** The round that settled it, if one did. */
	settled?: string;
};

export type WorkbenchSetup = {
	/**
	 * What every page of the workbench is wrapped in: the providers the app's components read.
	 *
	 * It is drawn once, around the chrome and around a frame alike, and never around a board.
	 */
	Wrap: ComponentType<{ children: ReactNode }>;
	/**
	 * What a frame stands on: the app's ground and ink, at the height of the window.
	 *
	 * A part that sets no ink of its own inherits the app's from here, in either appearance.
	 */
	Ground: ComponentType<{ children: ReactNode }>;
	/** Puts an appearance on the document, as the app's own boot does. Called while rendering. */
	applyAppearance: (appearance: Appearance) => void;
	/** The app's destinations, in the order its own navigation lists them. */
	sections: Section[];
	/** What the sidebar calls the ground, the one place under no section. Its title when left out. */
	groundLabel?: string;
	/** Every place the app has drawn. */
	places: PlaceSubject[];
	/** What is unsettled. */
	protos: Subject[];
	kit: {
		/** The catalogue, in the order the sidebar lists it. */
		entries: KitEntry[];
		/** Each component's page. */
		specimens: Specimen[];
		/** Every primitive on one sheet. */
		stress: Subject;
		/** The specimens' states as frames, so each has an address. */
		subjects: Subject[];
	};
};

/**
 * Every subject a setup stages, across its spaces. Ids are unique across all of them, since a
 * frame's address names a subject by id alone.
 */
export function subjectsOf(setup: WorkbenchSetup): Subject[] {
	return [...setup.places, ...setup.protos, setup.kit.stress, ...setup.kit.subjects];
}

/** The subjects that have settled: every one but the candidates still being argued about. */
export function settledSubjectsOf(setup: WorkbenchSetup): Subject[] {
	return subjectsOf(setup).filter(subject => !setup.protos.includes(subject));
}

/** Types a setup where it is written. It returns what it is given. */
export function defineSetup(setup: WorkbenchSetup): WorkbenchSetup {
	return setup;
}
