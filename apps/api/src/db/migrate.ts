import { migrate } from 'drizzle-orm/postgres-js/migrator';
import { logger } from '../utils/logger';
import { db } from './connection';

logger.info('Running migrations...');

await migrate(db, { migrationsFolder: './drizzle' });

logger.info('Migrations completed!');
process.exit(0);
