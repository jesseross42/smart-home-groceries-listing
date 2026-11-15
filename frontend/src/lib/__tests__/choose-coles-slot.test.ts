import { describe, expect, it } from "vitest";
import { slotMarkup } from "#/lib/__tests__/coles-drawer";
import {
	type ColesSlot,
	chooseColesSlot,
	readColesSlots,
} from "#/lib/store-tab/choose-coles-slot";

const page = (html: string) => {
	const root = document.createElement("div");
	root.innerHTML = html;
	return root;
};

const slot = (value: string, start: string, end: string, fee: number) =>
	({
		value,
		label: `${start} to ${end}`,
		start,
		end,
		fee,
		deliverMore: false,
	}) satisfies ColesSlot;

describe("readColesSlots", () => {
	it("reads each time's words, hours, fee and kind", () => {
		const root = page(
			[
				slotMarkup({
					value: "201044649",
					label: "9am to 5pm",
					fee: "$3",
					deliverMore: true,
				}),
				slotMarkup({
					value: "nsh_dlw_Bar_22:30_23:30_2026-11-14_stl_x",
					label: "9:30am to 10:30am",
					fee: "$13",
				}),
				slotMarkup({ value: "late", label: "11:30pm to 12:30am", fee: "Free" }),
			].join(""),
		);

		expect(readColesSlots(root)).toEqual([
			{
				value: "201044649",
				label: "9am to 5pm",
				start: "09:00",
				end: "17:00",
				fee: 3,
				deliverMore: true,
			},
			{
				value: "nsh_dlw_Bar_22:30_23:30_2026-11-14_stl_x",
				label: "9:30am to 10:30am",
				start: "09:30",
				end: "10:30",
				fee: 13,
				deliverMore: false,
			},
			{
				value: "late",
				label: "11:30pm to 12:30am",
				start: "23:30",
				end: "24:30",
				fee: 0,
				deliverMore: false,
			},
		]);
	});

	it("skips times that cannot be picked or read", () => {
		const root = page(
			[
				slotMarkup({ value: "a", label: "9am to 10am", fee: "$8.50" }),
				slotMarkup({
					value: "b",
					label: "9am to 10am",
					fee: "$5",
					disabled: true,
				}),
				slotMarkup({ value: "c", label: "Soon", fee: "$5" }),
			].join(""),
		);

		expect(readColesSlots(root).map((s) => [s.value, s.fee])).toEqual([
			["a", 8.5],
		]);
	});
});

describe("chooseColesSlot", () => {
	const day = [
		slot("long", "09:00", "17:00", 3),
		slot("early", "07:00", "08:00", 13),
		slot("noon", "12:00", "13:00", 10),
		slot("late", "18:00", "19:00", 10),
		slot("lateLong", "17:00", "21:00", 10),
	];

	it("takes the cheapest time for any time of day", () => {
		expect(chooseColesSlot(day, "any")?.value).toBe("long");
	});

	it("keeps the whole window inside the part of the day", () => {
		expect(chooseColesSlot(day, "morning")?.value).toBe("early");
		expect(chooseColesSlot(day, "afternoon")?.value).toBe("noon");
	});

	it("breaks a tie on fee by the earliest, then the shortest", () => {
		expect(chooseColesSlot(day, "evening")?.value).toBe("lateLong");
		expect(
			chooseColesSlot(
				[
					slot("long", "18:00", "21:00", 10),
					slot("short", "18:00", "19:00", 10),
				],
				"evening",
			)?.value,
		).toBe("short");
	});

	it("is null when nothing fits", () => {
		expect(
			chooseColesSlot([slot("long", "09:00", "17:00", 3)], "morning"),
		).toBeNull();
		expect(chooseColesSlot([], "any")).toBeNull();
	});
});
