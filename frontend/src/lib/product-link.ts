/**
 * Reads a store's product link into the store and the store's own product
 * id, for looking the product up (`GET /api/stores/{store}/products/{id}`).
 *
 * Link shapes, from the stores' websites:
 * - Woolworths: `https://www.woolworths.com.au/shop/productdetails/{stockcode}/{slug}`
 * - Coles: `https://www.coles.com.au/product/{slug}-{id}`
 */
import type { ProductChoice } from "#/lib/api";

/** The product a store link names, or `null` when it names none. */
export function parseProductLink(text: string): ProductChoice | null {
	let url: URL;
	try {
		url = new URL(text.trim());
	} catch {
		return null;
	}
	if (url.protocol !== "https:" && url.protocol !== "http:") return null;
	const host = url.hostname.toLowerCase();
	if (host === "woolworths.com.au" || host.endsWith(".woolworths.com.au")) {
		const match = /^\/shop\/productdetails\/(\d+)(\/|$)/i.exec(url.pathname);
		return match ? { store: "woolworths", product_id: match[1] } : null;
	}
	if (host === "coles.com.au" || host.endsWith(".coles.com.au")) {
		const match = /^\/product\/(?:[^/]*-)?(\d+)\/?$/i.exec(url.pathname);
		return match ? { store: "coles", product_id: match[1] } : null;
	}
	return null;
}
