/**
 * The day the Send to store sheet starts on, so the trolley is filled for
 * the day the plan was costed on.
 */
import type { DeliveryDay, OrderOptions, StoreId } from "#/lib/api";
import { DELIVERY_WEEKDAYS } from "#/lib/delivery-days";

/**
 * The weekday of the store's cheapest day the plan used, else the plan's
 * day, else the household's usual day.
 */
export function sendStartDay(
	plan: Pick<OrderOptions, "delivery_day" | "store_fees">,
	store: StoreId,
	usualDay: DeliveryDay | null,
): DeliveryDay | null {
	const fee = plan.store_fees.find((f) => f.store === store);
	if (fee) {
		const weekday = new Date(`${fee.date}T00:00:00`).getDay();
		return DELIVERY_WEEKDAYS[weekday].day;
	}
	return plan.delivery_day ?? usualDay;
}
