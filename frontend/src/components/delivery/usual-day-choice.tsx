import { Button } from "#/components/ui/button";
import type { DeliveryDay } from "#/lib/api";
import { DELIVERY_WEEKDAYS } from "#/lib/delivery-days";

/** Monday first, as a week is read in Australia. */
const WEEK = [...DELIVERY_WEEKDAYS.slice(1), DELIVERY_WEEKDAYS[0]];

/**
 * The household's usual delivery day: the day the delivery chooser starts on
 * when a list is sent to a store. "No usual day" starts it on tomorrow.
 */
export function UsualDayChoice({
	value,
	onChange,
}: {
	value: DeliveryDay | null;
	onChange: (day: DeliveryDay | null) => void;
}) {
	const choices: { day: DeliveryDay | null; label: string }[] = [
		{ day: null, label: "No usual day" },
		...WEEK,
	];
	return (
		<fieldset className="flex flex-col gap-2">
			<legend className="text-sm font-medium">Usual delivery day</legend>
			<div className="flex flex-wrap gap-2">
				{choices.map((choice) => (
					<Button
						key={choice.label}
						type="button"
						size="sm"
						variant={value === choice.day ? "default" : "outline"}
						aria-pressed={value === choice.day}
						onClick={() => onChange(choice.day)}
					>
						{choice.label}
					</Button>
				))}
			</div>
			<p className="text-xs text-muted-foreground">
				Send to Woolworths starts on this day. You can pick another day each
				shop.
			</p>
		</fieldset>
	);
}
