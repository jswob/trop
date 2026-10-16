import { declareChannel } from './declare-channel';
import type { DeclareStore as Store, FrameDeclared } from './declare-channel';

/**
 * **What a page declares to the app's frame**: the fields every frame reads
 * (`frame/declare-channel`). An app whose ground draws something of the open page adds those
 * fields here.
 */
export type PageDeclared = FrameDeclared;

/** Where the open page's declaration is kept between the page and the layout. */
export type DeclareStore = Store<PageDeclared>;

export const { createDeclareStore, DeclareProvider, useDeclared, useDeclare } =
	declareChannel<PageDeclared>();
