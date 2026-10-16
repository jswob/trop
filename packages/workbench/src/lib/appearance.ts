/**
 * The appearance a frame is drawn in.
 *
 * It is an option of the stage and lives in a frame's address. The chrome's own light and dark is
 * `bench/appearance`, a preference of the person looking. How an appearance is put on a document
 * is the app's to say (`lib/setup`).
 */
export type Appearance = 'dark' | 'light';
