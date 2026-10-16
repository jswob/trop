/**
 * `bun setup`: `.env` from `.env.example`, when there is none.
 *
 * `bun setup <name> [options]`, in a clone of the template, first turns the clone into the app
 * `<name>`: the template's own `setup.ts` does that, and goes with the rest of the template.
 */
import { existsSync, copyFileSync, readFileSync } from 'fs';
import { resolve } from 'path';

const root = resolve(import.meta.dirname, '..');
const args = process.argv.slice(2);

if (args.length > 0) {
	const maker = resolve(root, 'template', 'setup.ts');
	if (!existsSync(maker)) {
		console.error(
			'bun setup: this repo is an app already, so it takes no name. bun setup alone writes .env.'
		);
		process.exit(1);
	}
	const { main } = (await import(maker)) as { main: (argv: string[], root: string) => number };
	const code = main(args, root);
	if (code !== 0) process.exit(code);
}

const envPath = resolve(root, '.env');
const envExamplePath = resolve(root, '.env.example');

if (existsSync(envPath)) {
	console.log('✓ .env already exists, skipping copy.');
} else {
	copyFileSync(envExamplePath, envPath);
	console.log('✓ Created .env from .env.example');
}

const env = readFileSync(envPath, 'utf8');
const port = (name: string, fallback: string) =>
	new RegExp(`^${name}=(\\d+)`, 'm').exec(env)?.[1] ?? fallback;

console.log(`
──────────────────────────────────
  Getting Started
──────────────────────────────────

  1. Start the database:     bun db:up
  2. Run migrations:         bun db:migrate
  3. Start development:      bun dev

  Web  → http://localhost:${port('WEB_PORT', '5173')}
  API  → http://localhost:${port('API_PORT', '3001')}

  See README.md for more commands.
──────────────────────────────────
`);
