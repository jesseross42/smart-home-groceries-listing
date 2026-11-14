import { API_BASE_URL, expect, gotoList, itemCard, test } from "./fixtures";

/**
 * Items removed by Alexa, listed on Settings › Help with Undo, in a real
 * browser against the real backend. The removal is posted as the bridge
 * sidecar would (`ALEXA_BRIDGE_SECRET` is set by `scripts/e2e-tests.mjs`).
 */

test.describe("changed by voice", () => {
	test("a removed item is listed and Undo puts it back", async ({
		page,
		request,
	}) => {
		const added = await request.post(`${API_BASE_URL}/api/grocery-items`, {
			data: { name: "milk", quantity: 2 },
		});
		expect(added.status()).toBe(201);
		const removed = await request.post(
			`${API_BASE_URL}/api/intake/alexa/remove`,
			{
				headers: { "x-bridge-secret": "e2e-bridge-secret" },
				data: { item: "milk" },
			},
		);
		expect(removed.status()).toBe(201);

		await gotoList(page, "/settings/help");
		// Newest first: earlier runs (the other browser project) leave their
		// own undone milk rows in the list.
		const row = page
			.getByRole("region", { name: "Changed by voice" })
			.getByRole("listitem", { name: "milk" })
			.first();
		await expect(row).toContainText("Removed (was ×2)");

		await row.getByRole("button", { name: "Undo the change to milk" }).click();
		await expect(row).toContainText("Undone");

		await gotoList(page);
		await expect(itemCard(page, "milk")).toBeVisible();
	});
});
