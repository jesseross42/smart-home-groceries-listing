/**
 * Reserves a Woolworths delivery window from inside the household's
 * logged-in woolworths.com.au tab, before any product is added.
 *
 * Calls (live, 2026-10-02):
 * - `GET /apis/ui/Delivery/DeliveryInfo` — the account's address (`AddressId`,
 *   `AreaId`), method (`Courier` = delivery), the store's "today", and the
 *   window already reserved (`ReservedTime`).
 * - `GET /api/v3/ui/fulfilment/windows?areaId&fulfilmentMethod&addressId` —
 *   `Days[].Times[]` for about a week.
 * - `POST /apis/ui/Fulfilment` `{addressId, fulfilmentMethod, timeslotId,
 *   windowDate}` — reserves one; answers `{IsSuccessful}`.
 *
 * Every day's cheapest open time is returned as `days`, also when nothing new
 * is reserved, so the planner can cost delivery per day.
 *
 * A window already reserved on the right day and part of the day is kept, so
 * a time the household picked on Woolworths is not overwritten. The window
 * can always be changed on Woolworths later.
 *
 * Must stay **self-contained**: the bookmarklet carries its source text, and
 * the window rule arrives as the `chooseWindow` argument
 * (`chooseWoolworthsWindow`).
 */
import type {
	ChosenWindow,
	DeliveryWanted,
	WoolworthsDay,
} from "#/lib/store-tab/choose-woolworths-window";

/** What happened to the delivery window, in the backend's report shape. */
export interface DeliveryReport {
	outcome: "reserved" | "kept" | "failed" | "read";
	window_label?: string;
	/** Store-local, "YYYY-MM-DDTHH:MM:SS". */
	window_start?: string;
	window_end?: string;
	/** Decimal string. */
	fee?: string;
	problem?: string;
	/**
	 * The cheapest fee of each day's open times, for the planner (issue
	 * #70). Sent beside the report as `delivery_days`, not inside it.
	 */
	days?: DayFee[];
}

/** One day's cheapest open delivery time. */
export interface DayFee {
	/** "YYYY-MM-DD". */
	date: string;
	/** Decimal string. */
	cheapest_fee: string;
	windows?: ReportedWindow[];
}

/** An available regular delivery window in store-local time. */
export interface ReportedWindow {
	start: string;
	end: string;
	fee: string;
}

type ChooseWindow = (
	days: WoolworthsDay[],
	wanted: DeliveryWanted,
	storeToday: string,
) => ChosenWindow | null;

/** Reserves (or keeps) the delivery window `wanted` asks for. Never throws. */
export async function reserveWoolworthsDeliveryWindow(
	wanted: DeliveryWanted,
	chooseWindow: ChooseWindow,
	readOnly = false,
): Promise<DeliveryReport> {
	const JSON_HEADERS = {
		accept: "application/json, text/plain, */*",
		"content-type": "application/json",
	};
	const seconds = (dateTime: string) => dateTime.slice(0, 19);
	const failed = (problem: string): DeliveryReport => ({
		outcome: "failed",
		problem: problem.slice(0, 300),
	});
	const days: DayFee[] = [];
	const withDays = (report: DeliveryReport): DeliveryReport => ({
		...report,
		days,
	});
	try {
		const info = await fetch("/apis/ui/Delivery/DeliveryInfo", {
			credentials: "include",
			headers: JSON_HEADERS,
			cache: "no-store",
		}).then((r) => (r.ok ? r.json() : null));
		const address = info?.Address;
		if (!address?.AddressId) {
			return failed("Add a delivery address on Woolworths first.");
		}
		const method = info.DeliveryMethod || "Courier";
		const storeToday = String(info.CurrentDateAtFulfilmentStore ?? "").slice(
			0,
			10,
		);

		const query = new URLSearchParams({
			areaId: String(address.AreaId),
			fulfilmentMethod: method,
			addressId: String(address.AddressId),
		});
		const listed = await fetch(`/api/v3/ui/fulfilment/windows?${query}`, {
			credentials: "include",
			headers: JSON_HEADERS,
			cache: "no-store",
		}).then((r) => (r.ok ? r.json() : null));
		if (readOnly && (!Array.isArray(listed?.Days) || method !== "Courier")) {
			return failed(
				"Woolworths did not show delivery windows. Check that delivery is selected.",
			);
		}
		const listedDays: WoolworthsDay[] = listed?.Days ?? [];
		for (const day of listedDays) {
			const fees = (day.Times ?? [])
				.filter((time) => time.Available)
				.map((time) => Number(time.SalePrice))
				.filter((fee) => Number.isFinite(fee) && fee >= 0);
			if (!day.Available || fees.length === 0) continue;
			days.push({
				date: String(day.Date).slice(0, 10),
				cheapest_fee: String(Math.min(...fees)),
				...(readOnly && {
					windows: (day.Times ?? [])
						.filter(
							(time) =>
								time.Available &&
								!time.IsExpress &&
								Number.isFinite(time.SalePrice) &&
								time.SalePrice >= 0 &&
								typeof time.StartDateTime === "string" &&
								typeof time.EndDateTime === "string" &&
								seconds(time.EndDateTime) > seconds(time.StartDateTime),
						)
						.map((time) => ({
							start: seconds(time.StartDateTime),
							end: seconds(time.EndDateTime),
							fee: String(time.SalePrice),
						})),
				}),
			});
		}

		if (readOnly) return { outcome: "read", days };

		const reserved = info.ReservedTime;
		if (reserved?.Id && reserved.StartDateTime) {
			const asDay = [
				{
					Date: reserved.StartDateTime,
					Available: true,
					Times: [
						{
							Id: reserved.Id,
							Available: true,
							TimeWindow: reserved.TimeWindow ?? "",
							StartDateTime: reserved.StartDateTime,
							EndDateTime: reserved.EndDateTime,
							SalePrice: 0,
						},
					],
				},
			];
			const fits = chooseWindow(asDay, wanted, storeToday);
			if (fits && !fits.movedToLaterDay) {
				return {
					outcome: "kept",
					window_label: reserved.TimeWindow,
					window_start: seconds(reserved.StartDateTime),
					window_end: seconds(reserved.EndDateTime),
					days,
				};
			}
		}

		const chosen = chooseWindow(listedDays, wanted, storeToday);
		if (!chosen) {
			return withDays(
				failed(
					wanted.time_of_day === "any"
						? "Woolworths has no delivery times left this week."
						: `Woolworths has no ${wanted.time_of_day} delivery times left this week.`,
				),
			);
		}

		const answer = await fetch("/apis/ui/Fulfilment", {
			method: "POST",
			credentials: "include",
			headers: JSON_HEADERS,
			body: JSON.stringify({
				addressId: address.AddressId,
				fulfilmentMethod: method,
				timeslotId: chosen.window.Id,
				windowDate: chosen.date,
			}),
		}).then((r) => (r.ok ? r.json() : null));
		if (!answer?.IsSuccessful) {
			return withDays(
				failed(
					answer?.Message || "Woolworths did not reserve the delivery time.",
				),
			);
		}
		return {
			outcome: "reserved",
			window_label: chosen.window.TimeWindow,
			window_start: seconds(chosen.window.StartDateTime),
			window_end: seconds(chosen.window.EndDateTime),
			fee: String(chosen.window.SalePrice),
			problem: chosen.movedToLaterDay
				? `Nothing left on ${chosen.wantedDate}, so the next day with a time was used.`
				: undefined,
			days,
		};
	} catch (error) {
		return failed(`Could not reserve a delivery time: ${String(error)}`);
	}
}
