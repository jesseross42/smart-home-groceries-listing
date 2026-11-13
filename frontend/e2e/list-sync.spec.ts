import { API_BASE_URL, expect, gotoList, itemCard, test } from "./fixtures";

/**
 * The open list follows changes made elsewhere (another tab, Alexa), pushed
 * over the WebSocket, in a real browser against the real backend.
 */

test.describe("list sync", () => {
	test("an open list shows a change made elsewhere without a reload", async ({
		page,
		request,
	}) => {
		const added = await request.post(`${API_BASE_URL}/api/grocery-items`, {
			data: { name: "milk", quantity: 1 },
		});
		expect(added.status()).toBe(201);
		const { id } = (await added.json()) as { id: string };
		await gotoList(page);
		await expect(itemCard(page, "milk").getByLabel("Quantity 1")).toBeVisible();

		// The page's socket may still be opening, so the same edit is sent
		// again until the page has heard one. Each send is a full, valid edit.
		await expect(async () => {
			const edited = await request.patch(
				`${API_BASE_URL}/api/grocery-items/${id}`,
				{ data: { quantity: 5 } },
			);
			expect(edited.ok()).toBe(true);
			await expect(itemCard(page, "milk").getByLabel("Quantity 5")).toBeVisible({
				timeout: 2000,
			});
		}).toPass({ timeout: 20_000 });
	});
});
