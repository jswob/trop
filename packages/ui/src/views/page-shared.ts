/**
 * **What every record's page shares**: the measure a page sets itself at, and the words a row is
 * removed with, kept in one place so no two pages drift apart on either.
 */

/**
 * The measure the document is set at, whatever the column is given.
 *
 * 32rem is about 70 characters of the body face, which is inside the line length prose is
 * read at. It is also the measure the **page** is set at — `kit/PageSpread` sets each column of the
 * spread to exactly this, so the document and the column it is in are one number rather than two
 * that happen to agree. Any surplus past it is margin, and a rule drawn here stops where the content
 * stops — a rule running on past the last value is empty space, not structure.
 */
export const FORM_MEASURE = 'w-full max-w-[32rem]';

/** A row's removal, armed in place: the press that takes it, and the one that keeps it. */
export const REMOVE_TASK = 'Remove';
export const KEEP_TASK = 'Keep it';
