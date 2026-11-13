import { TriangleAlert } from "lucide-react";
import { createContext, useContext } from "react";

import type { StoreId } from "#/lib/api";
import type { DislikeMarks } from "#/lib/dislike-marks";
import { dislikesOf, dislikeWarning, isOverridden } from "#/lib/dislikes";

const DislikeMarksContext = createContext<DislikeMarks | null>(null);

/** Gives one page's chosen products the household's dislikes. Never global. */
export const DislikeMarksProvider = DislikeMarksContext.Provider;

/**
 * The read-only warning on a chosen product: "Phu disliked this item
 * previously.", and "OK to buy it this time." when the dislike is set aside
 * for the item. It never blocks or suggests anything; changing a dislike
 * happens on the Select item page.
 *
 * Renders nothing when nobody dislikes the product, or outside a provider.
 */
export function DislikeMark({
	itemId,
	store,
	productId,
}: {
	itemId: string;
	store: StoreId;
	productId: string;
}) {
	const marks = useContext(DislikeMarksContext);
	if (!marks) return null;

	const choice = { store, product_id: productId };
	const dislikes = dislikesOf(marks.dislikes, choice);
	if (dislikes.length === 0) return null;

	return (
		<p
			data-testid="dislike-mark"
			className="flex flex-wrap items-center gap-1 text-xs"
		>
			<TriangleAlert aria-hidden className="size-3 text-destructive" />
			<output>{dislikeWarning(dislikes)}</output>
			{isOverridden(marks.overrides, itemId, choice) && (
				<span className="text-muted-foreground">OK to buy it this time.</span>
			)}
		</p>
	);
}
