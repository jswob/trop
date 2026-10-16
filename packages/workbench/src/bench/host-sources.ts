import spend from 'virtual:workbench/spend';

/**
 * The app's source, as text, for counting which files spend a token.
 *
 * Apart from `host.ts` because it is every file of the app: only the tokens page pays for it.
 */
export const HOST_SOURCES: Record<string, string> = spend;
