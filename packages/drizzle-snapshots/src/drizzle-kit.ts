/**
 * drizzle-kit, run by Bun instead of the Node its bin names. An app's drizzle config imports
 * packages that ship TypeScript source, and Node refuses to strip types from a file under
 * `node_modules`; Bun runs it wherever it lives.
 */
export const DRIZZLE_KIT = 'bun --bun drizzle-kit';
