/**
 * The delivery days the household can pick from when sending the list to a
 * store: the next day (the default) and the six days after it.
 *
 * The first choice is sent as `date: null`, which means "the day after the
 * store's own today" — the store tab works it out on the store's website, so
 * a phone in another time zone cannot pick the wrong day. Later choices are
 * explicit dates in the household's local calendar.
 */
import type { DeliveryDay } from "#/lib/api";

/** Every weekday, in `Date.getDay()` order, with the word a person reads. */
export const DELIVERY_WEEKDAYS: { day: DeliveryDay; label: string }[] = [
	{ day: "sunday", label: "Sunday" },
	{ day: "monday", label: "Monday" },
	{ day: "tuesday", label: "Tuesday" },
	{ day: "wednesday", label: "Wednesday" },
	{ day: "thursday", label: "Thursday" },
	{ day: "friday", label: "Friday" },
	{ day: "saturday", label: "Saturday" },
];

/** One day button. */
export interface DeliveryDayChoice {
	/** `null` = the store's next day. */
	date: string | null;
	label: string;
}

/** How many days are offered. Stores list about a week of windows. */
export const DELIVERY_DAYS_OFFERED = 7;

/** "YYYY-MM-DD" for a local calendar day. */
export function localDate(day: Date): string {
	const pad = (n: number) => String(n).padStart(2, "0");
	return `${day.getFullYear()}-${pad(day.getMonth() + 1)}-${pad(day.getDate())}`;
}

/** The day choices, starting tomorrow, from `today` (local time). */
export function deliveryDayChoices(
	today: Date,
	count = DELIVERY_DAYS_OFFERED,
): DeliveryDayChoice[] {
	const name = (day: Date) =>
		day.toLocaleDateString("en-AU", {
			weekday: "short",
			day: "numeric",
			month: "short",
		});
	return Array.from({ length: count }, (_, offset) => {
		const day = new Date(
			today.getFullYear(),
			today.getMonth(),
			today.getDate() + offset + 1,
		);
		return offset === 0
			? { date: null, label: `Tomorrow (${name(day)})` }
			: { date: localDate(day), label: name(day) };
	});
}

/**
 * The choice to start on for the household's usual delivery day: the next
 * time that weekday comes, from tomorrow. `null` (tomorrow) with no usual day.
 */
export function usualDeliveryDate(
	today: Date,
	usualDay: DeliveryDay | null,
): string | null {
	if (!usualDay) return null;
	const wanted = DELIVERY_WEEKDAYS.findIndex((w) => w.day === usualDay);
	const daysAhead = ((wanted - today.getDay() + 6) % 7) + 1;
	if (daysAhead === 1) return null;
	return localDate(
		new Date(
			today.getFullYear(),
			today.getMonth(),
			today.getDate() + daysAhead,
		),
	);
}

/** "Sat 3 Oct" for a store-local "YYYY-MM-DDTHH:MM:SS". */
export function storeDayLabel(dateTime: string): string {
	const [year, month, day] = dateTime.slice(0, 10).split("-").map(Number);
	return new Date(year, month - 1, day).toLocaleDateString("en-AU", {
		weekday: "short",
		day: "numeric",
		month: "short",
	});
}
