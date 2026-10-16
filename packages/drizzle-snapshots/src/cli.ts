#!/usr/bin/env bun
/**
 * `drizzle-snapshots <command>`, run from the app's folder with its environment loaded. Each
 * command prints what it does; a failure prints its reason and exits 1.
 */

import { clean } from './commands/clean';
import { e2eRestore } from './commands/e2e-restore';
import { e2eSetup } from './commands/e2e-setup';
import { e2eTeardown } from './commands/e2e-teardown';
import { list } from './commands/list';
import { push } from './commands/push';
import { restore } from './commands/restore';
import { schemaCheck } from './commands/schema-check';
import { snapshot } from './commands/snapshot';
import { testSetup } from './commands/test-setup';
import { undo } from './commands/undo';
import { loadConfig } from './config';
import { runScript } from './docker';
import { applyRawConstraints } from './raw-constraints';

const USAGE = `Usage: drizzle-snapshots <command>

  snapshot [name]   Copy the dev database into a snapshot (base: the protected baseline)
  restore [name]    Restore a snapshot (base, a name, or the latest)
  undo              Restore the snapshot taken before the last push
  list              List this project's snapshots
  clean [--force]   List this project's snapshots and backups; --force removes them
  push              Snapshot, drizzle-kit push, then re-apply the raw constraints
  constraints       Apply missing raw constraints to the dev database
  test-setup        Rebuild the test database: migrations, push, raw constraints
  e2e-setup         Rebuild and seed the e2e database, then snapshot it
  e2e-restore       Put the e2e database back to its snapshot
  e2e-teardown      Remove the e2e snapshot
  schema-check      Fail when the staged schema has no migration (pre-commit)`;

async function constraints(): Promise<void> {
	const config = await loadConfig(process.cwd());
	const created = await applyRawConstraints(config.dbTarget('app').url, config.rawConstraints);
	console.log(
		created.length > 0
			? `✅ Applied raw constraint(s): ${created.join(', ')}`
			: '✅ Raw constraints already present'
	);
}

const [command, arg] = process.argv.slice(2);

const commands: Record<string, () => unknown> = {
	snapshot: () => snapshot(arg),
	restore: () => restore(arg),
	undo,
	list,
	clean: () => clean(arg === '--force'),
	push: async () => push(await loadConfig(process.cwd())),
	constraints,
	'test-setup': async () => testSetup(await loadConfig(process.cwd())),
	'e2e-setup': async () => e2eSetup(await loadConfig(process.cwd())),
	'e2e-restore': e2eRestore,
	'e2e-teardown': e2eTeardown,
	'schema-check': async () => schemaCheck(await loadConfig(process.cwd())),
};

const run = command ? commands[command] : undefined;
if (!run) {
	console.log(USAGE);
	process.exit(command ? 1 : 0);
}

await runScript(run);
