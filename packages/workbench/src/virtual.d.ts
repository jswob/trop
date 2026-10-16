/**
 * The modules the server writes or resolves for the bench (`vite.ts`): the app's two binding files
 * and what the bench reads of the app's source.
 */

declare module 'virtual:workbench/setup' {
	const setup: import('./lib/setup').WorkbenchSetup;
	export default setup;
}

declare module 'virtual:workbench/styles.css';

declare module 'virtual:workbench/host' {
	export const HOST: {
		/** What the app is called, beside the workbench's own name. */
		name: string;
		/** The stylesheet that declares the tokens, as text. */
		themeCss: string;
		/** The token a frame's ground is drawn in, which every colour's contrast is measured against. */
		ground: string;
		/** The token a line of specimen type is set in. */
		face: string;
	};
}

declare module 'virtual:workbench/sources' {
	const sources: Record<string, string>;
	export default sources;
}

declare module 'virtual:workbench/spend' {
	const sources: Record<string, string>;
	export default sources;
}
