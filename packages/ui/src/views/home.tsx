import type { SplitDest } from '@app/ui/kit';

/** The line under the app's name. */
export const HOME_LEDE = 'What each part of the app is for.';

/**
 * **The ground's page**: the app's home, beside every page from `lg` and under every page below
 * it. The shell gives it the scroll, the rule and the margin. It holds no records of its own, so
 * it names the app and says what each destination is for; an app with a home of its own draws
 * that here instead.
 */
export function HomePage({
	name,
	lede = HOME_LEDE,
	destinations,
}: {
	name: string;
	lede?: string;
	destinations: readonly SplitDest[];
}) {
	return (
		<div className="flex flex-col gap-6 pt-6 pb-7">
			<div className="flex flex-col gap-1">
				<h1 className="text-ui-lead font-medium text-ui-ink">{name}</h1>
				<p className="text-ui-body text-ui-ink-2">{lede}</p>
			</div>
			<dl className="flex flex-col gap-3 border-t border-ui-rule pt-3">
				{destinations.map(dest => (
					<div key={dest.id}>
						<dt className="text-ui-body font-medium text-ui-ink">{dest.label}</dt>
						<dd className="text-ui-body text-ui-ink-3">{dest.blurb}</dd>
					</div>
				))}
			</dl>
		</div>
	);
}
