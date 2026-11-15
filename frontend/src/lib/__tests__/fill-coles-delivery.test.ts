import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { openColesDrawer } from "#/lib/__tests__/coles-drawer";
import {
	colesHelpers,
	colesPage,
	config,
	fakeNetwork,
	reportOf,
} from "#/lib/__tests__/coles-fixtures";
import { fillColesTrolley } from "#/lib/store-tab/fill-coles-trolley";

const RESERVED = {
	cutoffTime: "2026-11-15T09:00:00+11:00",
	shiftId: "shift-9",
	reservationExpirationTime: "2026-11-14T23:00:00+11:00",
};

const tomorrow = () => {
	const date = new Date();
	date.setDate(date.getDate() + 1);
	return `${date.getDate()} ${date.toLocaleString("en-AU", { month: "short" })}`;
};

describe("fillColesTrolley picking the delivery time", () => {
	beforeEach(() => colesPage());
	afterEach(() => {
		vi.unstubAllGlobals();
		vi.restoreAllMocks();
		localStorage.clear();
		document.body.innerHTML = "";
	});

	it("picks the time in Coles' picker, keeps it on every add, and says when to order by", async () => {
		const network = fakeNetwork();
		openColesDrawer(
			[
				{
					day: "Tomorrow",
					date: tomorrow(),
					slots: [{ value: "201044649", label: "9am to 5pm", fee: "$3" }],
				},
			],
			() => network.setSlot(RESERVED),
		);

		const result = await fillColesTrolley(config, colesHelpers);

		expect(reportOf(network.calls).delivery).toMatchObject({
			outcome: "reserved",
			window_label: "9am to 5pm",
			fee: "3",
		});
		const patch = network.calls.find((c) => c.method === "PATCH");
		expect(patch?.body).toMatchObject({
			slotCutOffTime: RESERVED.cutoffTime,
			slotId: "shift-9",
		});
		expect(result.message).toContain("9am to 5pm ($3)");
		expect(result.message).toContain("Order by");
	});

	it("reports the time as not picked when Coles does not keep it", async () => {
		const network = fakeNetwork();
		openColesDrawer([
			{
				day: "Tomorrow",
				date: tomorrow(),
				slots: [{ value: "201044649", label: "9am to 5pm", fee: "$3" }],
			},
		]);

		const result = await fillColesTrolley(config, colesHelpers);

		expect(reportOf(network.calls).delivery).toEqual({
			outcome: "failed",
			problem: "Coles did not keep the delivery time the app picked",
		});
		expect(result.message).toContain("pick a delivery time on Coles");
	});

	it("says so when Coles shows no When button", async () => {
		const network = fakeNetwork();

		await fillColesTrolley(config, colesHelpers);

		expect(reportOf(network.calls).delivery.problem).toContain(
			"“When” button was not on this page",
		);
	});
});
