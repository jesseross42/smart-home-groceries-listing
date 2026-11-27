/**
 * The desktop notification for a finished trolley fill: how many products
 * went in, how many did not, and what to do next. The same words whether
 * the Send sheet saw the fill or the server pushed it.
 */
import type { TrolleyHandoff } from "#/lib/api/trolley-handoffs";
import type { Notice } from "#/lib/desktop/notify";

/** The notice for a fill at `storeName` that put in `added`, not `notAdded`. */
export function filledNotice(
	storeName: string,
	added: number,
	notAdded: number,
): Notice {
	const parts = [`${added} added.`];
	if (notAdded > 0) parts.push(`${notAdded} not added.`);
	parts.push("Check out when ready.");
	return { title: `${storeName} trolley filled`, body: parts.join(" ") };
}

/** The notice for `handoff`, or `null` while it is not filled yet. */
export function handoffNotice(handoff: TrolleyHandoff): Notice | null {
	if (handoff.status !== "filled" && handoff.status !== "filled_with_problems")
		return null;
	const count = (outcome: string) =>
		handoff.lines.filter((line) => line.outcome === outcome).length;
	return filledNotice(handoff.store_name, count("added"), count("failed"));
}
