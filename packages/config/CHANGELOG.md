# @jswob/style-guide

## 0.1.1

### Patch Changes

- 5d1ea70: `typescript/node`'s `outDir` is the app's `dist`, not the package's. It was resolved against the package, so an app's `tsc` wrote its build info into `node_modules/@jswob/style-guide/typescript/dist/`, and 0.1.0 was published with one. The package ships only its configs.
