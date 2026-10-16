import sources from 'virtual:workbench/sources';

/**
 * The source a design scan reads, as text: the design root's spaces, then the app's settled ones.
 *
 * The server reads the list with the function the verbs use (`sources.ts`), so a count printed on
 * a page and one printed in a terminal are counts of one list.
 */
export const DESIGN_SOURCES: Record<string, string> = sources;
