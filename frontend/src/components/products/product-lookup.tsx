import { type FormEvent, useState } from "react";

import { ChooseProductButton } from "#/components/products/choose-product-button";
import { ProductRow } from "#/components/products/product-row";
import { Button } from "#/components/ui/button";
import { Input } from "#/components/ui/input";
import { Label } from "#/components/ui/label";
import { ApiError, api, type StoreId, type StoreProduct } from "#/lib/api";
import { parseProductLink } from "#/lib/product-link";

const STORE_NAMES: Record<StoreId, string> = {
	woolworths: "Woolworths",
	coles: "Coles",
};

type Found =
	| { status: "idle" }
	| { status: "looking" }
	| { status: "found"; store: StoreId; product: StoreProduct }
	| { status: "failed"; message: string };

/**
 * "Look up by link": paste a Woolworths or Coles product link to see that
 * product and choose it, when the item's search does not show it. Priced
 * for one; the order screen prices the item's quantity.
 */
export function ProductLookup() {
	const [link, setLink] = useState("");
	const [found, setFound] = useState<Found>({ status: "idle" });

	async function lookUp(event: FormEvent) {
		event.preventDefault();
		const named = parseProductLink(link);
		if (!named) {
			setFound({
				status: "failed",
				message: "Paste a product link from woolworths.com.au or coles.com.au.",
			});
			return;
		}
		setFound({ status: "looking" });
		try {
			const product = await api.products.lookup(named.store, named.product_id);
			setFound({ status: "found", store: named.store, product });
		} catch (error) {
			setFound({
				status: "failed",
				message:
					error instanceof ApiError && error.status === 404
						? `${STORE_NAMES[named.store]} does not sell that product.`
						: `${STORE_NAMES[named.store]} could not be asked. Try again.`,
			});
		}
	}

	return (
		<section aria-label="Product lookup" className="flex flex-col gap-2">
			<form onSubmit={lookUp} className="flex flex-col gap-2">
				<Label htmlFor="product-link">Look up by link</Label>
				<div className="flex gap-2">
					<Input
						id="product-link"
						type="url"
						inputMode="url"
						placeholder="https://www.coles.com.au/product/…"
						value={link}
						onChange={(event) => setLink(event.target.value)}
					/>
					<Button
						type="submit"
						variant="outline"
						disabled={!link.trim() || found.status === "looking"}
					>
						{found.status === "looking" ? "Looking…" : "Look up"}
					</Button>
				</div>
			</form>
			{found.status === "failed" && (
				<p role="alert" className="text-sm text-destructive">
					{found.message}
				</p>
			)}
			{found.status === "found" && (
				<ProductRow
					product={found.product}
					quantity={1}
					storeName={STORE_NAMES[found.store]}
				>
					<ChooseProductButton
						store={found.store}
						storeName={STORE_NAMES[found.store]}
						product={found.product}
						lookedUp
					/>
				</ProductRow>
			)}
		</section>
	);
}
