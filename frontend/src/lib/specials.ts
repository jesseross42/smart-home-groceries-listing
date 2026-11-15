/**
 * The "specials only" display filter on the price comparison.
 *
 * "Specials only" narrows what is shown and nothing else. "Prefer specials"
 * (a display setting, off by default) moves promoted products to the top,
 * keeping the household's sort within each group. Neither changes a product
 * already chosen, and nothing is suggested.
 */
import type { StoreProduct } from "#/lib/api/products";
import type { StoreListing } from "#/lib/product-sort";

/** A product is on promotion: marked special, or carrying a multibuy. */
export function isPromoted(product: StoreProduct): boolean {
	return product.on_special || product.deals.length > 0;
}

/** The products to show, given whether only specials are wanted. */
export function visibleProducts(
	products: StoreProduct[],
	specialsOnly: boolean,
): StoreProduct[] {
	return specialsOnly ? products.filter(isPromoted) : products;
}

/** Promoted listings first, each group in the order it was given. */
export function specialsFirst(listings: StoreListing[]): StoreListing[] {
	return [
		...listings.filter(({ product }) => isPromoted(product)),
		...listings.filter(({ product }) => !isPromoted(product)),
	];
}
