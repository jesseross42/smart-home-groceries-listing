/**
 * Reads the delivery times in Coles' "Choose delivery slot" drawer and picks
 * one. The drawer's markup was captured from coles.com.au on 2026-11-14.
 *
 * Each time is a radio in `[data-testid="slot-window-item"]`, with the
 * store's words ("9:30am to 10:30am"), a fee ("$13") and, for the cheaper
 * long windows, a "Deliver More" badge. Radio values are Coles' own ids: a
 * number for a long window, `nsh_dlw_…` for a one-hour window.
 *
 * Rule (as for Woolworths): the cheapest, then the earliest start, then the
 * shortest. For a part of the day, the whole window must sit in it —
 * morning ends by 12pm, afternoon starts at 12pm and ends by 5pm, evening
 * starts at 5pm — so a 9am-to-5pm window is never a "morning" delivery.
 *
 * Both functions must stay **self-contained** (no imports, no outside
 * variables): the bookmarklet carries their source text.
 */
import type { TimeOfDay } from "#/lib/store-tab/choose-woolworths-window";

/** One delivery time in the drawer, store-local. */
export interface ColesSlot {
	/** The radio's value: Coles' id for this time. */
	value: string;
	/** The store's words, e.g. "9:30am to 10:30am". */
	label: string;
	/** "HH:MM", 24-hour. */
	start: string;
	/** "HH:MM", 24-hour; "24:00" for midnight. */
	end: string;
	/** Dollars, e.g. 13 or 8.5; 0 when free. */
	fee: number;
	deliverMore: boolean;
}

/** Reads the delivery times shown in the drawer, skipping disabled ones. */
export function readColesSlots(root: ParentNode): ColesSlot[] {
	const clock = (text: string): string | null => {
		const match = /^(\d{1,2})(?::(\d{2}))?\s*(am|pm)$/i.exec(text.trim());
		if (!match) return null;
		const hour = (Number(match[1]) % 12) + (/pm/i.test(match[3]) ? 12 : 0);
		return `${String(hour).padStart(2, "0")}:${match[2] ?? "00"}`;
	};
	const slots: ColesSlot[] = [];
	for (const item of root.querySelectorAll(
		'[data-testid="slot-window-item"]',
	)) {
		const input = item.querySelector<HTMLInputElement>('input[type="radio"]');
		const label =
			item
				.querySelector('[class*="SlotSelectItemLabelHeading"]')
				?.textContent?.trim() ?? "";
		const [from, to] = label.split(/\s+to\s+/i);
		const start = from ? clock(from) : null;
		let end = to ? clock(to) : null;
		if (!input || input.disabled || !start || !end) continue;
		if (end <= start) end = `${Number(end.slice(0, 2)) + 24}${end.slice(2)}`;
		const cost =
			item
				.querySelector('[class*="SlotWindowItemCost"]')
				?.textContent?.trim() ?? "";
		const fee = /free/i.test(cost)
			? 0
			: Number(cost.replace(/[^0-9.]/g, "") || Number.NaN);
		if (!Number.isFinite(fee)) continue;
		slots.push({
			value: input.value,
			label,
			start,
			end,
			fee,
			deliverMore: !!item.querySelector('[aria-label="Deliver More"]'),
		});
	}
	return slots;
}

/** Picks the delivery time to reserve on one day, or `null` when none fits. */
export function chooseColesSlot(
	slots: ColesSlot[],
	timeOfDay: TimeOfDay,
): ColesSlot | null {
	const minutes = (time: string) =>
		Number(time.slice(0, 2)) * 60 + Number(time.slice(3, 5));
	const fits = (slot: ColesSlot) => {
		const start = minutes(slot.start);
		const end = minutes(slot.end);
		switch (timeOfDay) {
			case "morning":
				return end <= 12 * 60;
			case "afternoon":
				return start >= 12 * 60 && end <= 17 * 60;
			case "evening":
				return start >= 17 * 60;
			default:
				return true;
		}
	};
	const length = (slot: ColesSlot) => minutes(slot.end) - minutes(slot.start);
	const sorted = slots
		.filter(fits)
		.sort(
			(a, b) =>
				a.fee - b.fee ||
				a.start.localeCompare(b.start) ||
				length(a) - length(b),
		);
	return sorted[0] ?? null;
}
