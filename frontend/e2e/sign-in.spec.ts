import { expect, gotoList, test } from "./fixtures";

test.describe("sign-in off (test mode)", () => {
	test("says sign-in is off on every page", async ({ page }) => {
		await gotoList(page, "/");
		await expect(page.getByRole("status")).toHaveText(
			"Sign-in is off. Use this only on your own computer.",
		);
	});

	test("sends /sign-in to the grocery list", async ({ page }) => {
		await page.goto("/sign-in");
		await expect(page).toHaveURL(/\/$/);
		await expect(
			page.getByRole("navigation", { name: "Main navigation" }),
		).toBeVisible();
	});

	test("shows no account button", async ({ page }) => {
		await gotoList(page, "/");
		await expect(
			page.getByRole("button", { name: /sign out/i }),
		).toHaveCount(0);
	});
});
