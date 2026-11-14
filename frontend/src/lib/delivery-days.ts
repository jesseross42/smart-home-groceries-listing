/**
 * The delivery days the household can pick from when sending the list to a
 * store: the next day (the default) and the six days after it.
 *
 * The first choice is sent as `date: null`, which means "the day after the
 * store's own today" — the store tab works it out on the store's website, so
 * a phone in another time zone cannot pick the wrong day. Later choices are
 * explicit dates in the household's local calendar.
 */
import type { DeliveryWeekday } from "#/lib/api";

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

/** The weekdays in the order `Date.getDay()` counts them. */
const WEEKDAYS_BY_INDEX: DeliveryWeekday[] = [
	"sunday",
	"monday",
	"tuesday",
	"wednesday",
	"thursday",
	"friday",
	"saturday",
];

/** The weekdays as Settings lists them, Monday first. */
export const WEEKDAY_CHOICES: { value: DeliveryWeekday; label: string }[] = [
	{ value: "monday", label: "Monday" },
	{ value: "tuesday", label: "Tuesday" },
	{ value: "wednesday", label: "Wednesday" },
	{ value: "thursday", label: "Thursday" },
	{ value: "friday", label: "Friday" },
	{ value: "saturday", label: "Saturday" },
	{ value: "sunday", label: "Sunday" },
];

/**
 * The `date` Send to store starts on: the next `weekday` after `today`, as
 * its day button sends it (`null` when that is tomorrow, or with no saved
 * weekday). A week of days is offered, so every weekday has one button.
 */
export function defaultDeliveryDate(
	weekday: DeliveryWeekday | null | undefined,
	today: Date,
): string | null {
	if (!weekday) return null;
	const daysAhead =
		((WEEKDAYS_BY_INDEX.indexOf(weekday) - today.getDay() + 6) % 7) + 1;
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
