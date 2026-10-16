/**
 * The docker steps the database scripts share. Each throws on failure instead of exiting, so a
 * `withStopped` block always starts its container again; a script's entry point prints the error
 * and exits non-zero.
 */

import { execSync } from 'child_process';

/**
 * Run a shell command and return its trimmed output.
 * @param command - The command
 * @param description - What it does, for the error message
 */
export function run(command: string, description: string): string {
	try {
		return execSync(command, { encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] }).trim();
	} catch (error) {
		const stderr = (error as { stderr?: string }).stderr?.trim();
		throw new Error(`Failed to ${description}${stderr ? `: ${stderr}` : ''}`, { cause: error });
	}
}

/** Every volume on the machine. */
export function listVolumes(): string[] {
	return run('docker volume ls --format "{{.Name}}"', 'list volumes').split('\n').filter(Boolean);
}

export function removeVolume(volume: string): void {
	run(`docker volume rm ${volume}`, `remove volume ${volume}`);
}

/**
 * Copy every file of `source` into a new volume `target`.
 * @param source - Volume to copy
 * @param target - Volume to create; must not exist
 */
export function copyToNewVolume(source: string, target: string): void {
	run(`docker volume create ${target}`, `create volume ${target}`);
	run(
		`docker run --rm -v ${source}:/source:ro -v ${target}:/target alpine sh -c "cp -a /source/. /target/"`,
		`copy ${source} into ${target}`
	);
}

/**
 * Replace everything in `target` with the contents of `source`.
 * @param source - Volume to copy from
 * @param target - Volume to overwrite
 */
export function replaceContents(source: string, target: string): void {
	run(
		`docker run --rm -v ${source}:/source:ro -v ${target}:/target alpine sh -c "rm -rf /target/* /target/.[^.]* && cp -a /source/. /target/"`,
		`restore ${source} into ${target}`
	);
}

/**
 * Stop a container, do `work`, and start it again whatever happens.
 * @param container - Container name
 * @param work - What to do while it is stopped
 */
export function withStopped<T>(container: string, work: () => T): T {
	console.log(`🛑 Stopping ${container}...`);
	run(`docker stop ${container}`, `stop ${container}`);
	try {
		return work();
	} finally {
		console.log(`🚀 Starting ${container}...`);
		run(`docker start ${container}`, `start ${container}`);
	}
}

/**
 * Run a script's entry point, printing a thrown error and exiting non-zero.
 * @param main - The script's work
 */
export async function runScript(main: () => unknown): Promise<void> {
	try {
		await main();
	} catch (error) {
		console.error(`❌ ${error instanceof Error ? error.message : String(error)}`);
		process.exit(1);
	}
}
