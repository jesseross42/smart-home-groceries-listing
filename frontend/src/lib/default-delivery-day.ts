/**
 * Reads the household's saved delivery weekday for a page loader.
 *
 * A page with Send to store still works without it, so a failed read means
 * "no saved day" (tomorrow) rather than a broken page.
 */
import { api, type DeliveryWeekday } from "#/lib/api";

/** The saved weekday, or `null` when none is saved or it cannot be read. */
export async function loadDefaultDeliveryDay(): Promise<DeliveryWeekday | null> {
	try {
		const settings = await api.deliverySettings.get();
		return settings.default_delivery_day ?? null;
	} catch {
		return null;
	}
}
