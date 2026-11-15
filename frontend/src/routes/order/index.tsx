import {
	createFileRoute,
	Link,
	useNavigate,
	useRouter,
} from "@tanstack/react-router";
import { useState } from "react";

import { DislikeMarksProvider } from "#/components/dislikes/dislike-mark";
import { OrderReview } from "#/components/order/order-review";
import { DayPicker } from "#/components/order-options/day-picker";
import { ModePicker } from "#/components/order-options/mode-picker";
import { OrderOptions } from "#/components/order-options/order-options";
import { SendToStore } from "#/components/trolley/send-to-store";
import { Button } from "#/components/ui/button";
import { useApplyBestOption } from "#/hooks/useApplyBestOption";
import { useListSync } from "#/hooks/useListSync";
import { useOrderStores } from "#/hooks/useOrderStores";
import { api, type OrderMode, type PlanDay, selectionsByItem } from "#/lib/api";
import { chosenAtStore } from "#/lib/chosen-at-store";
import { DELIVERY_WEEKDAYS } from "#/lib/delivery-days";
import { loadDislikeMarks } from "#/lib/dislike-marks";
import { ORDER_MODES } from "#/lib/order-modes";
import { loadUsualDeliveryDay } from "#/lib/usual-delivery-day";

/**
 * `?mode=` ranks the options another way for this visit; `?day=` costs
 * delivery on another day (`any` = each store's cheapest).
 */
interface OrderSearch {
	mode?: OrderMode;
	day?: PlanDay;
}

const PLAN_DAYS: readonly string[] = [
	...DELIVERY_WEEKDAYS.map((d) => d.day),
	"any",
];

export const Route = createFileRoute("/order/")({
	validateSearch: (search: Record<string, unknown>): OrderSearch => ({
		...(ORDER_MODES.some((m) => m.mode === search.mode)
			? { mode: search.mode as OrderMode }
			: {}),
		...(typeof search.day === "string" && PLAN_DAYS.includes(search.day)
			? { day: search.day as PlanDay }
			: {}),
	}),
	loaderDeps: ({ search }) => search,
	// The order re-priced now, its options with delivery, plus the list and
	// choices so the Send button counts exactly what a handoff would send.
	loader: async ({ deps }) => {
		const [review, plan, items, selections, dislikeMarks, usualDay] =
			await Promise.all([
				api.orderReview.get(),
				api.orderOptions.get(deps.mode, deps.day),
				api.grocery.list(),
				api.selections.list(),
				loadDislikeMarks(),
				loadUsualDeliveryDay(),
			]);
		const chosen = selectionsByItem(selections);
		return {
			review,
			plan,
			chosen,
			dislikeMarks,
			usualDay,
			chosenAtWoolworths: chosenAtStore(items, chosen, "woolworths"),
			chosenAtColes: chosenAtStore(items, chosen, "coles"),
		};
	},
	pendingComponent: () => (
		<p className="text-sm text-muted-foreground">Checking today's prices…</p>
	),
	errorComponent: OrderError,
	component: OrderPage,
});

/**
 * `/order` › In Progress — the order review and its options.
 *
 * "Ways to buy" ranks every way to split the order between the stores, with
 * delivery fees, by the household's mode; the switch button moves items between
 * the products the household chose (with Undo). Below, every committed item
 * with the product the order buys now, priced again at its store, grouped by
 * store with subtotals. Items with no product, and products a store no
 * longer sells, send the household back to the list to choose. Nothing here
 * chooses a product for them.
 *
 * "Check prices again" re-runs the loader; the backend answers most of it
 * from its 10-minute search cache.
 */
function OrderPage() {
	const {
		review,
		plan,
		chosen,
		dislikeMarks,
		usualDay,
		chosenAtWoolworths,
		chosenAtColes,
	} = Route.useLoaderData();
	const router = useRouter();
	useListSync();
	const navigate = useNavigate({ from: Route.fullPath });
	const orderStores = useOrderStores(chosen);
	useApplyBestOption(plan, orderStores.use);
	const [checking, setChecking] = useState(false);

	async function checkAgain() {
		setChecking(true);
		try {
			await router.invalidate({ sync: true });
		} finally {
			setChecking(false);
		}
	}

	return (
		<div className="flex flex-col gap-4">
			<div className="flex flex-wrap items-center justify-between gap-2">
				<h1 className="text-lg font-semibold">In Progress</h1>
				<Button
					variant="outline"
					size="sm"
					disabled={checking}
					onClick={checkAgain}
				>
					{checking ? "Checking…" : "Check prices again"}
				</Button>
			</div>

			{plan.options.length > 0 && (
				<ModePicker
					value={plan.mode}
					onChange={(mode) =>
						navigate({ search: (prev) => ({ ...prev, mode }) })
					}
				/>
			)}

			{plan.options.length > 0 && (
				<DayPicker
					value={plan.delivery_day}
					storeFees={plan.store_fees}
					onChange={(day) => navigate({ search: (prev) => ({ ...prev, day }) })}
				/>
			)}

			<OrderOptions plan={plan} onUse={orderStores.use}>
				<p className="text-sm">
					Some delivery fees are not set yet.{" "}
					<Link to="/settings/delivery" className="underline">
						Set delivery fees
					</Link>
				</p>
			</OrderOptions>

			{review.stores.length > 0 && (
				<h2 className="text-base font-semibold">What the order buys now</h2>
			)}

			<DislikeMarksProvider value={dislikeMarks}>
				<OrderReview review={review}>
					<OrderReview.Empty>
						<Button asChild variant="outline">
							<Link to="/">Go to the grocery list</Link>
						</Button>
					</OrderReview.Empty>
					<OrderReview.Unchosen>
						<Button asChild variant="outline" className="w-full sm:w-auto">
							<Link to="/">Choose products on the list</Link>
						</Button>
					</OrderReview.Unchosen>
					<OrderReview.Store store="woolworths">
						<SendToStore
							store="woolworths"
							chosenCount={chosenAtWoolworths}
							usualDay={plan.delivery_day ?? usualDay}
						>
							<SendToStore.Trigger />
							<SendToStore.Content />
						</SendToStore>
					</OrderReview.Store>
					<OrderReview.Store store="coles">
						<SendToStore
							store="coles"
							chosenCount={chosenAtColes}
							usualDay={plan.delivery_day ?? usualDay}
						>
							<SendToStore.Trigger />
							<SendToStore.Content />
						</SendToStore>
					</OrderReview.Store>
					<OrderReview.Total />
				</OrderReview>
			</DislikeMarksProvider>
		</div>
	);
}

/** The order could not be read; offer to try again. */
function OrderError() {
	const router = useRouter();
	return (
		<div role="alert" className="flex flex-col items-start gap-3 text-sm">
			<p>Could not check the order just now.</p>
			<Button variant="outline" onClick={() => router.invalidate()}>
				Try again
			</Button>
		</div>
	);
}
