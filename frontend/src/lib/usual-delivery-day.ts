/**
 * The household's usual delivery day, as a route loader needs it for the
 * Send to store sheet.
 */
import { api, type DeliveryDay } from "#/lib/api";

/**
 * Reads the usual delivery day. A failure gives none (the sheet then starts
 * on tomorrow) instead of an error: the list and the order matter more.
 */
export async function loadUsualDeliveryDay(): Promise<DeliveryDay | null> {
	try {
		return (await api.deliverySettings.get()).default_delivery_day;
	} catch {
		return null;
	}
}
