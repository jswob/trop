// What a task's tests may import: model calls answered from functions, so no test reaches the
// network. Kept apart from api.ts because it loads vitest.
export { fakeModels, schemaOf, type OpenRouterRequestBody } from './adapters/openrouter/fake.ts';
