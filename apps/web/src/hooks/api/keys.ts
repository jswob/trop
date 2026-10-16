export const noteKeys = {
	all: ['notes'] as const,
	lists: () => [...noteKeys.all, 'list'] as const,
	list: () => [...noteKeys.lists()] as const,
};
