/**
 * Every store the household can send a trolley to, as a {@link StoreTab}.
 *
 * The one place that names a store's fill script, so the rest of the web app
 * picks a store by its id and nothing else.
 */
import type { StoreId } from "#/lib/api/products";
import {
	chooseColesSlot,
	readColesSlots,
} from "#/lib/store-tab/choose-coles-slot";
import { chooseWoolworthsWindow } from "#/lib/store-tab/choose-woolworths-window";
import { fillColesTrolley } from "#/lib/store-tab/fill-coles-trolley";
import { fillWoolworthsTrolley } from "#/lib/store-tab/fill-woolworths-trolley";
import { pickColesDeliverySlot } from "#/lib/store-tab/pick-coles-delivery-slot";
import { pickWoolworthsAddress } from "#/lib/store-tab/pick-woolworths-address";
import { reserveWoolworthsDeliveryWindow } from "#/lib/store-tab/reserve-woolworths-delivery-window";
import type { StoreTab } from "#/lib/store-tab/store-tab";

/** Woolworths: fills the trolley and reserves the delivery time. */
export const WOOLWORTHS_TAB: StoreTab = {
	store: "woolworths",
	storeName: "Woolworths",
	trolleyUrl: "https://www.woolworths.com.au/shop/mytrolley",
	bookmarkName: "Fill Woolworths trolley",
	reservesDelivery: true,
	fill: fillWoolworthsTrolley,
	helpers: {
		chooseWindow: chooseWoolworthsWindow,
		pickAddress: pickWoolworthsAddress,
		reserveDeliveryWindow: reserveWoolworthsDeliveryWindow,
	},
};

/** Coles: fills the trolley and picks the delivery time in Coles' picker. */
export const COLES_TAB: StoreTab = {
	store: "coles",
	storeName: "Coles",
	trolleyUrl: "https://www.coles.com.au/",
	bookmarkName: "Fill Coles trolley",
	reservesDelivery: true,
	fill: fillColesTrolley,
	helpers: {
		readSlots: readColesSlots,
		chooseSlot: chooseColesSlot,
		pickDeliverySlot: pickColesDeliverySlot,
	},
};

/** Each store's trolley handoff, by store id. */
export const STORE_TABS: Record<StoreId, StoreTab> = {
	woolworths: WOOLWORTHS_TAB,
	coles: COLES_TAB,
};
