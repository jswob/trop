import type { ReactNode } from 'react';

import { setup } from '../lib/bound';
import type { ChromeKind } from '../lib/subject';

/**
 * The frame an entry is reviewed in: `bare`, for places and prototypes that draw their own
 * chrome.
 *
 * It stands on the app's ground and in its ink, not the workbench's: a part that sets no ink of
 * its own should inherit the system's, in either appearance, rather than the chrome's (76 found a
 * modal doing exactly that). What that ground is, and how tall, is the app's to say
 * (`setup.Ground`).
 */
export function SubjectFrame({ children }: { chrome: ChromeKind; children: ReactNode }) {
	return <setup.Ground>{children}</setup.Ground>;
}
