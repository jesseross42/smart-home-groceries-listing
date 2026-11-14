import { Label } from "#/components/ui/label";
import { RadioGroup, RadioGroupItem } from "#/components/ui/radio-group";
import type { DeliveryWeekday } from "#/lib/api";
import { WEEKDAY_CHOICES } from "#/lib/delivery-days";

/** The radio value for "no saved day". */
const NO_DAY = "none";

/**
 * The delivery day Send to store starts on: a weekday, or none (tomorrow).
 * The day can still be changed for each order.
 */
export function DefaultDayChoice({
	value,
	onChange,
}: {
	value: DeliveryWeekday | null;
	onChange: (day: DeliveryWeekday | null) => void;
}) {
	const choices = [{ value: NO_DAY, label: "Tomorrow" }, ...WEEKDAY_CHOICES];
	return (
		<fieldset className="flex flex-col gap-2">
			<legend className="text-sm font-medium">Usual delivery day</legend>
			<p className="text-xs text-muted-foreground">
				Send to store starts on this day. You can change it for each order.
			</p>
			<RadioGroup
				value={value ?? NO_DAY}
				onValueChange={(day) =>
					onChange(day === NO_DAY ? null : (day as DeliveryWeekday))
				}
				className="flex flex-wrap gap-x-4 gap-y-2"
			>
				{choices.map((choice) => (
					<div key={choice.value} className="flex items-center gap-2">
						<RadioGroupItem id={`day-${choice.value}`} value={choice.value} />
						<Label htmlFor={`day-${choice.value}`}>{choice.label}</Label>
					</div>
				))}
			</RadioGroup>
		</fieldset>
	);
}
