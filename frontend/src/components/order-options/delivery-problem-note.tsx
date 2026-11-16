import type { DeliveryDay, DeliveryProblem } from "#/lib/api";
import { DELIVERY_WEEKDAYS } from "#/lib/delivery-days";

/**
 * Says that the delivery asked for cannot come on an allowed delivery day
 * (Settings › Delivery), and which days are allowed. No option is
 * recommended while this shows.
 */
export function DeliveryProblemNote({
	problem,
	allowedDays,
}: {
	problem: DeliveryProblem;
	allowedDays: DeliveryDay[];
}) {
	const allowed = allowedDays
		.map((day) => DELIVERY_WEEKDAYS.find((d) => d.day === day)?.label ?? day)
		.join(", ");
	return (
		<p role="alert" className="text-sm text-destructive">
			{problem === "day_not_allowed"
				? "That day is not an allowed delivery day."
				: "No allowed delivery day comes that soon."}{" "}
			Allowed: {allowed}. Nothing is recommended.
		</p>
	);
}
