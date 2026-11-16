import { Button } from "#/components/ui/button";
import type { DeliveryDay } from "#/lib/api";
import { DELIVERY_WEEKDAYS } from "#/lib/delivery-days";

/** Monday first, as a week is read in Australia. */
const WEEK = [...DELIVERY_WEEKDAYS.slice(1), DELIVERY_WEEKDAYS[0]];

/**
 * The weekdays the household can take a delivery on. Each day is a toggle;
 * none pressed means any day. The order planner only uses these days.
 */
export function AllowedDaysChoice({
	value,
	onToggle,
}: {
	value: DeliveryDay[];
	onToggle: (day: DeliveryDay) => void;
}) {
	return (
		<fieldset className="flex flex-col gap-2">
			<legend className="text-sm font-medium">Allowed delivery days</legend>
			<div className="flex flex-wrap gap-2">
				{WEEK.map(({ day, label }) => (
					<Button
						key={day}
						type="button"
						size="sm"
						variant={value.includes(day) ? "default" : "outline"}
						aria-pressed={value.includes(day)}
						onClick={() => onToggle(day)}
					>
						{label}
					</Button>
				))}
			</div>
			<p className="text-xs text-muted-foreground">
				{value.length === 0
					? "Any day. Pick days to only plan deliveries on them."
					: "The order is only planned for these days."}
			</p>
		</fieldset>
	);
}
