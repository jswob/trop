import { createFileRoute } from '@tanstack/react-router';

import { KitList } from '../bench/KitList';

export const Route = createFileRoute('/_bench/kit/')({
	component: KitList,
});
