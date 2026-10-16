import { CentredScreen } from '@app/ui/views/sign-in';
import { Button, READ_FAILED, TRY_AGAIN } from '@app/ui/kit';

/** What the plate says while the failed reads are asked again. */
export const TRYING_AGAIN = 'Trying again';

/**
 * **The ground's reads failed, in place of the frame.**
 *
 * The ground is drawn from the app's own reads, and a ground whose sources failed is not one to
 * read — so the frame gives way to the centred screen sign-in stands on: the mark, the app's name,
 * the line in red type, and the `Try again` plate. It stands whether nothing had loaded yet or a
 * refresh failed after it had. A page's own failed read, with the ground fine, is said in that
 * page's wait instead (`kit/Waiting`'s `WaitFailed`).
 */
export function ReadFailed({
	retrying = false,
	onRetry,
}: {
	/** The failed reads are being asked again; the plate says so and takes no second press. */
	retrying?: boolean;
	onRetry: () => void;
}) {
	return (
		<CentredScreen
			lede={
				<p role="alert" className="mt-1.5 text-ui-body leading-5 text-balance text-ui-red">
					{READ_FAILED}
				</p>
			}
		>
			<div className="mt-6 grid">
				<Button weight="primary" running={retrying ? TRYING_AGAIN : null} onClick={onRetry}>
					{TRY_AGAIN}
				</Button>
			</div>
		</CentredScreen>
	);
}
