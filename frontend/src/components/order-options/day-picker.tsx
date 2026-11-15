import { Button } from "#/components/ui/button";
import type { DeliveryDay, StoreDayFee } from "#/lib/api";
import { DELIVERY_WEEKDAYS } from "#/lib/delivery-days";
import { formatMoney } from "#/lib/money";

/** Monday first, as a week is read in Australia. */
const WEEK = [...DELIVERY_WEEKDAYS.slice(1), DELIVERY_WEEKDAYS[0]];

/**
 * The day the Order page costs delivery on, for this visit. It starts on the
 * usual day (Settings › Delivery). "Cheapest day" takes each store's cheapest
 * day. Below, the fees that came from the stores' own delivery times.
 */
export function DayPicker({
	value,
	storeFees,
	onChange,
}: {
	value: DeliveryDay | null;
	storeFees: StoreDayFee[];
	onChange: (day: DeliveryDay | "any") => void;
}) {
	const choices: { day: DeliveryDay | null; label: string }[] = [
		{ day: null, label: "Cheapest day" },
		...WEEK,
	];
	return (
		<fieldset className="flex flex-col gap-2">
			<legend className="text-sm font-medium">Delivery day</legend>
			<div className="flex flex-wrap gap-2">
				{choices.map((choice) => (
					<Button
						key={choice.label}
						type="button"
						size="sm"
						variant={choice.day === value ? "secondary" : "outline"}
						aria-pressed={choice.day === value}
						onClick={() => onChange(choice.day ?? "any")}
					>
						{choice.label}
					</Button>
				))}
			</div>
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
		</fieldset>
	);
}

/** "Tue 17 Nov" for "2026-11-17". */
function dayLabel(date: string): string {
	return new Date(`${date}T00:00:00`).toLocaleDateString("en-AU", {
		weekday: "short",
		day: "numeric",
		month: "short",
	});
}
