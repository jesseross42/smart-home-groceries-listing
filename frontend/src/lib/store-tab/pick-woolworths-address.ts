/**
 * Picks the household's saved delivery address (Settings › Delivery) on
 * woolworths.com.au, using the website's own panel the way a person would.
 * Markup captured 2026-11-15.
 *
 * 1. The header ribbon's "Delivery to:" button
 *    (`button[class*="fulfilment-ribbon_component_fms-type"]`) shows the
 *    address now in use. Already that one: nothing to do.
 * 2. Otherwise press it. The panel ("How would you like to shop?") lists
 *    the account's saved addresses as radios in `label.saved-address`.
 * 3. Click the radio whose words match (case and spaces ignored), then
 *    "Save & continue".
 *
 * Only addresses already saved on the account can be picked; adding a new
 * one stays on Woolworths. Must stay **self-contained**: the bookmarklet
 * carries its source text.
 */

/** What happened to the address. */
export interface AddressOutcome {
	outcome: "kept" | "changed" | "failed";
	problem?: string;
}

/** Makes `address` the delivery address, when the account has it saved. */
export async function pickWoolworthsAddress(
	address: string,
): Promise<AddressOutcome> {
	const same = (text: string | null | undefined) =>
		(text ?? "").replace(/\s+/g, " ").trim().toLowerCase() ===
		address.replace(/\s+/g, " ").trim().toLowerCase();
	const wait = async (ready: () => boolean) => {
		for (let tries = 0; tries < 30 && !ready(); tries++) {
			await new Promise((resolve) => setTimeout(resolve, 100));
		}
		return ready();
	};
	const ribbon = document.querySelector<HTMLButtonElement>(
		'button[class*="fulfilment-ribbon_component_fms-type"]',
	);
	if (!ribbon) {
		return {
			outcome: "failed",
			problem: "The Woolworths “Delivery to” button was not on this page",
		};
	}
	const current = ribbon.querySelector('[class*="fms-value"]')?.textContent;
	if (same(current)) return { outcome: "kept" };

	ribbon.click();
	const saved = () => [
		...document.querySelectorAll<HTMLLabelElement>("label.saved-address"),
	];
	if (!(await wait(() => saved().length > 0))) {
		return {
			outcome: "failed",
			problem: "The Woolworths address list did not open",
		};
	}
	const match = saved().find((label) => same(label.textContent));
	const radio = match?.querySelector<HTMLInputElement>('input[type="radio"]');
	if (!radio) {
		return {
			outcome: "failed",
			problem: `“${address}” is not a saved Woolworths address. Add it on Woolworths first`,
		};
	}
	radio.click();
	const save = [...document.querySelectorAll<HTMLButtonElement>("button")].find(
		(button) => /save\s*&\s*continue/i.test(button.textContent ?? ""),
	);
	if (!save) {
		return {
			outcome: "failed",
			problem: "Woolworths changed its address panel",
		};
	}
	save.click();
	if (
		!(await wait(() =>
			same(ribbon.querySelector('[class*="fms-value"]')?.textContent),
		))
	) {
		return {
			outcome: "failed",
			problem: "Woolworths did not switch to the saved address",
		};
	}
	return { outcome: "changed" };
}
