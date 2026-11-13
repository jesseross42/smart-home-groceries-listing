/**
 * The desktop notification for a finished trolley fill: how many products
 * went in, how many did not, and what to do next. Each handoff is announced
 * once, whichever of the open sheet or the server's push hears of it first.
 */
import type { TrolleyHandoff } from "#/lib/api/trolley-handoffs";
import { type Notice, notifyDesktop } from "#/lib/desktop/notify";

/** The notice for a trolley filled at `storeName`. */
export function filledNotice(
	storeName: string,
	added: number,
	failed: number,
): Notice {
	const parts = [`${added} added.`];
	if (failed > 0) parts.push(`${failed} not added.`);
	parts.push("Check out when ready.");
	return { title: `${storeName} trolley filled`, body: parts.join(" ") };
}

/** The notice for `handoff`, or `null` while it is not filled yet. */
export function handoffNotice(handoff: TrolleyHandoff): Notice | null {
	if (handoff.status !== "filled" && handoff.status !== "filled_with_problems")
		return null;
	const count = (outcome: "added" | "failed") =>
		handoff.lines.filter((line) => line.outcome === outcome).length;
	return filledNotice(handoff.store_name, count("added"), count("failed"));
}

/** Handoffs already announced in this window. */
const announced = new Set<string>();

/**
 * Shows `notice` for the handoff unless it was already announced here: the
 * sheet and the server's push both report the same fill, and a repeated push
 * must not notify twice.
 */
export function announceFilledOnce(handoffId: string, notice: Notice): void {
	if (announced.has(handoffId)) return;
	announced.add(handoffId);
	notifyDesktop(notice);
}

/** Forgets what was announced. For tests. */
export function resetAnnouncedHandoffs(): void {
	announced.clear();
}
