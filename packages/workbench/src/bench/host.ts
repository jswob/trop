/**
 * Everything the bench takes from the app's config, in one module: what the app is called,
 * the stylesheet that declares its tokens as text, and the two tokens a specimen is drawn on.
 *
 * The bench itself knows nothing about the app: not its tokens, not its face, not where its source
 * lives. The server writes this module from the config (`vite.ts`).
 */
export { HOST } from 'virtual:workbench/host';
