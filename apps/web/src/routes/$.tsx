import { createFileRoute, redirect } from '@tanstack/react-router';

/**
 * **An address no route answers** lands on the ground, as an unknown record does. Replaced, so
 * the address never stays in the history.
 */
export const Route = createFileRoute('/$')({
	beforeLoad: () => {
		// eslint-disable-next-line @typescript-eslint/only-throw-error -- TanStack Router requires throwing redirect()
		throw redirect({ to: '/', replace: true });
	},
});
