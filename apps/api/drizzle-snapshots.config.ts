import { defineConfig } from '@jswob/drizzle-snapshots';
import { dbTarget } from './src/db/target';

export default defineConfig({
	dbTarget,
	migrate: 'src/db/migrate.ts',
	seed: 'src/db/seed.ts',
	/*
	 * Constraints Drizzle cannot declare (an exclusion constraint, say), which a push would drop:
	 * `db:push:safe` puts each back. None yet.
	 */
	rawConstraints: [],
});
