import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from "#/components/ui/select";
import type { DeliveryDay, StoreDayFee } from "#/lib/api";
import { DELIVERY_WEEKDAYS } from "#/lib/delivery-days";
import { formatMoney } from "#/lib/money";

/** Most days ahead; the stores list about a week of times. */
export const MAX_DAYS_AHEAD = 7;

/** Every choice of days ahead, from today (0) to the most. */
const DAYS_AHEAD = Array.from(
	{ length: MAX_DAYS_AHEAD + 1 },
	(_, days) => days,
);

/**
 * How soon the delivery must come, for this visit: today only, or within 1
 * to 7 days, each with the last day's name (Jesse, 2026-11-15). The plan
 * takes each store's cheapest day in that range. When the household has a
 * usual day and nothing is picked, that day is shown first. Below, the fees
 * that came from the stores' own delivery times.
 */
export function DayPicker({
	withinDays,
	usualDay,
	storeFees,
	usesWindows = false,
	today = new Date(),
	onChange,
}: {
	withinDays: number | null;
	/** The usual day the plan used, when no range was picked. */
	usualDay: DeliveryDay | null;
	storeFees: StoreDayFee[];
	usesWindows?: boolean;
	today?: Date;
	onChange: (withinDays: number) => void;
}) {
	const usual = withinDays === null && usualDay !== null;
	const value = usual ? "usual" : String(withinDays ?? MAX_DAYS_AHEAD);
	return (
		<div className="flex flex-col gap-2">
			<span id="delivery-within" className="text-sm font-medium">
				Delivery comes
			</span>
			<Select
				value={value}
				onValueChange={(choice) => {
					if (choice !== "usual") onChange(Number(choice));
				}}
			>
				<SelectTrigger
					aria-labelledby="delivery-within"
					size="sm"
					className="w-full sm:w-72"
				>
					<SelectValue />
				</SelectTrigger>
				<SelectContent>
					{usual && (
						<SelectItem value="usual">
							On the usual day ({weekdayName(usualDay)})
						</SelectItem>
					)}
					{DAYS_AHEAD.map((days) => (
						<SelectItem key={days} value={String(days)}>
							{withinLabel(days, today)}
						</SelectItem>
					))}
				</SelectContent>
			</Select>
			{!usesWindows && (
				<p className="text-xs text-muted-foreground">
					{storeFees.length === 0
						? "Fees from Settings. The store's own fees show after the bookmark has run."
						: storeFees
								.map(
									(fee) =>
										`${fee.store_name} ${formatMoney(fee.fee)} on ${dayLabel(fee.date)}`,
								)
								.join(" · ")}
				</p>
			)}
		</div>
	);
}

/** "Today (Sunday)" or "Within 2 days, by Tuesday 17 Nov". */
export function withinLabel(days: number, today: Date): string {
	const last = new Date(today);
	last.setDate(today.getDate() + days);
	const name = last.toLocaleDateString("en-AU", { weekday: "long" });
	if (days === 0) return `Today (${name})`;
	const date = last.toLocaleDateString("en-AU", {
		day: "numeric",
		month: "short",
	});
	return `Within ${days} ${days === 1 ? "day" : "days"}, by ${name} ${date}`;
}

/** "Tuesday" for "tuesday". */
function weekdayName(day: DeliveryDay | null): string {
	return DELIVERY_WEEKDAYS.find((d) => d.day === day)?.label ?? "";
}

/** "Tue 17 Nov" for "2026-11-17". */
function dayLabel(date: string): string {
	return new Date(`${date}T00:00:00`).toLocaleDateString("en-AU", {
		weekday: "short",
		day: "numeric",
		month: "short",
	});
}
