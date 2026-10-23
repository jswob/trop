import js from '@eslint/js';
import prettierConfig from 'eslint-config-prettier';
import { createTypeScriptImportResolver } from 'eslint-import-resolver-typescript';
import { importX } from 'eslint-plugin-import-x';
import prettierPlugin from 'eslint-plugin-prettier';
import reactHooks from 'eslint-plugin-react-hooks';
import globals from 'globals';
import tseslint from 'typescript-eslint';

export default tseslint.config([
	{
		ignores: [
			'**/node_modules/',
			'**/dist/',
			'**/build/',
			'**/coverage/',
			'**/*.d.ts',
			'**/drizzle/',
			'.claude/',
			'**/routeTree.gen.ts',
			'apps/workbench/probe/out/',
			// The engine is Rust; its only JavaScript is rustdoc's, under engine/target/
			'engine/',
		],
	},
	{
		files: ['**/*.{ts,tsx,js,jsx}'],
		extends: [js.configs.recommended, ...tseslint.configs.recommendedTypeChecked, prettierConfig],
		plugins: {
			'import-x': importX,
			prettier: prettierPlugin,
		},
		languageOptions: {
			ecmaVersion: 'latest',
			sourceType: 'module',
			globals: {
				...globals.node,
			},
			parserOptions: {
				tsconfigRootDir: import.meta.dirname,
				projectService: true,
			},
		},
		settings: {
			'import-x/resolver-next': [createTypeScriptImportResolver()],
			'import-x/internal-regex': '^@/',
		},
		rules: {
			'@typescript-eslint/no-unused-vars': [
				'error',
				{
					argsIgnorePattern: '^_',
					varsIgnorePattern: '^_',
				},
			],
			'@typescript-eslint/no-explicit-any': 'warn',
			'@typescript-eslint/consistent-type-imports': 'error',
			'@typescript-eslint/no-non-null-assertion': 'warn',
			'@typescript-eslint/naming-convention': [
				'error',
				{
					selector: 'variableLike',
					filter: {
						regex: '^_',
						match: false,
					},
					format: ['camelCase', 'PascalCase', 'UPPER_CASE'],
				},
				{
					selector: 'variableLike',
					filter: {
						regex: '^_',
						match: true,
					},
					format: null,
					custom: {
						regex: '^_[a-zA-Z][a-zA-Z0-9]*$',
						match: true,
					},
				},
				{
					selector: 'typeLike',
					format: ['PascalCase'],
				},
			],
			'import-x/order': [
				'error',
				{
					groups: ['builtin', 'external', 'internal', 'parent', 'sibling', 'index'],
					alphabetize: {
						order: 'asc',
					},
				},
			],
			'import-x/no-duplicates': 'error',
			'import-x/newline-after-import': 'error',
			'no-console': ['warn', { allow: ['warn', 'error'] }],
			'no-debugger': 'error',
			'prefer-const': 'error',
			'no-var': 'error',
			'object-shorthand': 'error',
			'prefer-template': 'error',
			'no-implicit-coercion': ['error', { boolean: true, number: true, string: true }],
			'prettier/prettier': 'error',
		},
	},
	{
		files: [
			'apps/web/src/**/*.{ts,tsx}',
			'apps/workbench/**/*.{ts,tsx}',
			'packages/ui/src/**/*.{ts,tsx}',
			'packages/workbench/src/**/*.{ts,tsx}',
		],
		plugins: {
			'react-hooks': reactHooks,
		},
		languageOptions: {
			globals: globals.browser,
			parserOptions: {
				ecmaFeatures: {
					jsx: true,
				},
			},
		},
		rules: {
			...reactHooks.configs.recommended.rules,
		},
	},
	{
		/*
		 * Tier 2 prototypes must export a type called `PROPOSED_SHAPE` — a fixed
		 * handoff name, not a style choice — and these files are disposable by
		 * design. Relaxing the naming rule here keeps prototyping cheap instead of
		 * requiring an inline disable in every new proto.
		 */
		files: ['apps/workbench/proto/**/*.{ts,tsx}'],
		rules: {
			'@typescript-eslint/naming-convention': 'off',
		},
	},
	{
		/*
		 * The app never imports the design workbench: not its content in `apps/workbench`, and not
		 * its engine, `@jswob/design-workbench`. The UI the app draws lives in `packages/ui`, which
		 * the web and the workbench both import, never the reverse — or the workbench's fixtures
		 * ship. The specimens beside the UI package's components are the workbench's too.
		 */
		files: ['apps/web/src/**/*.{ts,tsx}'],
		rules: {
			'no-restricted-imports': [
				'error',
				{
					patterns: [
						{
							group: [
								'**/apps/workbench',
								'**/apps/workbench/**',
								'../**/workbench/**',
								'@app/workbench-app',
								'@app/workbench-app/*',
							],
							message:
								'The app never imports the design workbench; move the module to packages/ui.',
						},
						{
							group: ['@jswob/design-workbench', '@jswob/design-workbench/*'],
							message: 'The app never imports the design workbench’s engine.',
						},
						{
							group: ['@app/ui/**/*.specimen'],
							message:
								'A specimen is a component’s page on the workbench; the app never imports one.',
						},
					],
				},
			],
		},
	},
	{
		files: ['apps/api/src/**/*.{ts,js}'],
		languageOptions: {
			globals: {
				...globals.node,
				Bun: 'readonly',
			},
		},
		rules: {
			'no-console': 'off',
		},
	},
	{
		files: ['**/*.config.{ts,js}', '**/*.d.ts'],
		languageOptions: {
			globals: globals.node,
		},
	},
	{
		files: ['**/*.test.ts', '**/*.test.tsx', '**/*.spec.ts', '**/*.spec.tsx'],
		rules: {
			'no-console': 'off',
			'@typescript-eslint/no-explicit-any': 'off',
			'@typescript-eslint/no-non-null-assertion': 'off',
		},
	},
	{
		// Command-line tools: printing is what they do.
		files: ['**/scripts/**/*.{ts,js}', 'packages/drizzle-snapshots/src/**/*.ts', 'template/*.ts'],
		rules: {
			'no-console': 'off',
		},
	},
	{
		files: ['**/vitest.config.ts', '**/vitest.setup.ts', '**/example.ts'],
		rules: {
			'no-console': 'off',
			'@typescript-eslint/no-unsafe-assignment': 'off',
			'@typescript-eslint/no-unsafe-member-access': 'off',
			'@typescript-eslint/no-unsafe-return': 'off',
			'@typescript-eslint/no-unsafe-call': 'off',
			'@typescript-eslint/no-unsafe-argument': 'off',
		},
	},
]);
