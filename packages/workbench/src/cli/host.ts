import type { ResolvedConfig } from '../config';

/**
 * The app a command is about, for every verb to read.
 *
 * `main` loads the config once and puts it here before a verb runs. A verb asks for it when it
 * runs and never when its module loads, so importing a verb reads no file and a test can put a
 * config of its own here first.
 */
let bound: ResolvedConfig | undefined;

/** Binds the commands that follow to one app. */
export function setHost(config: ResolvedConfig | undefined): void {
	bound = config;
}

/** The app's config. Throws when no command has loaded one. */
export function host(): ResolvedConfig {
	if (bound === undefined) {
		throw new Error('no workbench config is loaded — run this through the workbench command');
	}

	return bound;
}
