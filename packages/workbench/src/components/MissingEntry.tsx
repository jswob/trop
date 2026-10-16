import { setup } from '../lib/bound';

/**
 * Shown in a frame whose address names an entry or a variant that is no longer registered.
 *
 * It stands on the app's ground and inherits its ink and face, and spends no token of the app's:
 * the workbench knows none by name.
 */
export function MissingEntry({ what, id }: { what: string; id: string }) {
	return (
		<setup.Ground>
			<div style={{ padding: '32px 24px', fontSize: 14, lineHeight: '20px' }}>
				<p>
					No {what} called <code>{id}</code>.
				</p>
				<p style={{ marginTop: 4, opacity: 0.7 }}>
					It was renamed or removed. The workbench lists what is registered now.
				</p>
			</div>
		</setup.Ground>
	);
}
