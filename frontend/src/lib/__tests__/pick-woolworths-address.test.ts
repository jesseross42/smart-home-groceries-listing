import { afterEach, describe, expect, it } from "vitest";
import { pickWoolworthsAddress } from "#/lib/store-tab/pick-woolworths-address";

const SAVED = [
	"U 2  8 Athelstan Rd, CAMBERWELL VIC 3124",
	"33 Coventry St, SOUTHBANK VIC 3006",
];

/** The header ribbon and, once pressed, the saved-address panel (2026-11-15). */
function woolworthsPage(current: string, { panelOpens = true } = {}) {
	document.body.innerHTML = `<button type="button" class="core-button fulfilment-ribbon_component_fms-type__hsxul"><span>Delivery to:</span><span class="fulfilment-ribbon_component_fms-value___OPP8">${current}</span></button>`;
	const ribbon = document.querySelector("button") as HTMLButtonElement;
	const value = ribbon.querySelector("[class*=fms-value]") as HTMLElement;
	ribbon.addEventListener("click", () => {
		if (!panelOpens) return;
		const panel = document.createElement("div");
		panel.innerHTML = `${SAVED.map(
			(a) =>
				`<label class="saved-address"><input type="radio" class="wowRadio"><label>${a}</label></label>`,
		).join(
			"",
		)}<div class="fulfilment-button"><button class="button shopper-action" type="button"> Save &amp; continue </button></div>`;
		const save = panel.querySelector(".fulfilment-button button");
		save?.addEventListener("click", () => {
			const picked = [...panel.querySelectorAll("label.saved-address")].find(
				(l) => l.querySelector<HTMLInputElement>("input")?.checked,
			);
			if (picked) value.textContent = picked.textContent;
			panel.remove();
		});
		document.body.append(panel);
	});
}

afterEach(() => {
	document.body.innerHTML = "";
});

describe("pickWoolworthsAddress", () => {
	it("keeps the address already in use", async () => {
		woolworthsPage(SAVED[0]);
		expect(
			await pickWoolworthsAddress("u 2 8 athelstan rd, camberwell vic 3124"),
		).toEqual({
			outcome: "kept",
		});
	});

	it("picks a saved address and saves it", async () => {
		woolworthsPage(SAVED[0]);
		expect(
			await pickWoolworthsAddress(" 33 Coventry St,  SOUTHBANK VIC 3006"),
		).toEqual({
			outcome: "changed",
		});
		expect(document.body.textContent).toContain("33 Coventry St");
	});

	it("says when the address is not saved on the account", async () => {
		woolworthsPage(SAVED[0]);
		const result = await pickWoolworthsAddress("1 Nowhere St");
		expect(result.outcome).toBe("failed");
		expect(result.problem).toContain("not a saved Woolworths address");
	});

	it("says when the page has no Delivery to button", async () => {
		const result = await pickWoolworthsAddress("1 Nowhere St");
		expect(result.problem).toContain("“Delivery to” button");
	});
});
