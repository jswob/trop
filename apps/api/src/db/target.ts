/**
 * This checkout's databases. Every script, config and test that opens one asks here (see
 * `createDbTargets`); only the default names are the app's own.
 */

import { createDbTargets } from '@jswob/drizzle-utils';

export type { DbTarget, DbTargetName } from '@jswob/drizzle-utils';

export const { dbTarget, selectedDbTarget } = createDbTargets({
	app: 'app_db',
	test: 'app_test_db',
	e2e: 'app_e2e_db',
});
