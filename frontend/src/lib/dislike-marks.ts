/**
 * The household's dislikes as the grocery list and the order review need
 * them: read once by the route loader, shown as marks on chosen products.
 */
import { api, type DislikeOverride, type ProductDislike } from "#/lib/api";

/** Every member's dislikes and every "buy it this time" on the list. */
export interface DislikeMarks {
	dislikes: ProductDislike[];
	overrides: DislikeOverride[];
}

/** What a page shows when the dislikes cannot be read: no marks. */
export const NO_DISLIKE_MARKS: DislikeMarks = { dislikes: [], overrides: [] };

/**
 * Reads the dislikes and overrides for a route loader. A failure gives no
 * marks instead of an error: the list and the order matter more.
 */
export async function loadDislikeMarks(): Promise<DislikeMarks> {
	try {
		const [dislikes, overrides] = await Promise.all([
			api.dislikes.list(),
			api.dislikes.overrides(),
		]);
		return { dislikes, overrides };
	} catch {
		return NO_DISLIKE_MARKS;
	}
}
