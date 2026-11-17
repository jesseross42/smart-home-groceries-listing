/**
 * Picks a Coles delivery time in the household's own coles.com.au tab, by
 * using the website's "Choose delivery slot" drawer the way a person would.
 *
 * Why the drawer and not a call: the website reserves with
 * `POST /api/bff/slots/{slotId}`, whose body is not captured. The drawer
 * (captured 2026-11-14) makes that call itself, so the reservation is the
 * website's own.
 *
 * Steps:
 * 0. Open the drawer (`section[data-testid="slot-selector-drawer"]`) when it
 *    is closed, by pressing the header's "When" button
 *    (`button[data-testid="when-link"]`, "When, Select a time"; captured
 *    2026-11-15), and wait for it.
 * 1. Read the days (`[data-testid="date-select-item-N"]`, e.g. "Tomorrow",
 *    "15 Nov"). Start from the day asked for, or tomorrow when none was.
 * 2. On each day from there, click its radio, wait for its times, and pick
 *    one with `chooseSlot`. The first day with a time wins.
 * 3. Click that time, then "Confirm time slot".
 *
 * Only "This week" is read: "Next week" is an early-access tab Coles may
 * lock. A drawer that does not open is reported as not picked, with what to
 * do.
 *
 * Must stay **self-contained** (no imports, no outside variables): the
 * bookmarklet carries its source text, and its helpers arrive as arguments
 * (`readColesSlots`, `chooseColesSlot`).
 */
import type { ColesSlot } from "#/lib/store-tab/choose-coles-slot";
import type {
	DeliveryWanted,
	TimeOfDay,
} from "#/lib/store-tab/choose-woolworths-window";
import type {
	DayFee,
	DeliveryReport,
} from "#/lib/store-tab/reserve-woolworths-delivery-window";

/** The self-contained helpers the picker is handed. */
export interface ColesSlotHelpers {
	readSlots: (root: ParentNode) => ColesSlot[];
	chooseSlot: (slots: ColesSlot[], timeOfDay: TimeOfDay) => ColesSlot | null;
}

/** Picks and confirms a delivery time in the open drawer. Never throws. */
export async function pickColesDeliverySlot(
	wanted: DeliveryWanted,
	helpers: ColesSlotHelpers,
	now: Date = new Date(),
	readOnly = false,
): Promise<DeliveryReport> {
	const failed = (problem: string): DeliveryReport => ({
		outcome: "failed",
		problem: problem.slice(0, 300),
	});
	const iso = (date: Date) =>
		`${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, "0")}-${String(date.getDate()).padStart(2, "0")}`;
	const MONTHS = "janfebmaraprmayjunjulaugsepoctnovdec";
	/** "15 Nov" as "YYYY-MM-DD": this year, or next year for January in December. */
	const dayOf = (text: string): string | null => {
		const match = /^(\d{1,2})\s+([a-z]{3})/i.exec(text.trim());
		const month = match ? MONTHS.indexOf(match[2].toLowerCase()) / 3 : -1;
		if (!match || month < 0 || !Number.isInteger(month)) return null;
		const year = now.getFullYear() + (month < now.getMonth() - 6 ? 1 : 0);
		return iso(new Date(year, month, Number(match[1])));
	};
	/** "YYYY-MM-DD" plus "HH:MM" (24:00 and later is the next day). */
	const at = (date: string, time: string) => {
		const [year, month, day] = date.split("-").map(Number);
		const hours = Number(time.slice(0, 2));
		const moved = new Date(year, month - 1, day + Math.floor(hours / 24));
		return `${iso(moved)}T${String(hours % 24).padStart(2, "0")}${time.slice(2)}:00`;
	};
	const wait = async (ready: () => boolean) => {
		for (let tries = 0; tries < 30 && !ready(); tries++) {
			await new Promise((resolve) => setTimeout(resolve, 100));
		}
		return ready();
	};
	try {
		const findDrawer = () =>
			document.querySelector('section[data-testid="slot-selector-drawer"]');
		const when = document.querySelector<HTMLButtonElement>(
			'button[data-testid="when-link"]',
		);
		if (!findDrawer() && when) {
			when.click();
			await wait(() => !!findDrawer());
		}
		const drawer = findDrawer();
		if (!drawer) {
			return failed(
				when
					? "The Coles delivery time picker did not open"
					: "Coles' “When” button was not on this page",
			);
		}
		if (
			readOnly &&
			drawer
				.querySelector('[data-testid="drawer-heading"]')
				?.textContent?.trim() !== "Choose delivery slot"
		) {
			return failed(
				"Select delivery to your address on Coles, then run the bookmark again.",
			);
		}
		const tomorrow = new Date(now);
		tomorrow.setDate(tomorrow.getDate() + 1);
		const wantedDate = readOnly ? iso(now) : (wanted.date ?? iso(tomorrow));
		const days = [
			...drawer.querySelectorAll('[data-testid^="date-select-item-"]'),
		]
			.filter((item) =>
				/^date-select-item-\d+$/.test(item.getAttribute("data-testid") ?? ""),
			)
			.map((item) => ({
				input: item.querySelector<HTMLInputElement>('input[type="radio"]'),
				date: dayOf(
					item.querySelector('[data-testid$="-date"]')?.textContent ?? "",
				),
			}))
			.filter((day) => day.input && day.date && day.date >= wantedDate);

		const slotPanel = () =>
			drawer.querySelector('[data-testid="slots-radio-list"]');
		const list = () => slotPanel() ?? drawer;
		const selectDay = async (input: HTMLInputElement) => {
			// Consecutive sold-out dates may render the same empty result.
			const previous = slotPanel();
			let rendered = false;
			const observer = new MutationObserver(() => {
				rendered = true;
			});
			if (previous) {
				observer.observe(previous, {
					childList: true,
					subtree: true,
					characterData: true,
					attributes: true,
					attributeFilter: ["disabled", "value"],
				});
			}
			try {
				input.click();
				return await wait(() => {
					const panel = slotPanel();
					return (
						input.checked &&
						panel !== null &&
						!panel.closest('[aria-busy="true"]') &&
						(panel !== previous || rendered)
					);
				});
			} finally {
				observer.disconnect();
			}
		};
		// Each day looked at, as its cheapest open time, for the planner.
		const seen: DayFee[] = [];
		for (const day of days) {
			const input = day.input as HTMLInputElement;
			if (!input.checked && !(await selectDay(input))) {
				if (readOnly)
					return failed("Coles did not load delivery times for every day.");
				continue;
			}
			const slots = helpers.readSlots(list());
			if (slots.length > 0) {
				seen.push({
					date: day.date as string,
					cheapest_fee: String(Math.min(...slots.map((s) => s.fee))),
					...(readOnly && {
						windows: slots.map((slot) => ({
							start: at(day.date as string, slot.start),
							end: at(day.date as string, slot.end),
							fee: String(slot.fee),
						})),
					}),
				});
			}
			if (readOnly) continue;
			const slot = helpers.chooseSlot(slots, wanted.time_of_day);
			if (!slot) continue;
			const radio = [
				...list().querySelectorAll<HTMLInputElement>('input[type="radio"]'),
			].find((candidate) => candidate.value === slot.value);
			const confirm = drawer.querySelector<HTMLButtonElement>(
				'button[aria-label="Confirm time slot"]',
			);
			if (!radio || !confirm) return failed("Coles changed its time picker.");
			radio.click();
			if (!(await wait(() => !confirm.disabled))) {
				return failed("Coles would not let the app confirm the time.");
			}
			confirm.click();
			const date = day.date as string;
			return {
				outcome: "reserved",
				window_label: slot.label,
				window_start: at(date, slot.start),
				window_end: at(date, slot.end),
				fee: String(slot.fee),
				problem:
					date !== wantedDate
						? `Nothing left on ${wantedDate}, so the next day with a time was used.`
						: undefined,
				days: seen,
			};
		}
		if (readOnly) return { outcome: "read", days: seen };
		return {
			...failed(
				wanted.time_of_day === "any"
					? "Coles has no delivery times left this week."
					: `Coles has no ${wanted.time_of_day} delivery times left this week.`,
			),
			days: seen,
		};
	} catch (error) {
		return failed(`Could not pick a delivery time: ${String(error)}`);
	}
}
