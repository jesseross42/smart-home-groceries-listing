import { describe, expect, it } from "vitest";
import {
	defaultDeliveryDate,
	deliveryDayChoices,
	localDate,
	storeDayLabel,
} from "#/lib/delivery-days";

describe("deliveryDayChoices", () => {
	it("starts with tomorrow, sent as the store's next day", () => {
		const choices = deliveryDayChoices(new Date(2026, 9, 2, 7, 46));

		expect(choices).toHaveLength(7);
		expect(choices[0].date).toBeNull();
		expect(choices[0].label).toMatch(/^Tomorrow \(Sat,? 3 Oct\)$/);
		expect(choices[1].date).toBe("2026-10-04");
		expect(choices[6].date).toBe("2026-10-09");
	});

	it("crosses months and years", () => {
		expect(
			deliveryDayChoices(new Date(2026, 11, 30), 3).map((c) => c.date),
		).toEqual([null, "2027-01-01", "2027-01-02"]);
	});
});

describe("localDate and storeDayLabel", () => {
	it("format calendar days without time-zone drift", () => {
		expect(localDate(new Date(2026, 0, 5, 23, 59))).toBe("2026-01-05");
		expect(storeDayLabel("2026-10-03T07:00:00")).toMatch(/Sat,? 3 Oct/);
	});
});

describe("defaultDeliveryDate", () => {
	// Friday 2 October 2026.
	const today = new Date(2026, 9, 2, 7, 46);

	it("is tomorrow with no saved weekday", () => {
		expect(defaultDeliveryDate(null, today)).toBeNull();
		expect(defaultDeliveryDate(undefined, today)).toBeNull();
	});

	it("sends tomorrow's weekday as the store's next day", () => {
		expect(defaultDeliveryDate("saturday", today)).toBeNull();
	});

	it("finds the next such weekday, a week ahead for today's own", () => {
		expect(defaultDeliveryDate("monday", today)).toBe("2026-10-05");
		expect(defaultDeliveryDate("friday", today)).toBe("2026-10-09");
	});

	it("always lands on an offered day button", () => {
		const offered = deliveryDayChoices(today).map((c) => c.date);
		for (const day of ["sunday", "tuesday", "wednesday", "thursday"] as const) {
			expect(offered).toContain(defaultDeliveryDate(day, today));
		}
	});
});
