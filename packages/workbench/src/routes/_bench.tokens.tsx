import { createFileRoute } from '@tanstack/react-router';

import { TokensPage } from '../bench/TokensPage';

export const Route = createFileRoute('/_bench/tokens')({
	component: TokensPage,
});
