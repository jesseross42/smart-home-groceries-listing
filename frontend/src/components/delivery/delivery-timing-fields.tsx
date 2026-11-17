import { Input } from "#/components/ui/input";
import { Label } from "#/components/ui/label";
import { MAX_PERISHABLE_HOURS } from "#/lib/delivery-form";

/** Edits the deadline and the latest acceptable lead time for perishable items. */
export function DeliveryTimingFields({
	needBy,
	perishableHours,
	onNeedBy,
	onPerishableHours,
}: {
	needBy: string;
	perishableHours: string;
	onNeedBy: (value: string) => void;
	onPerishableHours: (value: string) => void;
}) {
	return (
		<fieldset className="grid gap-3 rounded-md border border-border p-3 sm:grid-cols-2">
			<legend className="px-1 text-sm font-medium">Delivery timing</legend>
			<div className="flex flex-col gap-1">
				<Label htmlFor="delivery-need-by">Need by (Sydney time)</Label>
				<Input
					id="delivery-need-by"
					type="datetime-local"
					step={1}
					value={needBy}
					onChange={(event) => onNeedBy(event.target.value)}
				/>
				<p className="text-xs text-muted-foreground">
					All delivery windows must end by this time. Leave blank for no
					deadline.
				</p>
			</div>
			<div className="flex flex-col gap-1">
				<Label htmlFor="delivery-perishable-hours">
					Perishables can arrive up to
				</Label>
				<div className="flex items-center gap-2">
					<Input
						id="delivery-perishable-hours"
						type="number"
						min={1}
						max={MAX_PERISHABLE_HOURS}
						disabled={!needBy}
						value={needBy ? perishableHours : ""}
						onChange={(event) => onPerishableHours(event.target.value)}
					/>
					<span className="text-sm">hours before need-by</span>
				</div>
				<p className="text-xs text-muted-foreground">
					Applies to items marked Perishable on the grocery list. Leave blank
					for no freshness limit.
				</p>
			</div>
		</fieldset>
	);
}
