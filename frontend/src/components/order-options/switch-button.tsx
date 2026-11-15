import { Info } from "lucide-react";
import { useState } from "react";

import { Button } from "#/components/ui/button";
import {
	Tooltip,
	TooltipContent,
	TooltipTrigger,
} from "#/components/ui/tooltip";
import type { OrderOption } from "#/lib/api";

/** Short button words per kind of option (Jesse, 2026-11-15). */
const WORDS: Record<OrderOption["kind"], string> = {
	woolworths: "Switch to Woolworths",
	coles: "Switch to Coles",
	split: "Switch to split",
	current: "Switch back",
};

/** What pressing the button does, for the (i) note. */
const NOTES: Record<OrderOption["kind"], string> = {
	woolworths:
		"Buys every item at Woolworths, with the products already chosen there. One delivery.",
	coles:
		"Buys every item at Coles, with the products already chosen there. One delivery.",
	split:
		"Buys each item at the store where it costs least with delivery. Two deliveries.",
	current: "Goes back to the stores the order used before.",
};

/**
 * Moves the order to `option`, overriding the plan the Order page picked by
 * itself. An (i) next to it says what it does, on hover or focus. Undo is in
 * the toast that follows.
 */
export function SwitchButton({
	option,
	onUse,
}: {
	option: OrderOption;
	onUse: (option: OrderOption) => Promise<void>;
}) {
	const [using, setUsing] = useState(false);

	async function use() {
		setUsing(true);
		try {
			await onUse(option);
		} finally {
			setUsing(false);
		}
	}

	return (
		<div className="flex w-full items-center gap-2 sm:w-auto">
			<Button
				size="sm"
				variant={option.recommended ? "default" : "outline"}
				className="flex-1 sm:flex-none"
				disabled={using}
				onClick={use}
			>
				{using ? "Switching…" : WORDS[option.kind]}
			</Button>
			<Tooltip>
				<TooltipTrigger asChild>
					<button
						type="button"
						aria-label={`About ${WORDS[option.kind]}`}
						className="text-muted-foreground"
					>
						<Info className="size-4" aria-hidden="true" />
					</button>
				</TooltipTrigger>
				<TooltipContent>{`${NOTES[option.kind]} Undo puts it back.`}</TooltipContent>
			</Tooltip>
		</div>
	);
}
