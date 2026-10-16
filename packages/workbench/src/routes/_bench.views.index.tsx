import { createFileRoute } from '@tanstack/react-router';

import { PlaceList } from '../bench/PlaceList';

export const Route = createFileRoute('/_bench/views/')({
	component: PlaceList,
});
