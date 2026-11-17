import type { APIRequestContext } from "@playwright/test";
import { API_BASE_URL, expect, gotoList, test } from "./fixtures";

/**
 * The order screen's "Ways to buy" and Settings › Delivery, in a real
 * browser against the real backend over the fake catalogue
 * (`STORE_CLIENTS=fake`): milk is $3.10 at Woolworths and $4.95 at Coles,
 * bananas $0.80 and $0.83.
 */

async function addItem(request: APIRequestContext, name: string) {
	const res = await request.post(`${API_BASE_URL}/api/grocery-items`, {
		data: { name, quantity: 1 },
	});
	expect(res.status()).toBe(201);
	return ((await res.json()) as { id: string }).id;
}

async function choose(
	request: APIRequestContext,
	itemId: string,
	store: string,
	productId: string,
) {
	const res = await request.put(
		`${API_BASE_URL}/api/grocery-items/${itemId}/selection`,
		{ data: { store, product_id: productId } },
	);
	expect(res.status()).toBe(200);
}

/** Milk and bananas chosen at both stores, Coles last, then committed. */
async function commitAtBothStores(request: APIRequestContext) {
	const milk = await addItem(request, "milk");
	const bananas = await addItem(request, "bananas");
	await choose(request, milk, "woolworths", "w-milk-2l");
	await choose(request, bananas, "woolworths", "w-banana");
	await choose(request, milk, "coles", "c-milk-3l");
	await choose(request, bananas, "coles", "c-banana");
	await request.post(`${API_BASE_URL}/api/grocery-items/commit`);
}

test.describe("delivery fees and ways to buy", () => {
	test("fees typed in Settings › Delivery are added to every option", async ({
		page,
		request,
	}) => {
		await commitAtBothStores(request);
		await gotoList(page, "/settings/delivery");

		for (const store of ["Woolworths", "Coles"]) {
			const fields = page.getByRole("group", { name: store, exact: true });
			await fields.getByLabel("Delivery fee").fill("9");
		}
		await page.getByRole("button", { name: "Save" }).click();
		await expect(page.getByText("Delivery settings saved")).toBeVisible();

		await gotoList(page, "/order");
		// The best way to buy is applied by itself.
		await expect(
			page.getByText('The order now uses "All at Woolworths"'),
		).toBeVisible();
		const best = page.getByRole("region", { name: "All at Woolworths" });
		await expect(best).toContainText("Recommended");
		await expect(best).toContainText("Current");
		await expect(best).toContainText("$12.90");
		await expect(best).toContainText("$3.90 + $9.00 delivery");
		await expect(
			page.getByRole("region", { name: "All at Coles" }),
		).toContainText("$14.78");
	});

	test("the best option moves the order by itself, and Undo moves it back", async ({
		page,
		request,
	}) => {
		await commitAtBothStores(request);
		await gotoList(page, "/order");

		await expect(page.getByText(/The order now uses/)).toBeVisible();
		await expect(
			page.getByRole("region", { name: "Woolworths", exact: true }),
		).toContainText("Subtotal$3.90");
		await expect(
			page.getByRole("region", { name: "Coles", exact: true }),
		).toBeHidden();

		await page.getByRole("button", { name: "Undo" }).click();
		await expect(
			page.getByRole("region", { name: "Coles", exact: true }),
		).toContainText("Subtotal$5.78");
	});

	test("unset fees point to the settings, and a mode can be tried", async ({
		page,
		request,
	}) => {
		await commitAtBothStores(request);
		await gotoList(page, "/order");

		await expect(page.getByText("Some delivery fees are not set yet.")).toBeVisible();
		await page.getByRole("button", { name: "Coles only" }).click();
		await expect(page).toHaveURL(/mode=coles_only/);
		const options = page.getByRole("region", { name: "Ways to buy" });
		await expect(options.getByRole("region").first()).toHaveAccessibleName(
			"All at Coles",
		);

		await page.getByRole("link", { name: "Set delivery fees" }).click();
		await expect(page.getByRole("heading", { name: "Delivery" })).toBeVisible();
	});
});

test("a saved deadline gates recommendations on the store's delivery windows", async ({
	page,
	request,
}) => {
	const date = new Date(Date.now() + 3 * 86400000).toISOString().slice(0, 10);
	const milk = await addItem(request, "milk");
	await choose(request, milk, "woolworths", "w-milk-2l");
	await request.post(`${API_BASE_URL}/api/grocery-items/commit`);

	await gotoList(page, "/settings/delivery");
	await page.getByLabel("Need by (Sydney time)").fill(`${date}T18:00`);
	await page.getByRole("button", { name: "Save" }).click();
	await expect(page.getByText("Delivery settings saved")).toBeVisible();

	await gotoList(page, "/order");
	const woolworths = page.getByRole("region", { name: "All at Woolworths" });
	const secretResponse = await request.get(
		`${API_BASE_URL}/api/trolley-handoffs/store-tab-secret`,
	);
	const { secret } = (await secretResponse.json()) as { secret: string };
	async function reportWindow(end: string) {
		const report = await request.post(`${API_BASE_URL}/api/store-tab/delivery-windows`, {
			headers: { origin: "https://www.woolworths.com.au" },
			data: { secret, store: "woolworths", windows: [
				{ start: `${date}T16:00:00`, end: `${date}T${end}:00`, fee: "5" },
			] },
		});
		expect(report.status()).toBe(204);
		await page.getByRole("button", { name: "Check prices again" }).click();
	}

	await reportWindow("18:01");
	await expect(woolworths).toContainText("No reported delivery window meets");
	await expect(woolworths.getByText("Recommended")).toHaveCount(0);

	await reportWindow("18:00");
	await expect(woolworths).toContainText("Delivery window");
	await expect(woolworths).toContainText("$5.00 delivery");
	await expect(woolworths.getByText("Recommended")).toBeVisible();
});
