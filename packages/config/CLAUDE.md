# Style Guide

Shared TypeScript and Prettier configurations for all workspace packages. ESLint is configured once,
in the root `eslint.config.js`.

## Exports

| Export Path                           | Description               |
| ------------------------------------- | ------------------------- |
| `@jswob/style-guide/typescript/base`  | Base TypeScript config    |
| `@jswob/style-guide/typescript/react` | React TypeScript config   |
| `@jswob/style-guide/typescript/node`  | Node.js TypeScript config |
| `@jswob/style-guide/prettier`         | Shared Prettier config    |

## Notes

- Config-only package — no runtime code, no scripts
- All workspace packages extend these configs via their local config files
