import { AccountPage } from '@app/ui/views/account';
import { useAppearance } from '@app/ui/views/appearance';
import { createFileRoute } from '@tanstack/react-router';

/**
 * **Account**: its head from the rail, the device's appearance — kept on the device, never the
 * server — and `Sign out`, the frame's press, never the page's.
 */
export const Route = createFileRoute('/account')({
	component: AccountRoute,
});

function AccountRoute() {
	return <AccountPage appearance={useAppearance()} />;
}
