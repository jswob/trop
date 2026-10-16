/**
 * Schema Changes Check
 *
 * Checks whether there are uncommitted database schema changes by running
 * drizzle-kit generate and using git to detect new files.
 *
 * Used by pre-commit hooks to ensure all schema changes have corresponding
 * migration files before committing. Fails (blocking the commit) when they do not.
 */

import { execSync } from 'child_process';
import { posix, relative } from 'path';
import type { ResolvedConfig } from '../config';
import { DRIZZLE_KIT } from '../drizzle-kit';

interface Paths {
	/** The repository's top level, where git's paths start. */
	repo: string;
	/** The schema folders, from the repository's top level. */
	schema: string[];
	/** The migrations folder, from the repository's top level. */
	migrations: string;
	/** The migrations folder, from the app's folder. */
	localMigrations: string;
}

function checkStagedSchemaDependencies({ repo, schema, migrations }: Paths): boolean {
	try {
		const stagedFiles = execSync('git diff --cached --name-only', {
			stdio: 'pipe',
			cwd: repo,
		})
			.toString()
			.split('\n')
			.filter(Boolean);

		const stagedSchemaDeps = stagedFiles.filter(file => schema.some(dep => file.startsWith(dep)));

		const stagedDrizzleFiles = stagedFiles.filter(file => file.startsWith(migrations));

		if (stagedSchemaDeps.length === 0 && stagedDrizzleFiles.length === 0) {
			console.log('ℹ️  No schema-related files staged, skipping schema check');
			return false;
		}

		console.log('📋 Found staged schema-related files:');
		if (stagedSchemaDeps.length > 0) {
			console.log('  Schema dependencies:');
			stagedSchemaDeps.forEach(file => console.log(`    - ${file}`));
		}
		if (stagedDrizzleFiles.length > 0) {
			console.log('  Migration files:');
			stagedDrizzleFiles.forEach(file => console.log(`    - ${file}`));
		}

		return true;
	} catch (error) {
		console.error('❌ Failed to check staged files:', String(error));
		return true; // Proceed with check on error to be safe
	}
}

function checkAllDependenciesStaged({ repo, schema }: Paths): boolean {
	try {
		const allStatus = execSync('git status --porcelain', {
			stdio: 'pipe',
			cwd: repo,
		}).toString();

		const unstagedSchemaDeps: string[] = [];

		allStatus.split('\n').forEach(line => {
			if (!line.trim()) return;

			const status = line.substring(0, 2);
			const file = line.substring(3);

			// Check for unstaged changes in schema dependencies
			// Status codes: ' M' (modified unstaged), 'MM' (modified staged and unstaged)
			if ((status[1] === 'M' || status === ' M') && schema.some(dep => file.startsWith(dep))) {
				unstagedSchemaDeps.push(file);
			}
		});

		if (unstagedSchemaDeps.length > 0) {
			console.error('❌ Found unstaged changes in schema dependencies!');
			console.error('📄 These schema files have unstaged changes:');
			unstagedSchemaDeps.forEach(file => {
				console.error(`  - ${file}`);
			});
			console.error('');
			console.error('To fix this:');
			console.error('  - Stage all schema changes: git add <files>');
			console.error('  - Or revert unstaged changes: git checkout -- <files>');
			console.error('');
			console.error('Schema check requires ALL dependencies to be staged to ensure consistency.');
			return false;
		}

		return true;
	} catch (error) {
		console.error('❌ Failed to check dependency staging:', String(error));
		return false;
	}
}

function checkUnstagedDrizzleChanges({ localMigrations }: Paths): boolean {
	try {
		const statusOutput = execSync(`git status --porcelain ${localMigrations}`, {
			stdio: 'pipe',
			cwd: process.cwd(),
		})
			.toString()
			.trim();

		if (!statusOutput) {
			return true;
		}

		// Parse status output to find truly unstaged changes
		const unstagedLines: string[] = [];

		statusOutput.split('\n').forEach(line => {
			if (!line.trim()) return;

			const status = line.substring(0, 2);

			// Check for unstaged changes
			// ' M' = modified unstaged, 'MM' = modified staged and unstaged, '??' = untracked
			// We ignore 'A ' (staged for addition) and 'M ' (staged for modification)
			if (status[1] === 'M' || status === ' M' || status === '??') {
				unstagedLines.push(line);
			}
		});

		if (unstagedLines.length > 0) {
			console.error(`❌ Found unstaged changes in ${localMigrations}!`);
			console.error('📄 Please handle these changes first:');
			unstagedLines.forEach(line => {
				console.error(`  ${line}`);
			});
			console.error('');
			console.error('To fix this:');
			console.error(`  - Stage changes: git add ${localMigrations}`);
			console.error(
				`  - Or revert changes: git checkout -- ${localMigrations} && git clean -fd ${localMigrations}`
			);
			return false;
		}

		return true;
	} catch (error) {
		console.error('❌ Failed to check unstaged drizzle changes:', String(error));
		return false;
	}
}

function checkSchemaChanges({ localMigrations }: Paths): boolean {
	console.log('🔍 Checking for uncommitted schema changes...');

	try {
		console.log('Running drizzle-kit generate...');
		execSync(`${DRIZZLE_KIT} generate`, {
			stdio: 'pipe',
			cwd: process.cwd(),
		});

		console.log('Checking for new migration files...');
		const gitStatus = execSync(`git status --porcelain ${localMigrations}`, {
			stdio: 'pipe',
			cwd: process.cwd(),
		}).toString();

		const newFiles = gitStatus.split('\n').filter(line => line.startsWith('??'));

		if (newFiles.length > 0) {
			console.error('❌ Uncommitted schema changes detected!');
			console.error(`📄 ${newFiles.length} new migration file(s) were generated:`);
			newFiles.forEach(file => {
				console.error(`  - ${file.substring(3)}`);
			});
			console.error('');
			console.error('To fix this:');
			console.error(`  1. Add migrations to your commit: git add ${localMigrations}`);
			console.error('  2. Commit again');
			console.error('');
			console.error('Or if you want to develop without migrations:');
			console.error('  - Use: bun db:push for development');
			console.error('  - Only generate migrations when ready for production');

			console.log('🧹 Cleaning up generated migration files...');
			// Revert tracked file changes (like _journal.json)
			execSync(`git checkout -- ${localMigrations}`, {
				stdio: 'pipe',
				cwd: process.cwd(),
			});
			// Remove untracked files
			execSync(`git clean -fd ${localMigrations}`, {
				stdio: 'pipe',
				cwd: process.cwd(),
			});

			return true;
		}

		console.log('✅ No uncommitted schema changes detected');
		return false;
	} catch (error) {
		const errorOutput = String(error);
		if (errorOutput.includes('No schema changes')) {
			console.log('✅ No schema changes detected');
			return false;
		}

		console.error('❌ Failed to check schema changes:', errorOutput);

		console.log('🧹 Attempting to clean up any generated files...');
		try {
			execSync(`git checkout -- ${localMigrations}`, {
				stdio: 'pipe',
				cwd: process.cwd(),
			});
			execSync(`git clean -fd ${localMigrations}`, {
				stdio: 'pipe',
				cwd: process.cwd(),
			});
		} catch {
			// Ignore cleanup errors
		}

		return true;
	}
}

/**
 * Fail when the staged schema has no migration for it.
 * @param config - The app's config
 */
export function schemaCheck(config: ResolvedConfig): void {
	const repo = execSync('git rev-parse --show-toplevel', { stdio: 'pipe' }).toString().trim();
	const app = relative(repo, process.cwd());
	const paths: Paths = {
		repo,
		schema: config.schema.map(path => posix.join(app, path)),
		migrations: posix.join(app, config.migrations),
		localMigrations: config.migrations,
	};

	// Step 1: Check if schema dependencies or drizzle files are staged
	if (!checkStagedSchemaDependencies(paths)) return;

	// Step 2: Check if ALL modified schema dependencies are staged
	if (!checkAllDependenciesStaged(paths)) {
		throw new Error('Schema check failed: schema files have unstaged changes');
	}

	// Step 3: Check for unstaged drizzle changes
	if (!checkUnstagedDrizzleChanges(paths)) {
		throw new Error('Schema check failed: the migrations folder has unstaged changes');
	}

	// Step 4: Run generate and check for differences
	if (checkSchemaChanges(paths)) {
		throw new Error('Schema check failed: the schema has changes no migration covers');
	}

	console.log('🎉 Schema check passed');
}
