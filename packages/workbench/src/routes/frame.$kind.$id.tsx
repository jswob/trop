import { createFileRoute } from '@tanstack/react-router';
import { z } from 'zod';

import { FrameDocument } from '../components/FrameDocument';
import { MissingEntry } from '../components/MissingEntry';
import { useTypedRoute } from '../lib/typed-route';

const frameSearchSchema = z.object({
	variant: z.string().optional(),
	chrome: z.enum(['bare']).optional(),
	/** Both mirror `stageSearchSchema`; see the notes on their bounds and meaning there. */
	t: z.coerce.number().min(0).max(10_000).optional(),
	rate: z.coerce.number().min(0).max(1).optional(),
	theme: z.enum(['dark', 'light']).optional(),
});

type FrameSearch = z.infer<typeof frameSearchSchema>;

/**
 * One variant, alone, owning the whole viewport. The compare grid loads these in
 * iframes; opening the same URL in a tab gives the variant full-size for a close
 * screenshot.
 *
 * The address is all this route owns. What a frame is — its appearance, its motion, the entry and
 * the state it draws — is `components/FrameDocument`, which a board's frame draws too.
 */
export const Route = createFileRoute('/frame/$kind/$id')({
	component: FramePage,
	validateSearch: (search: Record<string, unknown>) => {
		const parsed = frameSearchSchema.safeParse(search);
		return parsed.success ? parsed.data : {};
	},
});

function FramePage() {
	const { params, search } = useTypedRoute<{ kind: string; id: string }, FrameSearch>(Route);
	const { kind, id } = params;

	return (
		<FrameDocument
			id={id}
			variant={search.variant}
			chrome={search.chrome}
			t={search.t}
			rate={search.rate}
			theme={search.theme}
			missing={(what, name, of) => (
				<MissingEntry what={what === 'entry' ? kind : `variant of ${of ?? id}`} id={name} />
			)}
		/>
	);
}
