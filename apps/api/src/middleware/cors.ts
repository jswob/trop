import { cors } from '@elysiajs/cors';

import { config } from '../config';

/**
 * How long a browser may reuse a preflight answer. The plugin's default is five seconds, which had
 * the browser ask permission again before nearly every request: about half of what the API served
 * was preflights. Browsers cap the value themselves (Chromium at two hours, Safari at ten minutes),
 * so a day asks for the most each will give.
 */
const PREFLIGHT_MAX_AGE_SECONDS = 24 * 60 * 60;

/** The API's CORS policy: the web's origin when one is configured, any origin otherwise. */
export function corsPolicy() {
	return cors({
		origin: config.webOrigin ? [config.webOrigin] : true,
		maxAge: PREFLIGHT_MAX_AGE_SECONDS,
	});
}
