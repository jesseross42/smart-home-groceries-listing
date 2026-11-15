import { Check } from "lucide-react";
import { useState } from "react";

import {
	isChosen,
	useProductChoice,
} from "#/components/products/product-choice";
import { Button } from "#/components/ui/button";
import type { StoreId, StoreProduct } from "#/lib/api";

/**
 * A toggle: chooses the product for the item, or, when it is chosen already,
 * clears it.
 *
 * Renders nothing outside a page that can choose. An unavailable product
 * cannot be chosen, and the button is disabled while saving so a double
 * click does not send two. `lookedUp` chooses a product found by its link
 * rather than in the item's search.
 */
export function ChooseProductButton({
	store,
	storeName,
	product,
	lookedUp = false,
}: {
	store: StoreId;
	storeName: string;
	product: StoreProduct;
	lookedUp?: boolean;
}) {
	const choice = useProductChoice();
	const [saving, setSaving] = useState(false);
	if (!choice) return null;

	const title = [product.brand, product.name].filter(Boolean).join(" ");
	const thisProduct = { store, product_id: product.product_id };
	const chosen = isChosen(choice.chosen, thisProduct);

	async function toggle() {
		if (!choice) return;
		setSaving(true);
		try {
			await (chosen
				? choice.onUnchoose(thisProduct)
				: choice.onChoose(
						lookedUp ? { ...thisProduct, looked_up: true } : thisProduct,
					));
		} catch {
			// The page has already said why; the button only has to recover.
		} finally {
			setSaving(false);
		}
	}

	return (
		<Button
			size="sm"
			variant={chosen ? "secondary" : "outline"}
			aria-pressed={chosen}
			aria-label={
				chosen
					? `${title} at ${storeName} is chosen`
					: `Choose ${title} at ${storeName}`
			}
			disabled={saving || (!chosen && !product.available)}
			onClick={toggle}
		>
			{chosen && <Check aria-hidden />}
			{chosen ? "Chosen" : saving ? "Choosing…" : "Choose"}
		</Button>
	);
}
