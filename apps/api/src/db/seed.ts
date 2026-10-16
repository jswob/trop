import { db } from './connection';
import { seedDatabase } from './seed-core';
import { ALSO_CLEAR, SEED_DATA_ROOT, SEED_TABLES, seedFallback } from './seed-tables';

seedDatabase(db, {
	root: SEED_DATA_ROOT,
	tables: SEED_TABLES,
	alsoClear: ALSO_CLEAR,
	fallback: seedFallback,
})
	.then(() => {
		console.log('\nDone!');
		process.exit(0);
	})
	.catch(error => {
		console.error('Seed failed:', error);
		process.exit(1);
	});
