/**
 * Settings › Member prices: `GET` and `PUT /api/store-memberships`.
 *
 * Whether the household is a member of each store's rewards program. Only a
 * member household sees and pays member prices.
 */
import { request } from "#/lib/api/client";
import type { StoreId } from "#/lib/api/products";

/** The household's membership at one store with a rewards program. */
export interface StoreMembership {
	store: StoreId;
	store_name: string;
	/** The rewards program, e.g. "Everyday Rewards". */
	program: string;
	member: boolean;
}

/** Every store with a rewards program. */
export interface StoreMemberships {
	stores: StoreMembership[];
}

/** One membership to save. */
export interface StoreMembershipChange {
	store: StoreId;
	member: boolean;
}

export const storeMembershipsApi = {
	/** Every store with a rewards program, in display order. */
	get: () => request<StoreMemberships>("/api/store-memberships"),
	/** Saves `stores` and answers with every membership as it now stands. */
	save: (stores: StoreMembershipChange[]) =>
		request<StoreMemberships>("/api/store-memberships", {
			method: "PUT",
			body: JSON.stringify({ stores }),
		}),
};
