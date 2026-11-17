/**
 * The order planner: `GET /api/order-options`.
 *
 * The ways to buy the committed list — all at Woolworths, all at Coles, the
 * best mix, and the stores the order buys from now — each with delivery fees
 * from Settings › Delivery, best first under the mode. Using one sends its
 * `picks` to `api.selections.buyAt`. Money is a decimal string.
 */
import { request } from "#/lib/api/client";
import type { DeliveryDay, OrderMode } from "#/lib/api/delivery-settings";
import type { OrderLine, UnchosenItem } from "#/lib/api/order-review";
import type { StoreId } from "#/lib/api/products";
import type { OrderStorePick } from "#/lib/api/selections";

/** Which way of buying an option is. */
export type OrderOptionKind = "split" | "woolworths" | "coles" | "current";

/** What an option buys at one store, with its delivery. */
export interface OptionStore {
	store: StoreId;
	store_name: string;
	lines: OrderLine[];
	subtotal: string;
	delivery_fee: string;
	/** The fee is set in Settings › Delivery (otherwise counted as $0). */
	fee_known: boolean;
	free_delivery: boolean;
	/** The store will not deliver an order this small. */
	below_minimum: boolean;
	minimum_order: string | null;
	/** The live window used for timing and delivery cost. */
	delivery_window?: DeliveryWindow | null;
	delivery_problem?: StoreTimingProblem | null;
}

/** A live delivery window, in Sydney local time. */
export interface DeliveryWindow {
	start: string;
	end: string;
	fee: string;
}

export type StoreTimingProblem =
	| "windows_unavailable"
	| "no_matching_window"
	| "need_by_passed";

/** An item an option cannot buy, and why. */
export interface MissingItem {
	grocery_item_id: string;
	name: string;
	reason: string;
}

/** A disliked product an option buys, and who dislikes it. */
export interface DislikedPick {
	grocery_item_id: string;
	name: string;
	store: StoreId;
	store_name: string;
	disliked_by: string[];
	/** Another store has a product for the item that nobody dislikes. */
	avoidable: boolean;
}

/** One way to buy the order. */
export interface OrderOption {
	kind: OrderOptionKind;
	label: string;
	/** The best, and it can be placed as it is. */
	recommended: boolean;
	/** The order buys from these stores now. */
	is_current: boolean;
	stores: OptionStore[];
	missing: MissingItem[];
	disliked: DislikedPick[];
	items_total: string;
	delivery_total: string;
	total: string;
	complete: boolean;
	fees_known: boolean;
	meets_minimums: boolean;
	within_delivery_cap: boolean;
	picks: OrderStorePick[];
}

/** Every option, best first. */
export interface OrderOptions {
	mode: OrderMode;
	options: OrderOption[];
	unchosen: UnchosenItem[];
	/** The best mix was proven best (every combination tried). */
	exact: boolean;
	max_delivery_spend: string | null;
	/** The day delivery is costed for; `null` = each store's cheapest day. */
	delivery_day: DeliveryDay | null;
	/** Delivery comes within this many days (0 = today), when asked. */
	within_days: number | null;
	/** The stores' own fees the plan used instead of the typed ones. */
	store_fees: StoreDayFee[];
	/** The weekdays a delivery may come on; empty = any day. */
	allowed_delivery_days: DeliveryDay[];
	need_by?: string | null;
	/** Set when the delivery asked for cannot come on an allowed day. */
	delivery_problem: DeliveryProblem | null;
}

/** Why the delivery asked for breaks the allowed delivery days. */
export type DeliveryProblem = "day_not_allowed" | "no_allowed_day_in_range";

/** The cheapest fee a store showed for a day (from the bookmark). */
export interface StoreDayFee {
	store: StoreId;
	store_name: string;
	/** "YYYY-MM-DD". */
	date: string;
	fee: string;
}

export const orderOptionsApi = {
	/**
	 * The options, ranked by `mode` or the saved one, with delivery within
	 * `within` days (0 = today) or on the usual day.
	 */
	get: (mode?: OrderMode, within?: number) => {
		const query = new URLSearchParams();
		if (mode) query.set("mode", mode);
		if (within !== undefined) query.set("within", String(within));
		const text = query.toString();
		return request<OrderOptions>(`/api/order-options${text ? `?${text}` : ""}`);
	},
};
