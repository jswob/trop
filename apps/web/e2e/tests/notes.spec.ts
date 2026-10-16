import { expect, test } from '../fixtures/test.fixture';

/**
 * The smoke test: sign in, open Notes from the rail, then add a note and delete it through the real
 * API and database. The password is the API's `AUTH_PASSWORD`, from the root `.env`.
 */
test('signs in, adds a note and deletes it', async ({ page }) => {
	const password = process.env.AUTH_PASSWORD;
	if (password === undefined || password === '') throw new Error('AUTH_PASSWORD is not set');
	const title = `Smoke ${String(Date.now())}`;

	await page.goto('/');
	await page.getByLabel('Password').fill(password);
	await page.getByRole('button', { name: 'Sign in' }).click();
	await page
		.getByRole('navigation', { name: 'Destinations' })
		.getByRole('button', { name: 'Notes' })
		.click();

	await page.getByLabel('Title').fill(title);
	await page.getByRole('button', { name: 'Add note' }).click();

	const row = page.getByRole('listitem').filter({ hasText: title });
	await expect(row).toBeVisible();

	await row.getByRole('button', { name: 'Delete' }).click();
	await row.getByRole('button', { name: 'Remove' }).click();
	await expect(row).toHaveCount(0);
});
