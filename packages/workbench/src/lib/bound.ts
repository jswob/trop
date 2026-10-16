/**
 * The app this workbench is bound to: the one import the machinery has of it.
 *
 * The specifier is the server's (`vite.ts`), which resolves it to the setup file the app's config
 * names. Everything else reads the app through `setup`, so a module here names no file of the
 * app's.
 */
export { default as setup } from 'virtual:workbench/setup';
