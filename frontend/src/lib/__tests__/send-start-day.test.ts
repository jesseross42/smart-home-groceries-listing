import { describe, expect, it } from "vitest";

import { sendStartDay } from "#/lib/send-start-day";

describe("sendStartDay", () => {
	const fee = {
		store: "coles" as const,
		store_name: "Coles",
		date: "2026-11-19",
		fee: "5",
	};

	it("starts on the weekday of the store's cheapest day", () => {
		const plan = { delivery_day: null, store_fees: [fee] };
		expect(sendStartDay(plan, "coles", "monday")).toBe("thursday");
	});

	it("falls back to the plan's day, then the usual day", () => {
		expect(
			sendStartDay(
				{ delivery_day: "friday", store_fees: [fee] },
				"woolworths",
				"monday",
			),
		).toBe("friday");
		expect(
			sendStartDay({ delivery_day: null, store_fees: [] }, "coles", "monday"),
		).toBe("monday");
	});
});
