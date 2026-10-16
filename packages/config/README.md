# @jswob/style-guide

Shared TypeScript and Prettier configs for Bun monorepo apps.

```bash
bun add -d @jswob/style-guide typescript prettier
```

## TypeScript

```json
{ "extends": "@jswob/style-guide/typescript/react" }
```

| Export                                | For                                    |
| ------------------------------------- | -------------------------------------- |
| `@jswob/style-guide/typescript/base`  | Strict flags every workspace shares    |
| `@jswob/style-guide/typescript/react` | A React app or package (DOM, JSX)      |
| `@jswob/style-guide/typescript/node`  | A server, a CLI or a script (Bun/Node) |

The other `@jswob/*` packages ship TypeScript source, which your `tsc` checks under your flags.
Extending these configs is what keeps the two agreeing.

## Prettier

`.prettierrc.json`:

```json
"@jswob/style-guide/prettier"
```

ESLint is not here: each repo configures it once, in its own `eslint.config.js`.

## License

MIT
