import { SwitchButton } from "#/components/order-options/switch-button";
import { Badge } from "#/components/ui/badge";
import {
	Card,
	CardContent,
	CardFooter,
	CardHeader,
	CardTitle,
} from "#/components/ui/card";
import type { OrderOption } from "#/lib/api";
import { formatMoney } from "#/lib/money";
import { deliveryPhrase, optionNotes } from "#/lib/option-notes";

/**
 * One way to buy the order: its total, what each store's part costs with
 * delivery, the items it leaves out, and why it is not recommended when it
 * is not. Its switch button ("Switch to Coles", with an (i) note) makes the
 * order buy each item at this option's store, overriding the plan the page
 * picked; it is hidden for the stores the order buys from now.
 */
export function OptionCard({
	option,
	maxDeliverySpend,
	onUse,
}: {
	option: OrderOption;
	maxDeliverySpend: string | null;
	onUse: (option: OrderOption) => Promise<void>;
}) {
	const notes = optionNotes(option, maxDeliverySpend);

	return (
		<Card role="region" aria-label={option.label} className="gap-2">
			<CardHeader className="flex flex-row flex-wrap items-center justify-between gap-2">
				<CardTitle className="flex flex-wrap items-center gap-2">
					{option.label}
					{option.recommended && <Badge>Recommended</Badge>}
					{option.is_current && <Badge variant="secondary">Current</Badge>}
				</CardTitle>
				<span className="text-base font-semibold">
					{formatMoney(option.total)}
				</span>
			</CardHeader>
			<CardContent className="flex flex-col gap-2 text-sm">
				<ul className="flex list-none flex-col gap-1 p-0">
					{option.stores.map((store) => (
						<li key={store.store} className="flex justify-between gap-2">
							<span>
								{store.store_name}: {store.lines.length}{" "}
								{store.lines.length === 1 ? "item" : "items"}
							</span>
							<span className="text-muted-foreground">
								{formatMoney(store.subtotal)} + {deliveryPhrase(store)}
							</span>
						</li>
					))}
				</ul>
				{option.missing.length > 0 && (
					<ul
						aria-label="Items this option leaves out"
						className="flex list-none flex-col gap-1 p-0 text-muted-foreground"
					>
						{option.missing.map((item) => (
							<li key={item.grocery_item_id}>
								{item.name}: {item.reason}
							</li>
						))}
					</ul>
				)}
				{notes.map((note) => (
					<p key={note} className="text-xs text-muted-foreground">
						{note}
					</p>
				))}
			</CardContent>
			{!option.is_current && option.picks.length > 0 && (
				<CardFooter>
					<SwitchButton option={option} onUse={onUse} />
				</CardFooter>
			)}
		</Card>
	);
}
