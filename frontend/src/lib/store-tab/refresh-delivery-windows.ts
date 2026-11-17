/** Self-contained programs for refreshing availability in a store tab. */
import type { StoreId } from "#/lib/api/products";
import type { ColesFillHelpers } from "#/lib/store-tab/fill-coles-trolley";
import type { FillTrolleyHelpers } from "#/lib/store-tab/fill-woolworths-trolley";
import type { pickColesDeliverySlot } from "#/lib/store-tab/pick-coles-delivery-slot";
import type {
	DeliveryReport,
	reserveWoolworthsDeliveryWindow,
} from "#/lib/store-tab/reserve-woolworths-delivery-window";
import type { FillTrolleyConfig } from "#/lib/store-tab/store-tab";

/** Sends a complete read-only snapshot to the planner. */
export async function publishDeliveryWindows(
	config: FillTrolleyConfig,
	store: StoreId,
	report: DeliveryReport,
): Promise<{ message: string }> {
	if (report.outcome !== "read") {
		return { message: report.problem ?? "Could not read delivery windows." };
	}
	const windows = (report.days ?? []).flatMap((day) => day.windows ?? []);
	const response = await fetch(
		`${config.apiBaseUrl}/api/store-tab/delivery-windows`,
		{
			method: "POST",
			headers: { "content-type": "text/plain;charset=UTF-8" },
			body: JSON.stringify({ secret: config.secret, store, windows }),
		},
	);
	return {
		message: response.ok
			? `Updated ${windows.length} delivery ${windows.length === 1 ? "window" : "windows"}. Return to Order and check again.`
			: `The grocery app did not accept the delivery times (HTTP ${response.status}).`,
	};
}

/** Reads regular delivery windows from the account's current address. */
export async function refreshWoolworthsWindows(
	config: FillTrolleyConfig,
	helpers: Pick<FillTrolleyHelpers, "chooseWindow"> & {
		reserveDeliveryWindow: typeof reserveWoolworthsDeliveryWindow;
		publishWindows: typeof publishDeliveryWindows;
	},
): Promise<{ message: string }> {
	if (!/(^|\.)woolworths\.com\.au$/.test(location.hostname)) {
		return {
			message: "Open woolworths.com.au first, then press the bookmark.",
		};
	}
	const report = await helpers.reserveDeliveryWindow(
		{ date: null, time_of_day: "any" },
		helpers.chooseWindow,
		true,
	);
	return helpers.publishWindows(config, "woolworths", report);
}

/** Reads available slots from the Coles delivery drawer. */
export async function refreshColesWindows(
	config: FillTrolleyConfig,
	helpers: Pick<ColesFillHelpers, "chooseSlot" | "readSlots"> & {
		pickDeliverySlot: typeof pickColesDeliverySlot;
		publishWindows: typeof publishDeliveryWindows;
	},
): Promise<{ message: string }> {
	if (!/(^|\.)coles\.com\.au$/.test(location.hostname)) {
		return { message: "Open coles.com.au first, then press the bookmark." };
	}
	const sydneyDate = new Intl.DateTimeFormat("en-AU", {
		timeZone: "Australia/Sydney",
		year: "numeric",
		month: "numeric",
		day: "numeric",
	}).formatToParts(new Date());
	const part = (type: Intl.DateTimeFormatPartTypes) =>
		Number(sydneyDate.find((entry) => entry.type === type)?.value);
	// The picker needs a calendar date, independent of the browser's time zone.
	const day = new Date(part("year"), part("month") - 1, part("day"), 12);
	const report = await helpers.pickDeliverySlot(
		{ date: null, time_of_day: "any" },
		{ readSlots: helpers.readSlots, chooseSlot: helpers.chooseSlot },
		day,
		true,
	);
	return helpers.publishWindows(config, "coles", report);
}
