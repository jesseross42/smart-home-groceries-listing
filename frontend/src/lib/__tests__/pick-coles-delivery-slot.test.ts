import { afterEach, describe, expect, it, vi } from "vitest";
import { type DrawerDay, openColesDrawer } from "#/lib/__tests__/coles-drawer";
import {
	chooseColesSlot,
	readColesSlots,
} from "#/lib/store-tab/choose-coles-slot";
import { pickColesDeliverySlot } from "#/lib/store-tab/pick-coles-delivery-slot";

const helpers = { readSlots: readColesSlots, chooseSlot: chooseColesSlot };
/** Saturday 14 Nov 2026, store-local. */
const now = new Date(2026, 10, 14, 10, 0);

const days: DrawerDay[] = [
	{
		day: "Tomorrow",
		date: "15 Nov",
		slots: [
			{ value: "sun-long", label: "9am to 5pm", fee: "$3", deliverMore: true },
			{ value: "sun-am", label: "9:30am to 10:30am", fee: "$13" },
		],
	},
	{
		day: "Mon",
		date: "16 Nov",
		slots: [
			{ value: "mon-pm", label: "6pm to 7pm", fee: "$10" },
			{ value: "mon-late", label: "11:30pm to 12:30am", fee: "$8" },
		],
	},
];

afterEach(() => {
	document.body.innerHTML = "";
});

describe("pickColesDeliverySlot", () => {
	it("opens the time picker with Coles' When button", async () => {
		const confirmed = vi.fn();
		const when = document.createElement("button");
		when.dataset.testid = "when-link";
		when.setAttribute("aria-label", "When, Select a time");
		when.addEventListener("click", () => openColesDrawer(days, confirmed));
		document.body.append(when);

		const report = await pickColesDeliverySlot(
			{ date: null, time_of_day: "any" },
			helpers,
			now,
		);

		expect(confirmed).toHaveBeenCalledWith("sun-long");
		expect(report.outcome).toBe("reserved");
	});

	it("says so when the time picker does not open", async () => {
		const when = document.createElement("button");
		when.dataset.testid = "when-link";
		document.body.append(when);

		const report = await pickColesDeliverySlot(
			{ date: null, time_of_day: "any" },
			helpers,
			now,
		);

		expect(report.outcome).toBe("failed");
		expect(report.problem).toContain("picker did not open");
	});

	it("picks tomorrow's cheapest time and confirms it", async () => {
		const confirmed = vi.fn();
		openColesDrawer(days, confirmed);

		const report = await pickColesDeliverySlot(
			{ date: null, time_of_day: "any" },
			helpers,
			now,
		);

		expect(confirmed).toHaveBeenCalledWith("sun-long");
		expect(report).toEqual({
			outcome: "reserved",
			window_label: "9am to 5pm",
			window_start: "2026-11-15T09:00:00",
			window_end: "2026-11-15T17:00:00",
			fee: "3",
			problem: undefined,
		});
	});

	it("opens the day asked for, and a window past midnight ends the next day", async () => {
		const confirmed = vi.fn();
		openColesDrawer(days, confirmed);

		const report = await pickColesDeliverySlot(
			{ date: "2026-11-16", time_of_day: "any" },
			helpers,
			now,
		);

		expect(confirmed).toHaveBeenCalledWith("mon-late");
		expect(report.window_start).toBe("2026-11-16T23:30:00");
		expect(report.window_end).toBe("2026-11-17T00:30:00");
	});

	it("moves to the next day with a matching time, and says so", async () => {
		const confirmed = vi.fn();
		openColesDrawer(days, confirmed);

		const report = await pickColesDeliverySlot(
			{ date: "2026-11-15", time_of_day: "evening" },
			helpers,
			now,
		);

		expect(confirmed).toHaveBeenCalledWith("mon-late");
		expect(report.outcome).toBe("reserved");
		expect(report.problem).toBe(
			"Nothing left on 2026-11-15, so the next day with a time was used.",
		);
	});

	it("reports when no day has a matching time", async () => {
		const confirmed = vi.fn();
		openColesDrawer(days, confirmed);

		const report = await pickColesDeliverySlot(
			{ date: null, time_of_day: "afternoon" },
			helpers,
			now,
		);

		expect(confirmed).not.toHaveBeenCalled();
		expect(report).toEqual({
			outcome: "failed",
			problem: "Coles has no afternoon delivery times left this week.",
		});
	});
});
