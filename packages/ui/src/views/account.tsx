import { ErrandPlate } from '@app/ui/views/frame';
import { LogOut } from 'lucide-react';

import { Switch } from '@app/ui/kit';
import { APPEARANCE_OPTIONS, type Appearance } from '@app/ui/views/appearance';
import type { PageErrand } from '@app/ui/views/frame-types';

/**
 * **Account's page**: its head, from the rail, the device's appearance, and `Sign out`. The frame
 * owns the sign-out press (`PageErrand.press`), so the page never holds the power to sign anybody
 * out; the workbench's place and the app's route draw this same page.
 */
export const SIGN_OUT: PageErrand = { label: 'Sign out', icon: LogOut, press: 'sign-out' };

/** The line over the switch — the choice belongs to this device, not to a person. */
export const APPEARANCE_LINE = 'Appearance on this device';

/**
 * The page. `appearance` is the switch's value and press (`useAppearance`); a caller that hands
 * none draws `Sign out` alone. The switch stands at every width, since a phone has no other way to
 * reach it; the plate stands from `lg`, and below it `Sign out` is the slug's cell.
 */
export function AccountPage({
	appearance,
}: {
	appearance?: { value: Appearance; choose: (next: Appearance) => void };
}) {
	return (
		<>
			{appearance && (
				<div className="flex flex-col gap-2 pb-6">
					<p className="text-ui-body text-ui-ink-2">{APPEARANCE_LINE}</p>
					<Switch
						label={APPEARANCE_LINE}
						options={APPEARANCE_OPTIONS}
						value={appearance.value}
						choose={appearance.choose}
					/>
				</div>
			)}
			<ErrandPlate errand={SIGN_OUT} />
		</>
	);
}
