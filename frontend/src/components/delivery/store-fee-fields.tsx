import { AmountField } from "#/components/delivery/amount-field";
import { Input } from "#/components/ui/input";
import { Label } from "#/components/ui/label";
import type { StoreFeeFields as Fields } from "#/lib/delivery-form";

/**
 * One store's delivery fee, free-delivery amount and minimum order, and for
 * Woolworths the saved address the bookmark picks before a time.
 */
export function StoreFeeFields({
	fields,
	onChange,
}: {
	fields: Fields;
	onChange: (fields: Fields) => void;
}) {
	const id = (name: string) => `${fields.store}-${name}`;
	return (
		<fieldset className="flex flex-col gap-3 rounded-md border border-border p-3">
			<legend className="px-1 text-sm font-medium">{fields.storeName}</legend>
			<div className="grid gap-3 sm:grid-cols-3">
				<AmountField
					id={id("fee")}
					label="Delivery fee"
					hint="What one delivery costs."
					value={fields.deliveryFee}
					onChange={(deliveryFee) => onChange({ ...fields, deliveryFee })}
				/>
				<AmountField
					id={id("free-over")}
					label="Free delivery from"
					hint="Orders this big or bigger deliver free. Empty if never."
					value={fields.freeDeliveryOver}
					onChange={(freeDeliveryOver) =>
						onChange({ ...fields, freeDeliveryOver })
					}
				/>
				<AmountField
					id={id("minimum")}
					label="Minimum order"
					hint="The store will not deliver a smaller order. Empty if none."
					value={fields.minimumOrder}
					onChange={(minimumOrder) => onChange({ ...fields, minimumOrder })}
				/>
			</div>
			{fields.store === "woolworths" && (
				<div className="flex flex-col gap-1">
					<Label htmlFor={id("address")}>Delivery address</Label>
					<Input
						id={id("address")}
						maxLength={200}
						placeholder="U 2 8 Athelstan Rd, CAMBERWELL VIC 3124"
						value={fields.deliveryAddress}
						onChange={(event) =>
							onChange({ ...fields, deliveryAddress: event.target.value })
						}
					/>
					<p className="text-xs text-muted-foreground">
						One of your saved Woolworths addresses, as Woolworths writes it. The
						bookmark picks it before a time. Empty keeps the account's.
					</p>
				</div>
			)}
		</fieldset>
	);
}
